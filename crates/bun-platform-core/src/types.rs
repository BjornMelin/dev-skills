use serde::{Deserialize, Serialize};

pub const VERIFIED_BUN_VERSION: &str = "1.3.14";
pub const BUN_RELEASE_NOTES_URL: &str = "https://bun.com/blog/bun-v1.3.14";
pub const VERCEL_BUN_RUNTIME_URL: &str = "https://vercel.com/docs/functions/runtimes/bun";
pub const REF_BUN_RELEASE_NOTES: &str = "ref-bun-release-notes-latest.md";
pub const REF_BUN_CAPABILITIES: &str = "ref-bun-capabilities-latest.md";
pub const REF_BUN_CLI: &str = "ref-bun-cli-cheatsheet.md";
pub const REF_BUN_BUILTINS: &str = "ref-bun-builtins-cheatsheet.md";
pub const REF_BUN_PM_FALLBACKS: &str = "ref-bun-package-manager-fallbacks.md";
pub const REF_VERCEL_BUN_RUNTIME: &str = "ref-vercel-bun-runtime.md";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warn,
    Info,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
    Md,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    #[default]
    High,
    Medium,
    Low,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FixKind {
    Safe,
    Unsafe,
    Manual,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub category: String,
    pub severity: Severity,
    pub confidence: Confidence,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub why: Option<String>,
    pub suggested_fix: Option<String>,
    pub snippet: Option<String>,
    pub suppression_key: String,
}

/// A safe package.json rewrite plan. `rule_id` is the primary single-rule
/// identifier for older consumers; `rule_ids` is the canonical list to read
/// when one edit satisfies multiple rules.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlannedFix {
    pub rule_id: String,
    pub rule_ids: Vec<String>,
    pub kind: FixKind,
    pub file: String,
    pub description: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityClassification {
    DocsOnly,
    CapabilityPresent,
    MissingRule,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReleaseReference {
    pub file: String,
    pub hash: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityReport {
    pub topic: String,
    pub source: String,
    pub matched: bool,
    pub rules: Vec<String>,
    pub classification: CapabilityClassification,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReleaseSyncReport {
    pub synced_at: String,
    pub verified_bun_version: String,
    pub references: Vec<ReleaseReference>,
    pub capability_map: Vec<CapabilityReport>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReleaseSyncPreview {
    pub checked_at: String,
    pub verified_bun_version: String,
    pub would_update: Vec<String>,
    pub unchanged: Vec<String>,
    pub integrity_ok: bool,
}

impl Severity {
    pub fn rank(self) -> usize {
        match self {
            Self::Error => 3,
            Self::Warn => 2,
            Self::Info => 1,
        }
    }

    pub fn as_upper(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warn => "WARN",
            Self::Info => "INFO",
        }
    }
}
