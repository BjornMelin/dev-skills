//! Integration tests for the motion-token-audit rule engine.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use motion_token_audit_core::output::format_catalog_json;
use motion_token_audit_core::rules::{CATALOG, ids};
use motion_token_audit_core::source::source_type_for_extension;
use motion_token_audit_core::{
    Category, Confidence, Finding, MotionTokens, ScanOptions, Severity, TOOL_NAME, TOOL_VERSION,
    analyze_css, analyze_source, discover_css_tokens, discover_tokens, scan_root,
};

fn token_source() -> &'static str {
    r#"export const motion = {
  duration: { instant: 0, short: 200, medium: 360 },
  easing: { out: [0.16, 1, 0.3, 1] },
  spring: { snappy: { stiffness: 520, damping: 42, mass: 1 } },
} as const;"#
}

fn tokens() -> MotionTokens {
    discover_tokens(token_source(), source_type_for_extension("ts"))
}

fn ids(findings: &[Finding]) -> Vec<String> {
    findings.iter().map(|finding| finding.id.clone()).collect()
}

#[test]
fn discovers_ssot_from_motion_ts_shape() {
    let tokens = tokens();
    assert!(tokens.has_duration_ms(200));
    assert!(!tokens.is_empty());
}

#[test]
fn discovers_ssot_from_css_custom_properties() {
    let tokens = discover_css_tokens(
        r#":root {
  --motion-duration-short: 0.2s;
  --motion-ease-out: cubic-bezier(0.16, 1, 0.3, 1);
}"#,
    );
    assert!(tokens.has_duration_ms(200));
    assert!(!tokens.is_empty());
}

#[test]
fn discovers_duration_only_token_module_as_ssot() {
    let root = temp_scan_root("duration-only-ssot");
    fs::write(
        root.join("motion.ts"),
        "export const motion = { duration: { short: 200 } } as const;",
    )
    .unwrap();
    fs::write(root.join("app.ts"), "withTiming(1, { duration: 200 });").unwrap();

    let outcome = scan_root(&ScanOptions::new(root.clone(), BTreeSet::new(), 5000)).unwrap();

    assert!(!ids(&outcome.findings).contains(&ids::SSOT_NO_TOKEN_MODULE.to_string()));
    assert!(outcome.findings.iter().any(|finding| {
        finding.id == ids::REANIMATED_DURATION_LITERAL && finding.severity == Severity::Medium
    }));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn discovers_all_minified_css_custom_properties() {
    let tokens = discover_css_tokens(
        ":root{--motion-duration-short:200ms;--motion-duration-medium:360ms;--motion-ease-out:cubic-bezier(0.16, 1, 0.3, 1);}",
    );

    assert!(tokens.has_duration_ms(200));
    assert!(tokens.has_duration_ms(360));
    assert!(!tokens.is_empty());
}

#[test]
fn reanimated_known_duration_is_drift() {
    let analysis = analyze_source(
        "app.ts",
        "withTiming(1, { duration: 200 });",
        source_type_for_extension("ts"),
        &tokens(),
    );
    assert!(ids(&analysis.findings).contains(&ids::REANIMATED_DURATION_LITERAL.to_string()));
    assert_eq!(analysis.findings.len(), 1);
    let finding = &analysis.findings[0];
    assert_eq!(finding.severity, Severity::Medium);
    assert!(finding.message.starts_with("drift:"));
    assert!(finding.message.contains("200ms"));
    assert!(!finding.message.contains("1ms"));
}

#[test]
fn reanimated_unknown_duration_is_orphan() {
    let analysis = analyze_source(
        "app.ts",
        "withTiming(1, { duration: 237 });",
        source_type_for_extension("ts"),
        &tokens(),
    );
    let finding = &analysis.findings[0];
    assert_eq!(finding.id, ids::REANIMATED_DURATION_LITERAL);
    assert_eq!(finding.severity, Severity::Low);
    assert!(finding.message.starts_with("orphan:"));
}

#[test]
fn reanimated_partial_spring_config_is_orphan() {
    let analysis = analyze_source(
        "app.ts",
        "withSpring(1, { stiffness: 520, damping: 42 });",
        source_type_for_extension("ts"),
        &tokens(),
    );

    let finding = &analysis.findings[0];
    assert_eq!(finding.id, ids::REANIMATED_SPRING_LITERAL);
    assert_eq!(finding.severity, Severity::Low);
    assert!(finding.message.starts_with("orphan:"));
}

#[test]
fn css_transition_duration_classifies_drift_and_orphan() {
    let analysis = analyze_css(
        "style.css",
        ".a { transition: opacity 200ms ease; }\n.b { animation-duration: 150ms; }",
        &tokens(),
    );
    assert_eq!(analysis.findings.len(), 2);
    assert_eq!(analysis.findings[0].severity, Severity::Medium);
    assert!(analysis.findings[0].message.starts_with("drift:"));
    assert_eq!(analysis.findings[1].severity, Severity::Low);
    assert!(analysis.findings[1].message.starts_with("orphan:"));
}

#[test]
fn css_mixed_tokenized_declaration_still_flags_literal() {
    let analysis = analyze_css(
        "style.css",
        ".a { transition: opacity var(--motion-duration-short) ease, transform 237ms ease; }",
        &tokens(),
    );

    assert_eq!(analysis.findings.len(), 1);
    assert_eq!(analysis.findings[0].id, ids::CSS_DURATION_LITERAL);
    assert_eq!(analysis.findings[0].severity, Severity::Low);
}

#[test]
fn seconds_normalize_to_milliseconds_across_css_and_gsap() {
    let css = analyze_css("style.css", ".a { transition-duration: 0.2s; }", &tokens());
    assert_eq!(css.findings[0].severity, Severity::Medium);

    let gsap = analyze_source(
        "app.ts",
        r#"gsap.to(".box", { duration: 0.2 });"#,
        source_type_for_extension("ts"),
        &tokens(),
    );
    assert!(ids(&gsap.findings).contains(&ids::GSAP_DURATION_LITERAL.to_string()));
    assert_eq!(gsap.findings[0].severity, Severity::Medium);
}

#[test]
fn gsap_easing_uses_catalog_confidence() {
    let analysis = analyze_source(
        "app.ts",
        r#"gsap.to(".box", { ease: "power2.out" });"#,
        source_type_for_extension("ts"),
        &tokens(),
    );

    assert_eq!(analysis.findings[0].id, ids::GSAP_EASING_LITERAL);
    assert_eq!(analysis.findings[0].confidence, Confidence::Medium);
}

#[test]
fn gsap_rules_ignore_non_gsap_to_calls() {
    let analysis = analyze_source(
        "app.ts",
        r#"router.to({ duration: 1 }); gsap.to(".x", { duration: 1 });"#,
        source_type_for_extension("ts"),
        &tokens(),
    );
    assert_eq!(analysis.findings.len(), 1);
    assert_eq!(analysis.findings[0].id, ids::GSAP_DURATION_LITERAL);
}

#[test]
fn motion_jsx_element_does_not_count_as_tokenized_reference() {
    let analysis = analyze_source(
        "app.tsx",
        "const element = <motion.div animate={{ opacity: 1 }} />;",
        source_type_for_extension("tsx"),
        &tokens(),
    );
    assert!(analysis.findings.is_empty());
    assert!(
        analysis
            .coverage
            .iter()
            .all(|entry| entry.tokenized_references == 0)
    );
}

#[test]
fn scan_without_ssot_reports_low_no_token_module_and_orphans() {
    let root = temp_scan_root("no-ssot");
    fs::write(root.join("app.ts"), "withTiming(1, { duration: 200 });\n").unwrap();
    let outcome = scan_root(&ScanOptions::new(root.clone(), BTreeSet::new(), 5000)).unwrap();

    assert!(ids(&outcome.findings).contains(&ids::SSOT_NO_TOKEN_MODULE.to_string()));
    assert!(outcome.findings.iter().any(|finding| {
        finding.id == ids::REANIMATED_DURATION_LITERAL && finding.severity == Severity::Low
    }));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn category_filter_keeps_requested_findings() {
    let root = temp_scan_root("categories");
    fs::write(root.join("motion.ts"), token_source()).unwrap();
    fs::write(
        root.join("app.ts"),
        r#"withTiming(1, { duration: 200 }); gsap.to(".box", { duration: 0.2 });"#,
    )
    .unwrap();
    let mut categories = BTreeSet::new();
    categories.insert(Category::TokensGsap);
    let outcome = scan_root(&ScanOptions::new(root.clone(), categories, 5000)).unwrap();

    assert!(
        outcome
            .findings
            .iter()
            .all(|finding| { finding.category == Category::TokensGsap })
    );
    assert_eq!(outcome.findings.len(), 1);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn doctor_catalog_lists_every_rule() {
    let value = format_catalog_json(TOOL_NAME, TOOL_VERSION);
    let rules = value["rules"].as_array().unwrap();
    assert_eq!(rules.len(), CATALOG.len());
    for rule in CATALOG {
        assert!(rules.iter().any(|value| value["id"] == rule.id));
    }
}

fn temp_scan_root(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "motion-token-audit-core-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
