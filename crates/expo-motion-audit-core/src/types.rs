//! Shared types for expo-motion-audit findings, severities, and the rule
//! catalog.
//!
//! The oxc AST is arena-allocated and not serde-serializable, so these are the
//! crate's own owned, serializable output types.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity of a finding. Drives the process exit code contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    /// Lowercase stable string used in output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// How confident the rule is that the finding is a true positive.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
    /// Lowercase stable string used in output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Logical grouping for a rule. Used by the `--categories` filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    ReanimatedCore,
    WorkletsThreading,
    Gestures,
    Layout,
    Accessibility,
    Lifecycle,
    Config,
}

impl Category {
    /// Lowercase, kebab-case stable string used in output and the
    /// `--categories` CSV.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Category::ReanimatedCore => "reanimated-core",
            Category::WorkletsThreading => "worklets-threading",
            Category::Gestures => "gestures",
            Category::Layout => "layout",
            Category::Accessibility => "accessibility",
            Category::Lifecycle => "lifecycle",
            Category::Config => "config",
        }
    }

    /// Every category, used as the default filter set.
    #[must_use]
    pub fn all() -> &'static [Category] {
        &[
            Category::ReanimatedCore,
            Category::WorkletsThreading,
            Category::Gestures,
            Category::Layout,
            Category::Accessibility,
            Category::Lifecycle,
            Category::Config,
        ]
    }

    /// Parse a single category token from the `--categories` CSV.
    #[must_use]
    pub fn parse(token: &str) -> Option<Category> {
        match token.trim().to_ascii_lowercase().as_str() {
            "reanimated-core" => Some(Category::ReanimatedCore),
            "worklets-threading" => Some(Category::WorkletsThreading),
            "gestures" => Some(Category::Gestures),
            "layout" => Some(Category::Layout),
            "accessibility" => Some(Category::Accessibility),
            "lifecycle" => Some(Category::Lifecycle),
            "config" => Some(Category::Config),
            _ => None,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Static descriptor for a rule. The catalog is built from these so that
/// adding a rule later only requires registering one descriptor plus the
/// detection logic that emits its findings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuleDescriptor {
    /// Stable rule identifier, for example `worklets-threading.deprecated-run-on`.
    pub id: &'static str,
    /// Category used for filtering and grouping.
    pub category: Category,
    /// Default severity for findings emitted by this rule.
    pub severity: Severity,
    /// Default confidence for findings emitted by this rule.
    pub confidence: Confidence,
    /// One-line human summary of what the rule checks.
    pub summary: &'static str,
}

/// A single reported anti-pattern occurrence.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable rule id, matching a [`RuleDescriptor::id`].
    pub id: String,
    /// Category of the originating rule.
    pub category: Category,
    /// Severity of this occurrence.
    pub severity: Severity,
    /// Confidence of this occurrence.
    pub confidence: Confidence,
    /// Source file the finding was detected in, relative to the scan root.
    pub file: String,
    /// 1-based line number.
    pub line: u32,
    /// 1-based column number.
    pub column: u32,
    /// Human-readable description of the problem.
    pub message: String,
    /// Short remediation suggestion.
    pub suggestion: String,
}
