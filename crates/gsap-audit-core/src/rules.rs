//! The rule catalog.
//!
//! Each rule has a stable descriptor here. The analysis engine in
//! [`crate::analyze`] dispatches over AST nodes and emits findings tagged with
//! one of these ids. Adding a new rule is a matter of adding a descriptor to
//! [`CATALOG`] and the detection logic that references its id.

use crate::types::{Category, Confidence, RuleDescriptor, Severity};

/// Stable rule identifiers, exposed as constants so the engine and tests can
/// reference them without stringly-typed drift.
pub mod ids {
    pub const CORE_GSAP_TRIAL_IMPORT: &str = "core.gsap-trial-import";
    pub const PLUGINS_GSDEVTOOLS_IN_SOURCE: &str = "plugins.gsdevtools-in-source";
    pub const SCROLLTRIGGER_MARKERS_IN_PROD: &str = "scrolltrigger.markers-in-prod";
    pub const SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS: &str =
        "scrolltrigger.scrub-with-toggleactions";
    pub const CORE_GSAP2_SIGNATURE: &str = "core.gsap2-signature";
    pub const PERFORMANCE_LAG_SMOOTHING_DISABLED: &str = "performance.lag-smoothing-disabled";
    pub const CORE_LAYOUT_PROP_ANIMATION: &str = "core.layout-prop-animation";
    pub const PLUGINS_PLUGIN_USED_WITHOUT_REGISTER: &str = "plugins.plugin-used-without-register";
    pub const REACT_USEGSAP_NOT_REGISTERED: &str = "react.usegsap-not-registered";
    pub const REACT_GSAP_IN_SSR: &str = "react.gsap-in-ssr";
    pub const REACT_UNSCOPED_SELECTOR: &str = "react.unscoped-selector";
    pub const REACT_CONTEXT_MISSING_REVERT: &str = "react.context-missing-revert";
}

/// The full, ordered rule catalog. Order here drives `doctor` output order.
pub const CATALOG: &[RuleDescriptor] = &[
    RuleDescriptor {
        id: ids::CORE_GSAP_TRIAL_IMPORT,
        category: Category::Core,
        severity: Severity::High,
        confidence: Confidence::High,
        summary: "Imports from the obsolete `gsap-trial` package; all plugins are free in `gsap`.",
    },
    RuleDescriptor {
        id: ids::PLUGINS_GSDEVTOOLS_IN_SOURCE,
        category: Category::Plugins,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "GSDevTools referenced in non-test source; it is dev-only and must not ship.",
    },
    RuleDescriptor {
        id: ids::SCROLLTRIGGER_MARKERS_IN_PROD,
        category: Category::Scrolltrigger,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "`markers: true` left enabled in a ScrollTrigger config object.",
    },
    RuleDescriptor {
        id: ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS,
        category: Category::Scrolltrigger,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "ScrollTrigger config mixes `scrub` and `toggleActions`, which conflict.",
    },
    RuleDescriptor {
        id: ids::CORE_GSAP2_SIGNATURE,
        category: Category::Core,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "GSAP-2 duration-as-second-argument call signature; use vars.duration instead.",
    },
    RuleDescriptor {
        id: ids::PERFORMANCE_LAG_SMOOTHING_DISABLED,
        category: Category::Performance,
        severity: Severity::Medium,
        confidence: Confidence::High,
        summary: "`gsap.ticker.lagSmoothing` disabled; can worsen jank under load.",
    },
    RuleDescriptor {
        id: ids::CORE_LAYOUT_PROP_ANIMATION,
        category: Category::Core,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "Animating layout properties (top/left/width/...); prefer transforms.",
    },
    RuleDescriptor {
        id: ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER,
        category: Category::Plugins,
        severity: Severity::High,
        confidence: Confidence::Medium,
        summary: "A GSAP plugin is used in the file but never passed to gsap.registerPlugin.",
    },
    RuleDescriptor {
        id: ids::REACT_USEGSAP_NOT_REGISTERED,
        category: Category::React,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "useGSAP imported from @gsap/react but never registered with registerPlugin.",
    },
    RuleDescriptor {
        id: ids::REACT_GSAP_IN_SSR,
        category: Category::React,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "GSAP used in an app/ or pages/ file without a top-of-file \"use client\".",
    },
    RuleDescriptor {
        id: ids::REACT_UNSCOPED_SELECTOR,
        category: Category::React,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "useGSAP/gsap.context uses string selectors without a scope.",
    },
    RuleDescriptor {
        id: ids::REACT_CONTEXT_MISSING_REVERT,
        category: Category::React,
        severity: Severity::High,
        confidence: Confidence::Medium,
        summary: "gsap.context() result is never reverted or returned for cleanup.",
    },
];

/// Look up a descriptor by id.
#[must_use]
pub fn descriptor(id: &str) -> Option<&'static RuleDescriptor> {
    CATALOG.iter().find(|rule| rule.id == id)
}
