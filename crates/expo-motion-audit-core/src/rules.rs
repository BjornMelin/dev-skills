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
    pub const REANIMATED_CORE_LAYOUT_PROP_ANIMATION: &str = "reanimated-core.layout-prop-animation";
    pub const REANIMATED_CORE_SHARED_VALUE_REASSIGN: &str = "reanimated-core.shared-value-reassign";
    pub const WORKLETS_THREADING_DEPRECATED_RUN_ON: &str = "worklets-threading.deprecated-run-on";
    pub const WORKLETS_THREADING_VALUE_ACCESS_ON_JS: &str = "worklets-threading.value-access-on-js";
    pub const WORKLETS_THREADING_BRIDGE_IN_HOT_PATH: &str = "worklets-threading.bridge-in-hot-path";
    pub const WORKLETS_THREADING_MISSING_WORKLET: &str = "worklets-threading.missing-worklet";
    pub const LAYOUT_INFINITE_REPEAT_NO_REDUCED_MOTION: &str =
        "layout.infinite-repeat-no-reduced-motion";
    pub const ACCESSIBILITY_MISSING_REDUCED_MOTION: &str = "accessibility.missing-reduced-motion";
    pub const LIFECYCLE_MISSING_CANCEL_ANIMATION: &str = "lifecycle.missing-cancel-animation";
    pub const CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST: &str =
        "config.worklets-plugin-missing-or-not-last";
    pub const CONFIG_DEPRECATED_REANIMATED_PLUGIN: &str = "config.deprecated-reanimated-plugin";
    pub const CONFIG_NEW_ARCH_DISABLED: &str = "config.new-arch-disabled";
    pub const CONFIG_UNABLE_TO_ANALYZE: &str = "config.unable-to-analyze";
}

/// The full, ordered rule catalog. Order here drives `doctor` output order.
pub const CATALOG: &[RuleDescriptor] = &[
    RuleDescriptor {
        id: ids::REANIMATED_CORE_LAYOUT_PROP_ANIMATION,
        category: Category::ReanimatedCore,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "Animating layout props (width/height/top/left/margin) in an animated style; prefer transforms.",
    },
    RuleDescriptor {
        id: ids::REANIMATED_CORE_SHARED_VALUE_REASSIGN,
        category: Category::ReanimatedCore,
        severity: Severity::High,
        confidence: Confidence::High,
        summary: "Reassigning a useSharedValue binding directly (`sv = x`) instead of `sv.value = x`.",
    },
    RuleDescriptor {
        id: ids::WORKLETS_THREADING_DEPRECATED_RUN_ON,
        category: Category::WorkletsThreading,
        severity: Severity::High,
        confidence: Confidence::High,
        summary: "Use of runOnJS/runOnUI (deprecated in Reanimated 4); use scheduleOnRN/scheduleOnUI.",
    },
    RuleDescriptor {
        id: ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS,
        category: Category::WorkletsThreading,
        severity: Severity::High,
        confidence: Confidence::Medium,
        summary: "Reading/writing a shared value's `.value` on the JS thread (module scope or render).",
    },
    RuleDescriptor {
        id: ids::WORKLETS_THREADING_BRIDGE_IN_HOT_PATH,
        category: Category::WorkletsThreading,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "scheduleOnRN/runOnJS called inside a gesture onUpdate/onChange or per-frame callback.",
    },
    RuleDescriptor {
        id: ids::WORKLETS_THREADING_MISSING_WORKLET,
        category: Category::WorkletsThreading,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "Extracted named function passed to an animated hook/gesture lacks a 'worklet' directive.",
    },
    RuleDescriptor {
        id: ids::LAYOUT_INFINITE_REPEAT_NO_REDUCED_MOTION,
        category: Category::Layout,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "withRepeat(anim, -1, ...) in a file with no useReducedMotion/ReduceMotion reference.",
    },
    RuleDescriptor {
        id: ids::ACCESSIBILITY_MISSING_REDUCED_MOTION,
        category: Category::Accessibility,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "File animates with Reanimated but never references reduced-motion APIs.",
    },
    RuleDescriptor {
        id: ids::LIFECYCLE_MISSING_CANCEL_ANIMATION,
        category: Category::Lifecycle,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "A shared value is animated with with*(...) but the file never references cancelAnimation.",
    },
    RuleDescriptor {
        id: ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST,
        category: Category::Config,
        severity: Severity::High,
        confidence: Confidence::High,
        summary: "babel.config.js: react-native-worklets/plugin is absent or not the last plugin.",
    },
    RuleDescriptor {
        id: ids::CONFIG_DEPRECATED_REANIMATED_PLUGIN,
        category: Category::Config,
        severity: Severity::High,
        confidence: Confidence::High,
        summary: "babel.config.js uses the old react-native-reanimated/plugin instead of the worklets plugin.",
    },
    RuleDescriptor {
        id: ids::CONFIG_NEW_ARCH_DISABLED,
        category: Category::Config,
        severity: Severity::Medium,
        confidence: Confidence::Medium,
        summary: "app config disables/omits newArchEnabled while the project uses Reanimated 4.",
    },
    RuleDescriptor {
        id: ids::CONFIG_UNABLE_TO_ANALYZE,
        category: Category::Config,
        severity: Severity::Low,
        confidence: Confidence::Low,
        summary: "A config file is too dynamic to analyze statically (informational).",
    },
];

/// Look up a descriptor by id.
#[must_use]
pub fn descriptor(id: &str) -> Option<&'static RuleDescriptor> {
    CATALOG.iter().find(|rule| rule.id == id)
}
