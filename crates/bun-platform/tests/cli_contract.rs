use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_bun-platform")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures_root() -> PathBuf {
    workspace_root().join("crates/bun-platform-core/fixtures")
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(binary())
        .args(args)
        .output()
        .expect("run bun-platform")
}

fn stdout_string(output: Output) -> String {
    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

fn normalize(text: &str) -> String {
    let normalized = text.replace("\r\n", "\n");
    let mut lines = normalized
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n");
    if normalized.ends_with('\n') {
        lines.push('\n');
    }
    lines
}

fn golden(name: &str) -> &'static str {
    match name {
        "help" => include_str!("golden/help.txt"),
        "benchmark-help" => include_str!("golden/benchmark-help.txt"),
        "release-sync-help" => include_str!("golden/release-sync-help.txt"),
        _ => panic!("unknown golden {name}"),
    }
}

#[test]
fn top_level_help_matches_golden() {
    let stdout = stdout_string(run_cli(&["--help"]));
    assert_eq!(normalize(&stdout), golden("help"));
}

#[test]
fn benchmark_help_matches_golden() {
    let stdout = stdout_string(run_cli(&["benchmark", "--help"]));
    assert_eq!(normalize(&stdout), golden("benchmark-help"));
}

#[test]
fn release_sync_help_matches_golden() {
    let stdout = stdout_string(run_cli(&["release-sync", "--help"]));
    assert_eq!(normalize(&stdout), golden("release-sync-help"));
}

#[test]
fn benchmark_json_output_has_expected_shape() {
    let root = fixtures_root().join("github-actions");
    let root_arg = root.to_string_lossy().to_string();
    let canonical_root = fs::canonicalize(&root)
        .expect("canonical fixture root")
        .to_string_lossy()
        .to_string();
    let stdout = stdout_string(run_cli(&[
        "benchmark",
        "--root",
        &root_arg,
        "--iterations",
        "1",
        "--format",
        "json",
    ]));
    let json = serde_json::from_str::<Value>(&stdout).expect("benchmark json");

    assert_eq!(json["iterations"].as_u64(), Some(1));
    assert_eq!(json["root"].as_str(), Some(canonical_root.as_str()));
    assert_eq!(json["audit_ms"].as_array().map(Vec::len), Some(1));
    assert_eq!(json["plan_fix_ms"].as_array().map(Vec::len), Some(1));
    for key in ["min_ms", "median_ms", "max_ms", "mean_ms"] {
        assert!(json["summary"]["audit"][key].is_number());
        assert!(json["summary"]["plan_fixes"][key].is_number());
    }
}

#[test]
fn audit_json_output_has_expected_fields() {
    let root = fixtures_root().join("github-actions");
    let root_arg = root.to_string_lossy().to_string();
    let stdout = stdout_string(run_cli(&["audit", "--root", &root_arg, "--format", "json"]));
    let json = serde_json::from_str::<Value>(&stdout).expect("audit json");
    let findings = json.as_array().expect("findings array");
    assert!(!findings.is_empty());
    let finding = findings
        .iter()
        .find(|finding| finding["rule_id"].as_str() == Some("scripts-no-npm-in-bun-repos"))
        .expect("scripts-no-npm-in-bun-repos finding");
    assert_eq!(
        finding["rule_id"].as_str(),
        Some("scripts-no-npm-in-bun-repos")
    );
    for key in [
        "rule_id",
        "category",
        "severity",
        "confidence",
        "file",
        "line",
        "column",
        "message",
        "suppression_key",
    ] {
        assert!(finding.get(key).is_some(), "missing key `{key}`");
    }
}
