//! Drift-hardening parity guard for the `bun-dev` rule corpus.
//!
//! `bun audit` / `bun fixes` emit `rule_id` string literals (from `audit.rs` and
//! `fixes.rs`) that `bun rules show <id>` resolves by reading `rules/<id>.md`. Nothing
//! at runtime forces those two to agree, so a renamed or deleted detector-backed rule
//! silently drifts: the audit emits an id that `rules show` can no longer open.
//!
//! This test enforces the invariant statically:
//!   1. every rule id emitted by the detectors resolves to a `rules/<id>.md` file;
//!   2. every `rules/*.md` file is listed in the generated `rules/_index.md` (no orphans).
//!
//! Scope: detector-emitted ids only (a source scan of `audit.rs` + `fixes.rs`). The
//! capability-map gap detector in `release_sync.rs` intentionally references ids that
//! have no rule file (docs mention a capability we have no rule for), so it is
//! deliberately excluded here.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const RULE_PREFIXES: &[&str] = &[
    "pm-",
    "runtime-",
    "vercel-",
    "scripts-",
    "tsconfig-",
    "test-",
    "build-",
    "perf-",
    "migrate-",
    "troubleshooting-",
    "tooling-",
];

fn crate_src(file: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(file)
}

fn rules_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skills/bun-dev/rules")
}

fn looks_like_rule_id(s: &str) -> bool {
    RULE_PREFIXES.iter().any(|p| s.starts_with(p))
        && s.len() >= 4
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.ends_with('-')
        && !s.contains("--")
}

/// Extract rule-id-shaped string literals from Rust source, honoring `\"` escapes so a
/// message literal that contains a quote cannot desync the scan.
fn extract_rule_ids(source: &str) -> BTreeSet<String> {
    let bytes = source.as_bytes();
    let mut ids = BTreeSet::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'"' {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut j = start;
        while j < bytes.len() {
            match bytes[j] {
                b'\\' => j += 2,
                b'"' => break,
                _ => j += 1,
            }
        }
        let end = j.min(bytes.len());
        if let Ok(literal) = std::str::from_utf8(&bytes[start..end])
            && looks_like_rule_id(literal)
        {
            ids.insert(literal.to_string());
        }
        i = end + 1;
    }
    ids
}

fn rule_files() -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    for entry in fs::read_dir(rules_dir()).expect("rules dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("utf8 file stem")
            .to_string();
        if stem != "_index" {
            files.insert(stem);
        }
    }
    files
}

#[test]
fn every_detector_rule_id_has_a_rule_file() {
    let mut emitted = BTreeSet::new();
    for src in ["audit.rs", "fixes.rs"] {
        let source = fs::read_to_string(crate_src(src)).expect("read detector source");
        emitted.extend(extract_rule_ids(&source));
    }
    assert!(
        !emitted.is_empty(),
        "extracted zero detector rule ids; the extractor or source layout changed"
    );

    let files = rule_files();
    let missing: Vec<&String> = emitted.iter().filter(|id| !files.contains(*id)).collect();
    assert!(
        missing.is_empty(),
        "detector emits rule ids with no matching rules/<id>.md (rules show would fail): {missing:?}"
    );
}

#[test]
fn every_rule_file_is_listed_in_index() {
    let index = fs::read_to_string(rules_dir().join("_index.md")).expect("read _index.md");
    let orphans: Vec<String> = rule_files()
        .into_iter()
        .filter(|id| !index.contains(&format!("`{id}`")))
        .collect();
    assert!(
        orphans.is_empty(),
        "rules/*.md files missing from _index.md (run `codex-dev bun references sync`): {orphans:?}"
    );
}
