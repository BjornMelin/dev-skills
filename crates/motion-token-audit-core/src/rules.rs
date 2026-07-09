//! The rule catalog.

use crate::types::{Category, Confidence, RuleDescriptor, Severity};

pub mod ids {
    pub const SSOT_NO_TOKEN_MODULE: &str = "ssot.no-token-module";
    pub const CSS_DURATION_LITERAL: &str = "tokens-css.duration-literal";
    pub const CSS_EASING_LITERAL: &str = "tokens-css.easing-literal";
    pub const REANIMATED_DURATION_LITERAL: &str = "tokens-reanimated.duration-literal";
    pub const REANIMATED_EASING_LITERAL: &str = "tokens-reanimated.easing-literal";
    pub const REANIMATED_SPRING_LITERAL: &str = "tokens-reanimated.spring-literal";
    pub const GSAP_DURATION_LITERAL: &str = "tokens-gsap.duration-literal";
    pub const GSAP_EASING_LITERAL: &str = "tokens-gsap.easing-literal";
    pub const REACT_DURATION_LITERAL: &str = "tokens-react.duration-literal";
    pub const REACT_EASING_LITERAL: &str = "tokens-react.easing-literal";
}

pub const CATALOG: &[RuleDescriptor] = &[
    RuleDescriptor {
        id: ids::SSOT_NO_TOKEN_MODULE,
        category: Category::Ssot,
        severity: Severity::Low,
        confidence: Confidence::High,
        summary: "No motion token module or CSS custom-property SSOT was found.",
    },
    RuleDescriptor {
        id: ids::CSS_DURATION_LITERAL,
        category: Category::TokensCss,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "CSS transition or animation uses a hardcoded duration literal.",
    },
    RuleDescriptor {
        id: ids::CSS_EASING_LITERAL,
        category: Category::TokensCss,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "CSS uses a hardcoded cubic-bezier easing literal.",
    },
    RuleDescriptor {
        id: ids::REANIMATED_DURATION_LITERAL,
        category: Category::TokensReanimated,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "Reanimated withTiming/withDelay uses a hardcoded duration literal.",
    },
    RuleDescriptor {
        id: ids::REANIMATED_EASING_LITERAL,
        category: Category::TokensReanimated,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "Reanimated Easing.bezier uses a hardcoded easing tuple.",
    },
    RuleDescriptor {
        id: ids::REANIMATED_SPRING_LITERAL,
        category: Category::TokensReanimated,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "Reanimated withSpring uses an inline stiffness/damping/mass config.",
    },
    RuleDescriptor {
        id: ids::GSAP_DURATION_LITERAL,
        category: Category::TokensGsap,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "GSAP tween config uses a hardcoded duration literal.",
    },
    RuleDescriptor {
        id: ids::GSAP_EASING_LITERAL,
        category: Category::TokensGsap,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "GSAP tween config uses a hardcoded ease string.",
    },
    RuleDescriptor {
        id: ids::REACT_DURATION_LITERAL,
        category: Category::TokensReact,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "Motion React transition object uses a hardcoded duration literal.",
    },
    RuleDescriptor {
        id: ids::REACT_EASING_LITERAL,
        category: Category::TokensReact,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "Motion React transition object uses a hardcoded ease literal.",
    },
];

#[must_use]
pub fn descriptor(id: &str) -> Option<&'static RuleDescriptor> {
    CATALOG.iter().find(|rule| rule.id == id)
}
