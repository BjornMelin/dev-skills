//! Integration tests for the expo-motion-audit rule engine.
//!
//! Each test parses an inline source snippet (TSX/TS/JS/JSON) and asserts that
//! the expected rule id does (or does not) fire. Snippets are deliberately
//! small so the assertion pins one behavior at a time.

use std::collections::BTreeSet;

use expo_motion_audit_core::config::{analyze_app_config, analyze_babel_config};
use expo_motion_audit_core::rules::ids;
use expo_motion_audit_core::scan::{ScanOptions, scan_root};
use expo_motion_audit_core::source::source_type_for_extension;
use expo_motion_audit_core::types::Finding;
use expo_motion_audit_core::{TOOL_NAME, TOOL_VERSION, analyze_source, format_json};

/// Analyze a source snippet under a given file path + extension.
fn analyze(path: &str, ext: &str, source: &str) -> Vec<Finding> {
    analyze_source(path, source, source_type_for_extension(ext))
}

/// Whether any finding has the given id.
fn fired(findings: &[Finding], id: &str) -> bool {
    findings.iter().any(|finding| finding.id == id)
}

// ---------------------------------------------------------------------------
// Rule 1: reanimated-core.layout-prop-animation
// ---------------------------------------------------------------------------

#[test]
fn rule_layout_prop_animation_fires_and_clean_does_not() {
    let bad = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const style = useAnimatedStyle(() => ({ width: w.value, height: 10 }));"#,
    );
    assert!(fired(&bad, ids::REANIMATED_CORE_LAYOUT_PROP_ANIMATION));

    let bad_block = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const style = useAnimatedStyle(() => { return { marginTop: m.value }; });"#,
    );
    assert!(fired(
        &bad_block,
        ids::REANIMATED_CORE_LAYOUT_PROP_ANIMATION
    ));

    let clean = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const style = useAnimatedStyle(() => ({ transform: [{ translateX: x.value }] }));"#,
    );
    assert!(!fired(&clean, ids::REANIMATED_CORE_LAYOUT_PROP_ANIMATION));
}

// ---------------------------------------------------------------------------
// Rule 2: reanimated-core.shared-value-reassign (semantic)
// ---------------------------------------------------------------------------

#[test]
fn rule_shared_value_reassign_fires_and_value_write_does_not() {
    let bad = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  sv = 5;
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REANIMATED_CORE_SHARED_VALUE_REASSIGN));

    let clean = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  sv.value = 5;
  return null;
}"#,
    );
    assert!(!fired(&clean, ids::REANIMATED_CORE_SHARED_VALUE_REASSIGN));

    // A non-shared-value binding reassignment must not fire.
    let unrelated = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  let count = 0;
  count = 5;
  return null;
}"#,
    );
    assert!(!fired(
        &unrelated,
        ids::REANIMATED_CORE_SHARED_VALUE_REASSIGN
    ));
}

// ---------------------------------------------------------------------------
// Rule 3: worklets-threading.deprecated-run-on
// ---------------------------------------------------------------------------

#[test]
fn rule_deprecated_run_on_fires_and_schedule_does_not() {
    let run_on_js = analyze(
        "src/Box.tsx",
        "tsx",
        r#"import { runOnJS } from "react-native-reanimated";
const fn = () => runOnJS(setText)("done");"#,
    );
    assert!(fired(&run_on_js, ids::WORKLETS_THREADING_DEPRECATED_RUN_ON));

    let run_on_ui = analyze(
        "src/Box.tsx",
        "tsx",
        r#"import { runOnUI } from "react-native-reanimated";
const fn = () => runOnUI(work)();"#,
    );
    assert!(fired(&run_on_ui, ids::WORKLETS_THREADING_DEPRECATED_RUN_ON));

    let clean = analyze(
        "src/Box.tsx",
        "tsx",
        r#"import { scheduleOnRN } from "react-native-worklets";
const fn = () => scheduleOnRN(setText, "done");"#,
    );
    assert!(!fired(&clean, ids::WORKLETS_THREADING_DEPRECATED_RUN_ON));
}

// ---------------------------------------------------------------------------
// Rule 4: worklets-threading.value-access-on-js (semantic)
// ---------------------------------------------------------------------------

#[test]
fn rule_value_access_on_js_fires_at_render_scope() {
    // Reading sv.value during render (not in a worklet/effect) -> fires.
    let bad = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  const current = sv.value;
  return null;
}"#,
    );
    assert!(fired(&bad, ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS));
}

#[test]
fn rule_value_access_inside_worklet_does_not_fire() {
    // Reading sv.value inside an animated hook arrow (auto-workletized) -> clean.
    let clean_hook = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  const style = useAnimatedStyle(() => ({ opacity: sv.value }));
  return null;
}"#,
    );
    assert!(!fired(
        &clean_hook,
        ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS
    ));

    // Reading sv.value inside an explicit 'worklet' function -> clean.
    let clean_worklet = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  const compute = () => {
    'worklet';
    return sv.value + 1;
  };
  return null;
}"#,
    );
    assert!(!fired(
        &clean_worklet,
        ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS
    ));

    // Reading sv.value inside useEffect -> clean (effect runs off render path).
    let clean_effect = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  useEffect(() => {
    console.log(sv.value);
  }, []);
  return null;
}"#,
    );
    assert!(!fired(
        &clean_effect,
        ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS
    ));
}

#[test]
fn rule_value_access_in_jsx_event_handler_does_not_fire() {
    // Writing sv.value inside an onPress handler runs at event time on the JS
    // thread, which is fine -> must not fire.
    let clean = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  return <Pressable onPress={() => { sv.value = withTiming(1); }} />;
}"#,
    );
    assert!(!fired(&clean, ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS));
}

#[test]
fn rule_value_access_in_jsx_render_expression_still_fires() {
    // Reading sv.value directly in a style prop (no intervening function) runs
    // during render on the JS thread -> must still fire.
    let bad = analyze(
        "src/Box.tsx",
        "tsx",
        r#"function C() {
  const sv = useSharedValue(0);
  return <View style={{ width: sv.value }} />;
}"#,
    );
    assert!(fired(&bad, ids::WORKLETS_THREADING_VALUE_ACCESS_ON_JS));
}

// ---------------------------------------------------------------------------
// Rule 5: worklets-threading.bridge-in-hot-path
// ---------------------------------------------------------------------------

#[test]
fn rule_bridge_in_hot_path_fires_and_outside_does_not() {
    let bad = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const g = Gesture.Pan().onUpdate((e) => {
  scheduleOnRN(setX, e.translationX);
});"#,
    );
    assert!(fired(&bad, ids::WORKLETS_THREADING_BRIDGE_IN_HOT_PATH));

    // Same bridge call in onEnd (not a per-frame hot path) -> does not fire.
    let clean = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const g = Gesture.Pan().onEnd((e) => {
  scheduleOnRN(setX, e.translationX);
});"#,
    );
    assert!(!fired(&clean, ids::WORKLETS_THREADING_BRIDGE_IN_HOT_PATH));
}

// ---------------------------------------------------------------------------
// Rule 6: worklets-threading.missing-worklet
// ---------------------------------------------------------------------------

#[test]
fn rule_missing_worklet_fires_for_named_function_and_arrow_is_clean() {
    let bad = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const d = useDerivedValue(function compute() {
  return sv.value * 2;
});"#,
    );
    assert!(fired(&bad, ids::WORKLETS_THREADING_MISSING_WORKLET));

    // Inline arrow -> auto-workletized by babel plugin -> does not fire.
    let clean_arrow = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const d = useDerivedValue(() => sv.value * 2);"#,
    );
    assert!(!fired(
        &clean_arrow,
        ids::WORKLETS_THREADING_MISSING_WORKLET
    ));

    // Named function WITH a 'worklet' directive -> does not fire.
    let clean_worklet = analyze(
        "src/Box.tsx",
        "tsx",
        r#"const d = useDerivedValue(function compute() {
  'worklet';
  return sv.value * 2;
});"#,
    );
    assert!(!fired(
        &clean_worklet,
        ids::WORKLETS_THREADING_MISSING_WORKLET
    ));
}

// ---------------------------------------------------------------------------
// Rule 7: layout.infinite-repeat-no-reduced-motion
// ---------------------------------------------------------------------------

#[test]
fn rule_infinite_repeat_fires_and_guarded_does_not() {
    let bad = analyze(
        "src/Spinner.tsx",
        "tsx",
        r#"sv.value = withRepeat(withTiming(1), -1, true);"#,
    );
    assert!(fired(&bad, ids::LAYOUT_INFINITE_REPEAT_NO_REDUCED_MOTION));

    // Same file but with a reduced-motion reference present -> does not fire.
    let clean = analyze(
        "src/Spinner.tsx",
        "tsx",
        r#"import { useReducedMotion } from "react-native-reanimated";
const reduced = useReducedMotion();
sv.value = withRepeat(withTiming(1), -1, true);"#,
    );
    assert!(!fired(
        &clean,
        ids::LAYOUT_INFINITE_REPEAT_NO_REDUCED_MOTION
    ));

    // Finite repeat -> does not fire.
    let finite = analyze(
        "src/Spinner.tsx",
        "tsx",
        r#"sv.value = withRepeat(withTiming(1), 3, true);"#,
    );
    assert!(!fired(
        &finite,
        ids::LAYOUT_INFINITE_REPEAT_NO_REDUCED_MOTION
    ));
}

// ---------------------------------------------------------------------------
// Rule 8: accessibility.missing-reduced-motion (file-scoped heuristic)
// ---------------------------------------------------------------------------

#[test]
fn rule_missing_reduced_motion_fires_and_referenced_does_not() {
    let bad = analyze(
        "src/Fade.tsx",
        "tsx",
        r#"function C() {
  sv.value = withTiming(1);
  return <Animated.View entering={FadeIn} />;
}"#,
    );
    assert!(fired(&bad, ids::ACCESSIBILITY_MISSING_REDUCED_MOTION));

    let clean = analyze(
        "src/Fade.tsx",
        "tsx",
        r#"import { useReducedMotion } from "react-native-reanimated";
function C() {
  const reduced = useReducedMotion();
  sv.value = withTiming(1);
  return null;
}"#,
    );
    assert!(!fired(&clean, ids::ACCESSIBILITY_MISSING_REDUCED_MOTION));

    // A file with no animation at all -> does not fire.
    let no_animation = analyze("src/Plain.tsx", "tsx", r#"const value = 1;"#);
    assert!(!fired(
        &no_animation,
        ids::ACCESSIBILITY_MISSING_REDUCED_MOTION
    ));
}

// ---------------------------------------------------------------------------
// Rule 9: lifecycle.missing-cancel-animation (heuristic)
// ---------------------------------------------------------------------------

#[test]
fn rule_missing_cancel_animation_fires_and_present_does_not() {
    let bad = analyze(
        "src/Pulse.tsx",
        "tsx",
        r#"import { useReducedMotion } from "react-native-reanimated";
function C() {
  const reduced = useReducedMotion();
  sv.value = withTiming(1);
  return null;
}"#,
    );
    assert!(fired(&bad, ids::LIFECYCLE_MISSING_CANCEL_ANIMATION));

    let clean = analyze(
        "src/Pulse.tsx",
        "tsx",
        r#"import { cancelAnimation, useReducedMotion } from "react-native-reanimated";
function C() {
  const reduced = useReducedMotion();
  sv.value = withTiming(1);
  useEffect(() => () => cancelAnimation(sv), []);
  return null;
}"#,
    );
    assert!(!fired(&clean, ids::LIFECYCLE_MISSING_CANCEL_ANIMATION));
}

// ---------------------------------------------------------------------------
// Clean component: zero findings.
// ---------------------------------------------------------------------------

#[test]
fn clean_reanimated_component_has_zero_findings() {
    let clean = analyze(
        "src/Clean.tsx",
        "tsx",
        r#"import {
  useSharedValue,
  useAnimatedStyle,
  useReducedMotion,
  cancelAnimation,
  withTiming,
} from "react-native-reanimated";
import { useEffect } from "react";

export default function Clean() {
  const opacity = useSharedValue(0);
  const reduced = useReducedMotion();

  const style = useAnimatedStyle(() => ({ opacity: opacity.value }));

  useEffect(() => {
    opacity.value = withTiming(reduced ? 1 : 1);
    return () => cancelAnimation(opacity);
  }, []);

  return null;
}"#,
    );
    assert_eq!(clean, Vec::new(), "expected zero findings, got: {clean:#?}");
}

// ---------------------------------------------------------------------------
// Rule 10 & 11: babel.config.js worklets plugin placement + deprecated plugin.
// ---------------------------------------------------------------------------

#[test]
fn babel_config_worklets_plugin_last_is_clean() {
    let clean = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = function (api) {
  api.cache(true);
  return {
    presets: ["babel-preset-expo"],
    plugins: ["react-native-worklets/plugin"],
  };
};"#,
    );
    assert!(!fired(
        &clean,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
    assert!(!fired(&clean, ids::CONFIG_DEPRECATED_REANIMATED_PLUGIN));
}

#[test]
fn babel_config_worklets_plugin_missing_fires() {
    // No babel-preset-expo to supply the worklets plugin -> missing fires.
    let missing = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = {
  plugins: ["some-other-plugin"],
};"#,
    );
    assert!(fired(
        &missing,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_worklets_plugin_not_last_fires() {
    let not_last = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = {
  plugins: ["react-native-worklets/plugin", "another-plugin"],
};"#,
    );
    assert!(fired(
        &not_last,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_deprecated_reanimated_plugin_fires() {
    let deprecated = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = {
  plugins: ["react-native-reanimated/plugin"],
};"#,
    );
    assert!(fired(&deprecated, ids::CONFIG_DEPRECATED_REANIMATED_PLUGIN));
    // The old plugin is also "missing the worklets plugin".
    assert!(fired(
        &deprecated,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_dynamic_export_is_informational() {
    let dynamic = analyze_babel_config("babel.config.js", r#"module.exports = buildConfig();"#);
    assert!(fired(&dynamic, ids::CONFIG_UNABLE_TO_ANALYZE));
}

#[test]
fn babel_config_plugin_tuple_form_resolves() {
    // The worklets plugin written in tuple form `["plugin", options]` as the
    // last entry is clean.
    let tuple = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = {
  plugins: [["react-native-worklets/plugin", { processNestedWorklets: true }]],
};"#,
    );
    assert!(!fired(
        &tuple,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_dynamic_plugins_property_is_informational() {
    // `plugins` exists but is a variable, not an inline array -> low advisory,
    // not the high missing finding.
    let dynamic = analyze_babel_config(
        "babel.config.js",
        r#"const plugins = ["react-native-worklets/plugin"];
module.exports = { plugins };"#,
    );
    assert!(fired(&dynamic, ids::CONFIG_UNABLE_TO_ANALYZE));
    assert!(!fired(
        &dynamic,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_expo_preset_only_is_clean() {
    // babel-preset-expo auto-includes the worklets plugin; no plugins array is
    // needed and the missing finding must be suppressed.
    let preset_only = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = { presets: ["babel-preset-expo"] };"#,
    );
    assert!(!fired(
        &preset_only,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_expo_preset_with_other_plugins_is_clean() {
    // Expo preset present, plugins array has unrelated plugins but not worklets
    // -> still suppressed (the preset supplies it).
    let preset_other = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = {
  presets: ["babel-preset-expo"],
  plugins: ["some-other-plugin"],
};"#,
    );
    assert!(!fired(
        &preset_other,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_expo_preset_worklets_not_last_still_fires() {
    // Even with the expo preset, an explicit worklets plugin that is not last is
    // an ordering error and must still fire.
    let not_last = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = {
  presets: ["babel-preset-expo"],
  plugins: ["react-native-worklets/plugin", "x"],
};"#,
    );
    assert!(fired(
        &not_last,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_explicit_worklets_no_preset_is_clean() {
    // Explicit worklets plugin (last) with no expo preset is correct.
    let explicit = analyze_babel_config(
        "babel.config.js",
        r#"module.exports = { plugins: ["react-native-worklets/plugin"] };"#,
    );
    assert!(!fired(
        &explicit,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

#[test]
fn babel_config_no_presets_no_plugins_still_fires_missing() {
    // No presets and no plugins at all -> missing fires.
    let empty = analyze_babel_config("babel.config.js", r#"module.exports = {};"#);
    assert!(fired(
        &empty,
        ids::CONFIG_WORKLETS_PLUGIN_MISSING_OR_NOT_LAST
    ));
}

// ---------------------------------------------------------------------------
// Rule 12: app.json newArchEnabled.
// ---------------------------------------------------------------------------

#[test]
fn app_config_new_arch_enabled_is_clean() {
    let clean = analyze_app_config(
        "app.json",
        r#"{ "expo": { "name": "demo", "newArchEnabled": true } }"#,
        true,
    );
    assert!(!fired(&clean, ids::CONFIG_NEW_ARCH_DISABLED));
}

#[test]
fn app_config_new_arch_disabled_fires_when_reanimated_used() {
    let disabled = analyze_app_config(
        "app.json",
        r#"{ "expo": { "name": "demo", "newArchEnabled": false } }"#,
        true,
    );
    assert!(fired(&disabled, ids::CONFIG_NEW_ARCH_DISABLED));

    let absent = analyze_app_config("app.json", r#"{ "expo": { "name": "demo" } }"#, true);
    assert!(fired(&absent, ids::CONFIG_NEW_ARCH_DISABLED));
}

#[test]
fn app_config_new_arch_absent_clean_when_reanimated_unused() {
    // Same disabled config, but the project does not use Reanimated -> clean.
    let clean = analyze_app_config(
        "app.json",
        r#"{ "expo": { "name": "demo", "newArchEnabled": false } }"#,
        false,
    );
    assert!(!fired(&clean, ids::CONFIG_NEW_ARCH_DISABLED));
}

// ---------------------------------------------------------------------------
// Output shape sanity.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Fix 4: dynamic app.config.js routes to the app-config analyzer.
// ---------------------------------------------------------------------------

#[test]
fn scan_dynamic_app_config_js_emits_unable_to_analyze() {
    // Create a unique scratch directory under the system temp dir.
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "expo-motion-audit-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::create_dir_all(&dir).expect("create temp scan dir");

    let config_path = dir.join("app.config.js");
    std::fs::write(
        &config_path,
        r#"module.exports = () => ({ expo: { name: "demo" } });"#,
    )
    .expect("write app.config.js");

    let options = ScanOptions::new(dir.clone(), BTreeSet::new(), 1000);
    let outcome = scan_root(&options).expect("scan succeeds");

    // Clean up before asserting so a failed assertion still removes the dir.
    let _ = std::fs::remove_dir_all(&dir);

    assert!(
        fired(&outcome.findings, ids::CONFIG_UNABLE_TO_ANALYZE),
        "expected config.unable-to-analyze for dynamic app.config.js, got: {:#?}",
        outcome.findings
    );
}

#[test]
fn json_output_has_stable_shape() {
    let findings = analyze(
        "src/Box.tsx",
        "tsx",
        r#"import { runOnJS } from "react-native-reanimated";
const fn = () => runOnJS(setText)("done");"#,
    );
    let value = format_json(TOOL_NAME, TOOL_VERSION, &findings);
    assert_eq!(value["tool"], TOOL_NAME);
    assert!(value["findings"].is_array());
    assert!(value["summary"]["total"].as_u64().unwrap() >= 1);
}
