//! Shared CI-gate layer for the static auditors.
//!
//! `gsap-audit`, `expo-motion-audit` and `motion-token-audit` each produce
//! findings with the same shape and each needs the same three things to be
//! usable as a gate on a repository that is not already clean:
//!
//! - **exclusion**, so vendored and generated trees do not decide the exit code;
//! - **a baseline**, so a repository with known findings can block only on NEW
//!   ones while the backlog is worked down;
//! - **SARIF**, so findings render as annotations on the diff rather than as a
//!   log a human has to read.
//!
//! The auditors keep their own rule catalogs and severities; they map into
//! [`GateFinding`] at the boundary. That is deliberately a conversion rather
//! than a shared type: the cores stay independent, and this crate stays free of
//! any parser dependency.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Severity as the gate understands it. Mirrors each core's own enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GateSeverity {
    Low,
    Medium,
    High,
}

impl GateSeverity {
    /// Parse the lowercase form the auditors already emit.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    /// SARIF has no "medium"; it uses `error`, `warning`, `note`.
    #[must_use]
    pub fn sarif_level(self) -> &'static str {
        match self {
            Self::High => "error",
            Self::Medium => "warning",
            Self::Low => "note",
        }
    }
}

/// One finding, normalized across the auditors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateFinding {
    pub id: String,
    pub severity: GateSeverity,
    pub file: String,
    pub line: u32,
    pub message: String,
}

impl GateFinding {
    /// A stable identity for baselining.
    ///
    /// Deliberately excludes the line number and the message. Findings move
    /// when unrelated lines above them change, and messages get reworded; a
    /// fingerprint sensitive to either would make a baseline expire on contact
    /// with ordinary editing, which is the failure that makes teams delete
    /// baselines rather than maintain them.
    #[must_use]
    pub fn fingerprint(&self) -> String {
        format!("{}::{}", self.id, self.file)
    }
}

/// Known findings a repository has chosen to accept for now.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Baseline {
    /// Schema marker so a stale file fails loudly rather than matching nothing.
    pub schema: String,
    /// Fingerprints, sorted for a reviewable diff.
    pub findings: BTreeSet<String>,
}

pub const BASELINE_SCHEMA: &str = "audit-gate.baseline.v1";

impl Baseline {
    #[must_use]
    pub fn from_findings(findings: &[GateFinding]) -> Self {
        Self {
            schema: BASELINE_SCHEMA.to_string(),
            findings: findings.iter().map(GateFinding::fingerprint).collect(),
        }
    }

    /// Read a baseline, rejecting one written by a different schema.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("read baseline {}", path.display()))?;
        let parsed: Self = serde_json::from_str(&text)
            .with_context(|| format!("parse baseline {}", path.display()))?;
        anyhow::ensure!(
            parsed.schema == BASELINE_SCHEMA,
            "baseline {} has schema {:?}, expected {BASELINE_SCHEMA}",
            path.display(),
            parsed.schema
        );
        Ok(parsed)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        let text = serde_json::to_string_pretty(self)? + "\n";
        std::fs::write(path, text).with_context(|| format!("write {}", path.display()))
    }

    #[must_use]
    pub fn contains(&self, finding: &GateFinding) -> bool {
        self.findings.contains(&finding.fingerprint())
    }
}

/// Match a path against a glob supporting `*`, `**` and `?`.
///
/// A small matcher rather than a dependency: the patterns these gates need are
/// ordinary path globs, and the auditors have no other use for a glob engine.
/// `*` does not cross a `/`; `**` does.
#[must_use]
pub fn glob_matches(pattern: &str, path: &str) -> bool {
    // A bare pattern with no separator matches any path component, so
    // `--exclude node_modules` behaves the way people expect.
    if !pattern.contains('/') && !pattern.contains('*') && !pattern.contains('?') {
        return path.split('/').any(|part| part == pattern);
    }
    matches_from(pattern.as_bytes(), path.as_bytes())
}

fn matches_from(pattern: &[u8], path: &[u8]) -> bool {
    if pattern.is_empty() {
        return path.is_empty();
    }
    // `**` consumes any number of characters, separators included.
    if pattern.starts_with(b"**") {
        let rest = &pattern[2..];
        let rest = rest.strip_prefix(b"/").unwrap_or(rest);
        if rest.is_empty() {
            return true;
        }
        for split in 0..=path.len() {
            if matches_from(rest, &path[split..]) {
                return true;
            }
        }
        return false;
    }
    match pattern[0] {
        b'*' => {
            let rest = &pattern[1..];
            // `*` stops at a separator.
            let limit = path.iter().position(|&c| c == b'/').unwrap_or(path.len());
            (0..=limit).any(|split| matches_from(rest, &path[split..]))
        }
        b'?' => !path.is_empty() && path[0] != b'/' && matches_from(&pattern[1..], &path[1..]),
        literal => {
            !path.is_empty() && path[0] == literal && matches_from(&pattern[1..], &path[1..])
        }
    }
}

/// Whether a path matches any exclusion pattern.
#[must_use]
pub fn is_excluded(path: &str, patterns: &[String]) -> bool {
    let normalized = path.replace('\\', "/");
    patterns
        .iter()
        .any(|pattern| glob_matches(pattern, &normalized))
}

/// Render findings as SARIF 2.1.0.
///
/// `results` carry the rule id, so GitHub groups annotations by rule, and the
/// `partialFingerprints` field lets it track a finding across moves.
#[must_use]
pub fn to_sarif(tool: &str, version: &str, findings: &[GateFinding]) -> serde_json::Value {
    let rules: BTreeSet<&str> = findings.iter().map(|f| f.id.as_str()).collect();
    serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": tool,
                    "version": version,
                    "informationUri": "https://github.com/BjornMelin/dev-skills",
                    "rules": rules.iter().map(|id| serde_json::json!({
                        "id": id,
                        "shortDescription": { "text": *id },
                    })).collect::<Vec<_>>(),
                }
            },
            "results": findings.iter().map(|f| serde_json::json!({
                "ruleId": f.id,
                "level": f.severity.sarif_level(),
                "message": { "text": f.message },
                "partialFingerprints": { "auditGate/v1": f.fingerprint() },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": f.file },
                        // SARIF lines are 1-based and a 0 is invalid.
                        "region": { "startLine": f.line.max(1) },
                    }
                }],
            })).collect::<Vec<_>>(),
        }],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(id: &str, file: &str, line: u32) -> GateFinding {
        GateFinding {
            id: id.to_string(),
            severity: GateSeverity::Medium,
            file: file.to_string(),
            line,
            message: "message".to_string(),
        }
    }

    #[test]
    fn bare_pattern_matches_any_path_component() {
        assert!(glob_matches("node_modules", "a/node_modules/b.ts"));
        assert!(glob_matches("node_modules", "node_modules/b.ts"));
        assert!(!glob_matches("node_modules", "src/node_modules_helper.ts"));
    }

    #[test]
    fn single_star_does_not_cross_a_separator() {
        assert!(glob_matches("src/*.ts", "src/a.ts"));
        assert!(!glob_matches("src/*.ts", "src/nested/a.ts"));
    }

    #[test]
    fn double_star_crosses_separators() {
        assert!(glob_matches("src/**/*.ts", "src/a/b/c.ts"));
        assert!(glob_matches("**/vendor/**", "apps/web/vendor/x/y.js"));
        assert!(!glob_matches("src/**/*.ts", "lib/a.ts"));
    }

    #[test]
    fn question_mark_matches_one_non_separator_character() {
        assert!(glob_matches("a?.ts", "ab.ts"));
        assert!(!glob_matches("a?.ts", "a/.ts"));
    }

    #[test]
    fn exclusion_normalizes_windows_separators() {
        let patterns = vec!["**/vendor/**".to_string()];
        assert!(is_excluded(r"apps\web\vendor\x.js", &patterns));
    }

    #[test]
    fn fingerprints_survive_a_line_move_but_not_a_file_move() {
        let a = finding("rule.one", "src/a.ts", 10);
        let moved_down = finding("rule.one", "src/a.ts", 90);
        let other_file = finding("rule.one", "src/b.ts", 10);
        assert_eq!(a.fingerprint(), moved_down.fingerprint());
        assert_ne!(a.fingerprint(), other_file.fingerprint());
    }

    #[test]
    fn baseline_round_trips_and_rejects_a_foreign_schema() {
        let dir = std::env::temp_dir().join(format!("audit-gate-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("baseline.json");

        let baseline = Baseline::from_findings(&[finding("rule.one", "src/a.ts", 3)]);
        baseline.save(&path).unwrap();
        let loaded = Baseline::load(&path).unwrap();
        assert!(loaded.contains(&finding("rule.one", "src/a.ts", 3)));
        assert!(!loaded.contains(&finding("rule.two", "src/a.ts", 3)));

        std::fs::write(&path, r#"{"schema":"something.else","findings":[]}"#).unwrap();
        assert!(
            Baseline::load(&path).is_err(),
            "a foreign schema must fail loudly rather than silently baselining nothing"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn sarif_uses_valid_levels_and_never_a_zero_line() {
        let findings = vec![
            GateFinding {
                severity: GateSeverity::High,
                ..finding("rule.one", "src/a.ts", 0)
            },
            GateFinding {
                severity: GateSeverity::Low,
                ..finding("rule.two", "src/b.ts", 7)
            },
        ];
        let sarif = to_sarif("gsap-audit", "0.1.0", &findings);
        let results = sarif["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "note");
        assert_eq!(
            results[0]["locations"][0]["physicalLocation"]["region"]["startLine"], 1,
            "SARIF rejects line 0"
        );
        assert_eq!(
            sarif["runs"][0]["tool"]["driver"]["rules"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
    }
}
