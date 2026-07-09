//! Shared types for motion-token-audit findings and coverage.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Low,
    Medium,
    High,
}

impl Confidence {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Ssot,
    TokensCss,
    TokensReanimated,
    TokensGsap,
    TokensReact,
    TokensR3f,
}

impl Category {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Ssot => "ssot",
            Category::TokensCss => "tokens-css",
            Category::TokensReanimated => "tokens-reanimated",
            Category::TokensGsap => "tokens-gsap",
            Category::TokensReact => "tokens-react",
            Category::TokensR3f => "tokens-r3f",
        }
    }

    #[must_use]
    pub fn all() -> &'static [Category] {
        &[
            Category::Ssot,
            Category::TokensCss,
            Category::TokensReanimated,
            Category::TokensGsap,
            Category::TokensReact,
            Category::TokensR3f,
        ]
    }

    #[must_use]
    pub fn parse(token: &str) -> Option<Category> {
        match token.trim().to_ascii_lowercase().as_str() {
            "ssot" => Some(Category::Ssot),
            "tokens-css" => Some(Category::TokensCss),
            "tokens-reanimated" => Some(Category::TokensReanimated),
            "tokens-gsap" => Some(Category::TokensGsap),
            "tokens-react" => Some(Category::TokensReact),
            "tokens-r3f" => Some(Category::TokensR3f),
            _ => None,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RuleDescriptor {
    pub id: &'static str,
    pub category: Category,
    pub severity: Severity,
    pub confidence: Confidence,
    pub summary: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub category: Category,
    pub severity: Severity,
    pub confidence: Confidence,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    pub stack: String,
    pub tokenized_references: usize,
    pub hardcoded_literals: usize,
    pub drift: usize,
    pub orphan: usize,
}

impl Coverage {
    #[must_use]
    pub fn new(stack: &str) -> Coverage {
        Coverage {
            stack: stack.to_string(),
            ..Coverage::default()
        }
    }

    #[must_use]
    pub fn percentage(&self) -> u8 {
        let total = self.tokenized_references + self.hardcoded_literals;
        if total == 0 {
            return 100;
        }
        ((self.tokenized_references * 100) / total)
            .try_into()
            .unwrap_or(100)
    }
}
