//! Integration tests for the gsap-audit rule engine.
//!
//! Each test parses an inline source snippet and asserts that the expected rule
//! id does (or does not) fire. Snippets are deliberately small so the assertion
//! pins one behavior at a time.

use gsap_audit_core::analyze_source;
use gsap_audit_core::rules::ids;
use gsap_audit_core::source::source_type_for_extension;
use gsap_audit_core::types::{Category, Finding};

/// Analyze a snippet under a given file path + extension.
fn analyze(path: &str, ext: &str, source: &str) -> Vec<Finding> {
    analyze_source(path, source, source_type_for_extension(ext))
}

/// Whether any finding has the given id.
fn fired(findings: &[Finding], id: &str) -> bool {
    findings.iter().any(|finding| finding.id == id)
}

/// Count findings with the given id.
fn count(findings: &[Finding], id: &str) -> usize {
    findings.iter().filter(|finding| finding.id == id).count()
}

#[test]
fn rule_gsap_trial_import_fires_and_clean_does_not() {
    let bad = analyze("src/a.ts", "ts", r#"import { Flip } from "gsap-trial";"#);
    assert!(fired(&bad, ids::CORE_GSAP_TRIAL_IMPORT));

    let bad_subpath = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger } from "gsap-trial/ScrollTrigger";"#,
    );
    assert!(fired(&bad_subpath, ids::CORE_GSAP_TRIAL_IMPORT));

    let clean = analyze("src/a.ts", "ts", r#"import { Flip } from "gsap/Flip";"#);
    assert!(!fired(&clean, ids::CORE_GSAP_TRIAL_IMPORT));
}

#[test]
fn rule_gsdevtools_in_source() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(fired(&bad, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let clean = analyze("src/a.ts", "ts", r#"const x = 1; export { x };"#);
    assert!(!fired(&clean, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let test_file = analyze(
        "src/a.test.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(!fired(&test_file, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let fixture = analyze(
        "src/fixtures/gsdevtools.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(!fired(&fixture, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));
}

#[test]
fn rule_markers_in_prod() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { scrollTrigger: { trigger: ".box", markers: true } });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { scrollTrigger: { trigger: ".box", markers: false } });"#,
    );
    assert!(!fired(&clean, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
}

#[test]
fn rule_scrub_with_toggleactions_conflict() {
    // Inside a ScrollTrigger.create config -> fires (GSAP context).
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ trigger: ".x", scrub: true, toggleActions: "play none none reverse" });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ trigger: ".x", scrub: true });"#,
    );
    assert!(!fired(&clean, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));
}

#[test]
fn rule_gsap2_signature() {
    let bad = analyze("src/a.ts", "ts", r#"gsap.to(".box", 1.5, { x: 100 });"#);
    assert!(fired(&bad, ids::CORE_GSAP2_SIGNATURE));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { duration: 1.5, x: 100 });"#,
    );
    assert!(!fired(&clean, ids::CORE_GSAP2_SIGNATURE));
}

#[test]
fn rule_gsap2_signature_categorized_as_core() {
    // Fix 7: the GSAP-2 signature rule fires on core gsap.to/from/fromTo and
    // must be categorized as Core (not Timeline) and use the `core.` id, so a
    // `--categories core` filter does not miss it.
    let findings = analyze("src/a.ts", "ts", r#"gsap.to(".box", 1.5, { x: 100 });"#);
    let finding = findings
        .iter()
        .find(|finding| finding.id == ids::CORE_GSAP2_SIGNATURE)
        .expect("gsap2-signature finding");
    assert_eq!(finding.id, "core.gsap2-signature");
    assert_eq!(finding.category, Category::Core);
}

#[test]
fn rule_lag_smoothing_disabled() {
    let bad_zero = analyze("src/a.ts", "ts", r#"gsap.ticker.lagSmoothing(0);"#);
    assert!(fired(&bad_zero, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));

    let bad_false = analyze("src/a.ts", "ts", r#"gsap.ticker.lagSmoothing(false);"#);
    assert!(fired(&bad_false, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));

    let clean = analyze("src/a.ts", "ts", r#"gsap.ticker.lagSmoothing(500, 33);"#);
    assert!(!fired(&clean, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));
}

#[test]
fn rule_layout_prop_animation() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { top: 100, left: 50 });"#,
    );
    assert!(fired(&bad, ids::CORE_LAYOUT_PROP_ANIMATION));

    let clean = analyze("src/a.ts", "ts", r#"gsap.to(".box", { x: 100, y: 50 });"#);
    assert!(!fired(&clean, ids::CORE_LAYOUT_PROP_ANIMATION));
}

#[test]
fn rule_plugin_used_without_register() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(ScrollTrigger); ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(&clean, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    let custom_ease = analyze(
        "src/a.ts",
        "ts",
        r#"CustomEase.create("hop", "M0,0 C0.1,0.8 0.2,1 1,0");"#,
    );
    assert!(fired(
        &custom_ease,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let registered_custom_ease = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(CustomEase); CustomEase.create("hop", "M0,0 C0.1,0.8 0.2,1 1,0");"#,
    );
    assert!(!fired(
        &registered_custom_ease,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));
}

#[test]
fn rule_usegsap_not_registered() {
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP } from "@gsap/react";
function C() { useGSAP(() => {}); return null; }"#,
    );
    assert!(fired(&bad, ids::REACT_USEGSAP_NOT_REGISTERED));

    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP } from "@gsap/react";
import { gsap } from "gsap";
gsap.registerPlugin(useGSAP);
function C() { useGSAP(() => {}); return null; }"#,
    );
    assert!(!fired(&clean, ids::REACT_USEGSAP_NOT_REGISTERED));
}

#[test]
fn rule_gsap_in_ssr() {
    let bad = analyze(
        "app/page.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(fired(&bad, ids::REACT_GSAP_IN_SSR));

    let clean = analyze(
        "app/page.tsx",
        "tsx",
        r#""use client";
import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&clean, ids::REACT_GSAP_IN_SSR));

    // A file outside app should not trigger SSR even without use client.
    let outside = analyze(
        "src/widget.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function W() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&outside, ids::REACT_GSAP_IN_SSR));

    let pages_router = analyze(
        "pages/index.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&pages_router, ids::REACT_GSAP_IN_SSR));

    let type_only = analyze(
        "app/page.tsx",
        "tsx",
        r#"import type { ScrollTrigger } from "gsap/ScrollTrigger";
type Props = { trigger: ScrollTrigger };
export default function Page(_props: Props) { return null; }"#,
    );
    assert!(!fired(&type_only, ids::REACT_GSAP_IN_SSR));
}

#[test]
fn rule_unscoped_selector_semantic() {
    // useGSAP with a string selector and no scope -> fires.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  });
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_UNSCOPED_SELECTOR));

    // gsap.context with a string selector and no scope -> fires.
    let bad_context = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(".box", { x: 100 });
  });
  return () => ctx.revert();
}"#,
    );
    assert!(fired(&bad_context, ids::REACT_UNSCOPED_SELECTOR));
}

#[test]
fn rule_unscoped_selector_clean_with_scope() {
    // useGSAP with a scope config -> does NOT fire (clean React snippet).
    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const container = useRef(null);
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, { scope: container });
  return null;
}"#,
    );
    assert!(!fired(&clean, ids::REACT_UNSCOPED_SELECTOR));

    // gsap.context with a scope ref argument -> does NOT fire.
    let clean_context = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(".box", { x: 100 });
  }, containerRef);
  return () => ctx.revert();
}"#,
    );
    assert!(!fired(&clean_context, ids::REACT_UNSCOPED_SELECTOR));
}

#[test]
fn rule_context_missing_revert_semantic() {
    // ctx created, never reverted, never returned -> fires.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(refEl, { x: 100 });
  }, scopeRef);
  console.log(ctx);
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_CONTEXT_MISSING_REVERT));

    // ctx reverted in returned cleanup -> does NOT fire.
    let clean_revert = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(refEl, { x: 100 });
  }, scopeRef);
  return () => ctx.revert();
}"#,
    );
    assert!(!fired(&clean_revert, ids::REACT_CONTEXT_MISSING_REVERT));

    let property_read = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {});
  console.log(ctx.revert);
  return null;
}"#,
    );
    assert!(fired(&property_read, ids::REACT_CONTEXT_MISSING_REVERT));

    let argument_to_other_call = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {});
  foo.revert(ctx);
  return null;
}"#,
    );
    assert!(fired(
        &argument_to_other_call,
        ids::REACT_CONTEXT_MISSING_REVERT
    ));

    let discarded = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  gsap.context(() => {});
  return null;
}"#,
    );
    assert!(fired(&discarded, ids::REACT_CONTEXT_MISSING_REVERT));
}

#[test]
fn clean_react_usegsap_snippet_has_zero_findings() {
    // A fully idiomatic React + useGSAP snippet should yield no findings at all.
    let clean = analyze(
        "src/Component.tsx",
        "tsx",
        r#""use client";
import { gsap } from "gsap";
import { useGSAP } from "@gsap/react";
import { useRef } from "react";

gsap.registerPlugin(useGSAP);

export default function Component() {
  const container = useRef(null);
  useGSAP(
    () => {
      gsap.to(boxRef.current, { duration: 1, x: 100 });
    },
    { scope: container }
  );
  return null;
}"#,
    );
    assert_eq!(clean, Vec::new(), "expected zero findings, got: {clean:#?}");
}

#[test]
fn category_filtering_independent_of_rule_count() {
    // Sanity: a snippet with multiple rule classes produces multiple ids.
    let findings = analyze(
        "src/a.ts",
        "ts",
        r#"import { Flip } from "gsap-trial";
gsap.to(".box", 1, { top: 0, markers: true });
gsap.ticker.lagSmoothing(0);"#,
    );
    assert!(fired(&findings, ids::CORE_GSAP_TRIAL_IMPORT));
    assert!(fired(&findings, ids::CORE_GSAP2_SIGNATURE));
    assert!(fired(&findings, ids::CORE_LAYOUT_PROP_ANIMATION));
    assert!(fired(&findings, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
    assert!(fired(&findings, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));
    // No duplicate trial-import findings for a single import.
    assert_eq!(count(&findings, ids::CORE_GSAP_TRIAL_IMPORT), 1);
}

// ---------------------------------------------------------------------------
// Fix 1: context-missing-revert must not be suppressed by an unrelated return.
// ---------------------------------------------------------------------------

#[test]
fn rule_context_missing_revert_return_jsx_using_ctx_still_fires() {
    // `return <div>{String(ctx)}</div>` reads ctx but does NOT tear it down.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {});
  return <div>{String(ctx)}</div>;
}"#,
    );
    assert!(fired(&bad, ids::REACT_CONTEXT_MISSING_REVERT));
}

#[test]
fn rule_context_missing_revert_bare_return_ctx_does_not_fire() {
    // `return ctx;` hands the handle to a parent that can revert it.
    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {});
  return ctx;
}"#,
    );
    assert!(!fired(&clean, ids::REACT_CONTEXT_MISSING_REVERT));
}

#[test]
fn rule_context_missing_revert_returned_cleanup_does_not_fire() {
    // `return () => ctx.revert();` is the canonical cleanup pattern.
    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {});
  return () => ctx.revert();
}"#,
    );
    assert!(!fired(&clean, ids::REACT_CONTEXT_MISSING_REVERT));
}

// ---------------------------------------------------------------------------
// Fix 2: GSDevTools type-only references must not flag.
// ---------------------------------------------------------------------------

#[test]
fn rule_gsdevtools_type_position_does_not_fire() {
    let var_type = analyze("src/a.ts", "ts", r#"let x: GSDevTools;"#);
    assert!(!fired(&var_type, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let param_type = analyze(
        "src/a.ts",
        "ts",
        r#"function f(p: GSDevTools) { return p; }"#,
    );
    assert!(!fired(&param_type, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));
}

#[test]
fn rule_gsdevtools_value_use_still_fires() {
    let value_call = analyze("src/a.ts", "ts", r#"GSDevTools.create();"#);
    assert!(fired(&value_call, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let value_import = analyze(
        "src/a.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(fired(&value_import, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));
}

// ---------------------------------------------------------------------------
// Fix 3: markers / scrub+toggleActions gated to GSAP/ScrollTrigger context.
// ---------------------------------------------------------------------------

#[test]
fn rule_markers_unrelated_object_does_not_fire() {
    let unrelated = analyze("src/a.ts", "ts", r#"const opts = { markers: true };"#);
    assert!(!fired(&unrelated, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
}

#[test]
fn rule_markers_in_nested_scrolltrigger_fires() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".x", { scrollTrigger: { markers: true } });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let timeline = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline({ scrollTrigger: { trigger: ".x", markers: true } });"#,
    );
    assert!(fired(&timeline, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
}

#[test]
fn rule_scrub_toggleactions_in_scrolltrigger_create_fires() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ scrub: 1, toggleActions: "play none none none" });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));

    let timeline = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline({ scrollTrigger: { scrub: 1, toggleActions: "play none none none" } });"#,
    );
    assert!(fired(
        &timeline,
        ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS
    ));
}

#[test]
fn rule_scrub_toggleactions_unrelated_object_does_not_fire() {
    let unrelated = analyze(
        "src/a.ts",
        "ts",
        r#"const opts = { scrub: 1, toggleActions: "play none none none" };"#,
    );
    assert!(!fired(
        &unrelated,
        ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS
    ));
}

// ---------------------------------------------------------------------------
// Fix 4: fromTo layout-prop check inspects both fromVars and toVars.
// ---------------------------------------------------------------------------

#[test]
fn rule_layout_prop_fromto_scans_fromvars() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.fromTo(".b", { top: 0 }, { x: 100 });"#,
    );
    assert!(fired(&bad, ids::CORE_LAYOUT_PROP_ANIMATION));
}

#[test]
fn rule_layout_prop_fromto_scans_tovars() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.fromTo(".b", { x: 0 }, { top: 100 });"#,
    );
    assert!(fired(&bad, ids::CORE_LAYOUT_PROP_ANIMATION));
}

// ---------------------------------------------------------------------------
// Fix 5: registerPlugin argument handling.
// ---------------------------------------------------------------------------

#[test]
fn rule_plugin_register_via_array_still_fires() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin([ScrollTrigger]); ScrollTrigger.create({});"#,
    );
    assert!(fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

#[test]
fn rule_plugin_register_via_spread_suppresses_check() {
    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(...plugins); ScrollTrigger.create({});"#,
    );
    assert!(!fired(&clean, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

// ---------------------------------------------------------------------------
// Fix 6: unscoped-selector traversal reaches loop/try/switch bodies.
// ---------------------------------------------------------------------------

#[test]
fn rule_unscoped_selector_inside_for_loop_fires() {
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    for (let i = 0; i < 3; i++) {
      gsap.to(".box", { x: 1 });
    }
  });
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_UNSCOPED_SELECTOR));
}

// ---------------------------------------------------------------------------
// Fix 8: lagSmoothing(-0) is treated as disabled.
// ---------------------------------------------------------------------------

#[test]
fn rule_lag_smoothing_negative_zero_fires() {
    let bad = analyze("src/a.ts", "ts", r#"gsap.ticker.lagSmoothing(-0);"#);
    assert!(fired(&bad, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));
}

// ---------------------------------------------------------------------------
// Review fix: ScrollTrigger used via a `scrollTrigger:` tween/timeline config
// (not only `ScrollTrigger.create`) counts as usage for missing-registration.
// ---------------------------------------------------------------------------

#[test]
fn rule_scrolltrigger_config_without_register_fires() {
    // gsap.to(target, { scrollTrigger: {...} }) without registering ScrollTrigger.
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // gsap.timeline({ scrollTrigger: {...} }) likewise.
    let bad_tl = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline({ scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(fired(&bad_tl, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // Registered -> does NOT fire.
    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(ScrollTrigger); gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(!fired(&clean, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // No scrollTrigger config -> does NOT fire.
    let plain = analyze("src/a.ts", "ts", r#"gsap.to(".x", { x: 100 });"#);
    assert!(!fired(&plain, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

#[test]
fn rule_configured_gsap_reexports_do_not_require_local_register() {
    let configured_plugin = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger } from "@/lib/gsap";
ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(
        &configured_plugin,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_gsap = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap } from "@/lib/gsap";
gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(!fired(
        &configured_gsap,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_gsap_direct_plugin = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap } from "@/lib/gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(
        &configured_gsap_direct_plugin,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));
}

// ---------------------------------------------------------------------------
// Review fix: useGSAP(cb, []) dependency-array overload is not a scope, so an
// unscoped selector inside it still fires.
// ---------------------------------------------------------------------------

#[test]
fn rule_unscoped_selector_with_dependency_array_fires() {
    // useGSAP(cb, []) -> the array is deps, not a scope -> fires.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, []);
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_UNSCOPED_SELECTOR));

    // useGSAP(cb, { dependencies: [...] }) without a scope key -> still fires.
    let bad_config = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, { dependencies: [a] });
  return null;
}"#,
    );
    assert!(fired(&bad_config, ids::REACT_UNSCOPED_SELECTOR));

    // gsap.context(cb, scopeRef) -> the bare ref IS a scope -> does NOT fire.
    let clean_context = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(".box", { x: 100 });
  }, root);
  return () => ctx.revert();
}"#,
    );
    assert!(!fired(&clean_context, ids::REACT_UNSCOPED_SELECTOR));
}
