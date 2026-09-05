//! Integration tests for the gsap-audit rule engine.
//!
//! Each test parses an inline source snippet and asserts that the expected rule
//! id does (or does not) fire. Snippets are deliberately small so the assertion
//! pins one behavior at a time.

use gsap_audit_core::analyze_source;
use gsap_audit_core::rules::ids;
use gsap_audit_core::source::source_type_for_extension;
use gsap_audit_core::types::{Category, Finding};

/// Analyze a snippet under a given file path + extension.
fn analyze(path: &str, ext: &str, source: &str) -> Vec<Finding> {
    analyze_source(path, source, source_type_for_extension(ext))
}

/// Whether any finding has the given id.
fn fired(findings: &[Finding], id: &str) -> bool {
    findings.iter().any(|finding| finding.id == id)
}

/// Count findings with the given id.
fn count(findings: &[Finding], id: &str) -> usize {
    findings.iter().filter(|finding| finding.id == id).count()
}

#[test]
fn rule_gsap_trial_import_fires_and_clean_does_not() {
    let bad = analyze("src/a.ts", "ts", r#"import { Flip } from "gsap-trial";"#);
    assert!(fired(&bad, ids::CORE_GSAP_TRIAL_IMPORT));

    let bad_subpath = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger } from "gsap-trial/ScrollTrigger";"#,
    );
    assert!(fired(&bad_subpath, ids::CORE_GSAP_TRIAL_IMPORT));

    let clean = analyze("src/a.ts", "ts", r#"import { Flip } from "gsap/Flip";"#);
    assert!(!fired(&clean, ids::CORE_GSAP_TRIAL_IMPORT));
}

#[test]
fn rule_gsdevtools_in_source() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(fired(&bad, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let clean = analyze("src/a.ts", "ts", r"const x = 1; export { x };");
    assert!(!fired(&clean, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let test_file = analyze(
        "src/a.test.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(!fired(&test_file, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let fixture = analyze(
        "src/fixtures/gsdevtools.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(!fired(&fixture, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let motion_path_helper = analyze(
        "src/a.ts",
        "ts",
        r##"import { MotionPathHelper } from "gsap/MotionPathHelper"; MotionPathHelper.create(".dot", { path: "#path" });"##,
    );
    assert!(fired(
        &motion_path_helper,
        ids::PLUGINS_GSDEVTOOLS_IN_SOURCE
    ));

    let motion_path_helper_test = analyze(
        "src/a.test.ts",
        "ts",
        r##"import { MotionPathHelper } from "gsap/MotionPathHelper"; MotionPathHelper.create(".dot", { path: "#path" });"##,
    );
    assert!(!fired(
        &motion_path_helper_test,
        ids::PLUGINS_GSDEVTOOLS_IN_SOURCE
    ));

    let named_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import { GSDevTools as DevTools } from "gsap/GSDevTools"; DevTools.create();"#,
    );
    assert!(fired(&named_alias, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let default_alias = analyze(
        "src/a.ts",
        "ts",
        r##"import Helper from "gsap/MotionPathHelper"; Helper.create(".dot", { path: "#path" });"##,
    );
    assert!(fired(&default_alias, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let namespace_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import * as DevTools from "gsap/GSDevTools"; DevTools.create();"#,
    );
    assert!(fired(&namespace_alias, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));
}

#[test]
fn rule_markers_in_prod() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { scrollTrigger: { trigger: ".box", markers: true } });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { scrollTrigger: { trigger: ".box", markers: false } });"#,
    );
    assert!(!fired(&clean, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
}

#[test]
fn rule_scrub_with_toggleactions_conflict() {
    // Inside a ScrollTrigger.create config -> fires (GSAP context).
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ trigger: ".x", scrub: true, toggleActions: "play none none reverse" });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ trigger: ".x", scrub: true });"#,
    );
    assert!(!fired(&clean, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));

    let batch = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.batch(".card", { scrub: true, toggleActions: "play none none reverse" });"#,
    );
    assert!(fired(&batch, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));
}

#[test]
fn rule_gsap2_signature() {
    let bad = analyze("src/a.ts", "ts", r#"gsap.to(".box", 1.5, { x: 100 });"#);
    assert!(fired(&bad, ids::CORE_GSAP2_SIGNATURE));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { duration: 1.5, x: 100 });"#,
    );
    assert!(!fired(&clean, ids::CORE_GSAP2_SIGNATURE));
}

#[test]
fn rule_gsap2_signature_categorized_as_core() {
    // Fix 7: the GSAP-2 signature rule fires on core gsap.to/from/fromTo and
    // must be categorized as Core (not Timeline) and use the `core.` id, so a
    // `--categories core` filter does not miss it.
    let findings = analyze("src/a.ts", "ts", r#"gsap.to(".box", 1.5, { x: 100 });"#);
    let finding = findings
        .iter()
        .find(|finding| finding.id == ids::CORE_GSAP2_SIGNATURE)
        .expect("gsap2-signature finding");
    assert_eq!(finding.id, "core.gsap2-signature");
    assert_eq!(finding.category, Category::Core);
}

#[test]
fn rule_lag_smoothing_disabled() {
    let bad_zero = analyze("src/a.ts", "ts", r"gsap.ticker.lagSmoothing(0);");
    assert!(fired(&bad_zero, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));

    let bad_false = analyze("src/a.ts", "ts", r"gsap.ticker.lagSmoothing(false);");
    assert!(fired(&bad_false, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));

    let clean = analyze("src/a.ts", "ts", r"gsap.ticker.lagSmoothing(500, 33);");
    assert!(!fired(&clean, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));
}

#[test]
fn rule_layout_prop_animation() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { top: 100, left: 50 });"#,
    );
    assert!(fired(&bad, ids::CORE_LAYOUT_PROP_ANIMATION));

    let clean = analyze("src/a.ts", "ts", r#"gsap.to(".box", { x: 100, y: 50 });"#);
    assert!(!fired(&clean, ids::CORE_LAYOUT_PROP_ANIMATION));
}

#[test]
fn rule_gsap_import_aliases_are_audited() {
    let named_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap as animate } from "gsap";
animate.to(".box", 1, { top: 0, scrollTrigger: { markers: true }, motionPath: true });
animate.ticker.lagSmoothing(0);"#,
    );
    assert!(fired(&named_alias, ids::CORE_GSAP2_SIGNATURE));
    assert!(fired(&named_alias, ids::CORE_LAYOUT_PROP_ANIMATION));
    assert!(fired(&named_alias, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
    assert!(fired(
        &named_alias,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));
    assert!(fired(&named_alias, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));

    let default_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import animate from "gsap";
const tl = animate.timeline({ scrollTrigger: { markers: true } });
tl.to(".box", { top: 0 });"#,
    );
    assert!(fired(&default_alias, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
    assert!(fired(&default_alias, ids::CORE_LAYOUT_PROP_ANIMATION));

    let registered = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap as animate } from "gsap";
animate.registerPlugin(ScrollTrigger);
animate.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(!fired(
        &registered,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));
}

#[test]
fn rule_plugin_used_without_register() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(ScrollTrigger); ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(&clean, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    let aliased = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger as ST } from "gsap/ScrollTrigger";
ST.create({ trigger: ".x" });"#,
    );
    assert!(fired(&aliased, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    let registered_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger as ST } from "gsap/ScrollTrigger";
gsap.registerPlugin(ST);
ST.create({ trigger: ".x" });"#,
    );
    assert!(!fired(
        &registered_alias,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let default_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import ST from "gsap/ScrollTrigger";
ST.create({ trigger: ".x" });"#,
    );
    assert!(fired(
        &default_alias,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let custom_ease = analyze(
        "src/a.ts",
        "ts",
        r#"CustomEase.create("hop", "M0,0 C0.1,0.8 0.2,1 1,0");"#,
    );
    assert!(fired(
        &custom_ease,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let registered_custom_ease = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(CustomEase); CustomEase.create("hop", "M0,0 C0.1,0.8 0.2,1 1,0");"#,
    );
    assert!(!fired(
        &registered_custom_ease,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    for plugin in ["CustomWiggle", "CustomBounce"] {
        let bad_source = format!(r#"{plugin}.create("namedEase", {{}});"#);
        let bad = analyze("src/a.ts", "ts", &bad_source);
        assert!(
            fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER),
            "expected missing-registration finding for {plugin}, got {bad:#?}"
        );

        let registered_source =
            format!(r#"gsap.registerPlugin({plugin}); {plugin}.create("namedEase", {{}});"#);
        let registered = analyze("src/a.ts", "ts", &registered_source);
        assert!(
            !fired(&registered, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER),
            "expected registered {plugin} to be clean, got {registered:#?}"
        );
    }
}

#[test]
fn configured_gsap_module_suppresses_registration_rule() {
    // A workspace re-export that centralizes registerPlugin. The final path
    // segment is `gsap`, so it counts as the configured entrypoint.
    let scoped = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { gsap } from "@scope/motion/web/gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(&scoped, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // The historical forms must keep working.
    for source in ["./gsap", "lib/gsap", "~/lib/gsap"] {
        let legacy = analyze(
            "src/a.tsx",
            "tsx",
            &format!(
                r#"import {{ gsap }} from "{source}";
import {{ ScrollTrigger }} from "gsap/ScrollTrigger";
ScrollTrigger.create({{ trigger: ".x" }});"#
            ),
        );
        assert!(
            !fired(&legacy, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER),
            "configured module {source} should suppress the rule"
        );
    }

    // Negative control: an unrelated module must NOT suppress it.
    let unrelated = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { helper } from "@scope/motion/web/utils";
import { ScrollTrigger } from "gsap/ScrollTrigger";
ScrollTrigger.create({ trigger: ".x" });
void helper;"#,
    );
    assert!(fired(&unrelated, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

#[test]
fn sequenced_dynamic_import_of_configured_gsap_module_suppresses_registration_rule() {
    // An AWAITED dynamic import orders the registration side effect ahead of the
    // later plugin use, so it legitimately suppresses the rule.
    let awaited = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { ScrollTrigger } from "gsap/ScrollTrigger";
export async function setup() {
  await import("@scope/motion/web/gsap");
  ScrollTrigger.create({ trigger: ".x" });
}"#,
    );
    assert!(!fired(&awaited, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // A `.then()` chain sequences the callback after resolution, so it also
    // suppresses. This is the common lazy-load shape.
    let chained = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { ScrollTrigger } from "gsap/ScrollTrigger";
void import("@scope/motion/web/gsap").then(({ gsap }) => {
  gsap.to(".x", { x: 1 });
  ScrollTrigger.create({ trigger: ".x" });
});"#,
    );
    assert!(!fired(&chained, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // A BARE floating import proves nothing about ordering: the plugin is used
    // before the module resolves. That is exactly the bug this rule catches.
    let floating = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { ScrollTrigger } from "gsap/ScrollTrigger";
void import("@scope/motion/web/gsap");
ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(fired(&floating, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // Negative control: awaiting an unrelated module must not suppress.
    let other = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { ScrollTrigger } from "gsap/ScrollTrigger";
export async function setup() {
  await import("@scope/motion/web/utils");
  ScrollTrigger.create({ trigger: ".x" });
}"#,
    );
    assert!(fired(&other, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

#[test]
fn rule_usegsap_not_registered() {
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP } from "@gsap/react";
function C() { useGSAP(() => {}); return null; }"#,
    );
    assert!(fired(&bad, ids::REACT_USEGSAP_NOT_REGISTERED));

    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP } from "@gsap/react";
import { gsap } from "gsap";
gsap.registerPlugin(useGSAP);
function C() { useGSAP(() => {}); return null; }"#,
    );
    assert!(!fired(&clean, ids::REACT_USEGSAP_NOT_REGISTERED));

    let aliased_clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP as useGsap } from "@gsap/react";
import { gsap } from "gsap";
gsap.registerPlugin(useGsap);
function C() { useGsap(() => {}); return null; }"#,
    );
    assert!(!fired(&aliased_clean, ids::REACT_USEGSAP_NOT_REGISTERED));

    let configured_gsap_clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { gsap } from "@/lib/gsap";
import { useGSAP } from "@gsap/react";
function C() { useGSAP(() => gsap.to(".box", { x: 100 })); return null; }"#,
    );
    assert!(!fired(
        &configured_gsap_clean,
        ids::REACT_USEGSAP_NOT_REGISTERED
    ));

    let configured_usegsap_alias_clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP as useGsap } from "@/lib/gsap";
function C() { useGsap(() => {}); return null; }"#,
    );
    assert!(!fired(
        &configured_usegsap_alias_clean,
        ids::REACT_USEGSAP_NOT_REGISTERED
    ));

    let aliased_bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP as useGsap } from "@gsap/react";
function C() { useGsap(() => {}); return null; }"#,
    );
    assert!(fired(&aliased_bad, ids::REACT_USEGSAP_NOT_REGISTERED));

    let type_only = analyze(
        "app/page.tsx",
        "tsx",
        r#"import type { useGSAP } from "@gsap/react";
type Hook = typeof useGSAP;
export default function Page(_props: { hook?: Hook }) { return null; }"#,
    );
    assert!(!fired(&type_only, ids::REACT_USEGSAP_NOT_REGISTERED));
    assert!(!fired(&type_only, ids::REACT_GSAP_IN_SSR));
}

#[test]
fn rule_gsap_in_ssr() {
    let bad = analyze(
        "app/page.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(fired(&bad, ids::REACT_GSAP_IN_SSR));

    let clean = analyze(
        "app/page.tsx",
        "tsx",
        r#""use client";
import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&clean, ids::REACT_GSAP_IN_SSR));

    // A file outside app should not trigger SSR even without use client.
    let outside = analyze(
        "src/widget.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function W() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&outside, ids::REACT_GSAP_IN_SSR));

    let pages_router = analyze(
        "pages/index.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&pages_router, ids::REACT_GSAP_IN_SSR));

    let usegsap_only = analyze(
        "app/page.tsx",
        "tsx",
        r#"import { useGSAP } from "@gsap/react";
export default function Page() { useGSAP(() => {}); return null; }"#,
    );
    assert!(fired(&usegsap_only, ids::REACT_GSAP_IN_SSR));

    let configured_usegsap_alias = analyze(
        "app/page.tsx",
        "tsx",
        r#"import { useGSAP as useGsap } from "@/lib/gsap";
export default function Page() { useGsap(() => gsap.to(".box", { x: 1 })); return null; }"#,
    );
    assert!(fired(&configured_usegsap_alias, ids::REACT_GSAP_IN_SSR));
    assert!(fired(
        &configured_usegsap_alias,
        ids::REACT_UNSCOPED_SELECTOR
    ));

    let src_app = analyze(
        "src/app/page.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function Page() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(fired(&src_app, ids::REACT_GSAP_IN_SSR));

    let nested_app_folder = analyze(
        "src/components/app/Widget.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export default function Widget() { gsap.to(".x", { x: 1 }); return null; }"#,
    );
    assert!(!fired(&nested_app_folder, ids::REACT_GSAP_IN_SSR));

    let type_only = analyze(
        "app/page.tsx",
        "tsx",
        r#"import type { ScrollTrigger } from "gsap/ScrollTrigger";
type Props = { trigger: ScrollTrigger };
export default function Page(_props: Props) { return null; }"#,
    );
    assert!(!fired(&type_only, ids::REACT_GSAP_IN_SSR));
}

#[test]
fn rule_unscoped_selector_semantic() {
    // useGSAP with a string selector and no scope -> fires.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  });
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_UNSCOPED_SELECTOR));

    // gsap.context with a string selector and no scope -> fires.
    let bad_context = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(".box", { x: 100 });
  });
  return () => ctx.revert();
}"#,
    );
    assert!(fired(&bad_context, ids::REACT_UNSCOPED_SELECTOR));
}

#[test]
fn rule_unscoped_selector_clean_with_scope() {
    // useGSAP with a scope config -> does NOT fire (clean React snippet).
    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const container = useRef(null);
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, { scope: container });
  return null;
}"#,
    );
    assert!(!fired(&clean, ids::REACT_UNSCOPED_SELECTOR));

    let config_variable = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const container = useRef(null);
  const opts = { scope: container };
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, opts);
  return null;
}"#,
    );
    assert!(!fired(&config_variable, ids::REACT_UNSCOPED_SELECTOR));

    // gsap.context with a scope ref argument -> does NOT fire.
    let clean_context = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(".box", { x: 100 });
  }, containerRef);
  return () => ctx.revert();
}"#,
    );
    assert!(!fired(&clean_context, ids::REACT_UNSCOPED_SELECTOR));
}

#[test]
fn rule_context_missing_revert_semantic() {
    // ctx created, never reverted, never returned -> fires.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {
    gsap.to(refEl, { x: 100 });
  }, scopeRef);
  console.log(ctx);
  return null;
}",
    );
    assert!(fired(&bad, ids::REACT_CONTEXT_MISSING_REVERT));

    // ctx reverted in returned cleanup -> does NOT fire.
    let clean_revert = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {
    gsap.to(refEl, { x: 100 });
  }, scopeRef);
  return () => ctx.revert();
}",
    );
    assert!(!fired(&clean_revert, ids::REACT_CONTEXT_MISSING_REVERT));

    let property_read = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {});
  console.log(ctx.revert);
  return null;
}",
    );
    assert!(fired(&property_read, ids::REACT_CONTEXT_MISSING_REVERT));

    let argument_to_other_call = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {});
  foo.revert(ctx);
  return null;
}",
    );
    assert!(fired(
        &argument_to_other_call,
        ids::REACT_CONTEXT_MISSING_REVERT
    ));

    let discarded = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  gsap.context(() => {});
  return null;
}",
    );
    assert!(fired(&discarded, ids::REACT_CONTEXT_MISSING_REVERT));
}

#[test]
fn clean_react_usegsap_snippet_has_zero_findings() {
    // A fully idiomatic React + useGSAP snippet should yield no findings at all.
    let clean = analyze(
        "src/Component.tsx",
        "tsx",
        r#""use client";
import { gsap } from "gsap";
import { useGSAP } from "@gsap/react";
import { useRef } from "react";

gsap.registerPlugin(useGSAP);

export default function Component() {
  const container = useRef(null);
  useGSAP(
    () => {
      gsap.to(boxRef.current, { duration: 1, x: 100 });
    },
    { scope: container }
  );
  return null;
}"#,
    );
    assert_eq!(clean, Vec::new(), "expected zero findings, got: {clean:#?}");
}

#[test]
fn category_filtering_independent_of_rule_count() {
    // Sanity: a snippet with multiple rule classes produces multiple ids.
    let findings = analyze(
        "src/a.ts",
        "ts",
        r#"import { Flip } from "gsap-trial";
gsap.to(".box", 1, { top: 0, scrollTrigger: { markers: true } });
gsap.ticker.lagSmoothing(0);"#,
    );
    assert!(fired(&findings, ids::CORE_GSAP_TRIAL_IMPORT));
    assert!(fired(&findings, ids::CORE_GSAP2_SIGNATURE));
    assert!(fired(&findings, ids::CORE_LAYOUT_PROP_ANIMATION));
    assert!(fired(&findings, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
    assert!(fired(&findings, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));
    // No duplicate trial-import findings for a single import.
    assert_eq!(count(&findings, ids::CORE_GSAP_TRIAL_IMPORT), 1);
}

// ---------------------------------------------------------------------------
// Fix 1: context-missing-revert must not be suppressed by an unrelated return.
// ---------------------------------------------------------------------------

#[test]
fn rule_context_missing_revert_return_jsx_using_ctx_still_fires() {
    // `return <div>{String(ctx)}</div>` reads ctx but does NOT tear it down.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {});
  return <div>{String(ctx)}</div>;
}",
    );
    assert!(fired(&bad, ids::REACT_CONTEXT_MISSING_REVERT));
}

#[test]
fn rule_context_missing_revert_bare_return_ctx_does_not_fire() {
    // `return ctx;` hands the handle to a parent that can revert it.
    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {});
  return ctx;
}",
    );
    assert!(!fired(&clean, ids::REACT_CONTEXT_MISSING_REVERT));
}

#[test]
fn rule_context_missing_revert_returned_cleanup_does_not_fire() {
    // `return () => ctx.revert();` is the canonical cleanup pattern.
    let clean = analyze(
        "src/a.tsx",
        "tsx",
        r"function C() {
  const ctx = gsap.context(() => {});
  return () => ctx.revert();
}",
    );
    assert!(!fired(&clean, ids::REACT_CONTEXT_MISSING_REVERT));
}

// ---------------------------------------------------------------------------
// Fix 2: GSDevTools type-only references must not flag.
// ---------------------------------------------------------------------------

#[test]
fn rule_gsdevtools_type_position_does_not_fire() {
    let var_type = analyze("src/a.ts", "ts", r"let x: GSDevTools;");
    assert!(!fired(&var_type, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let param_type = analyze("src/a.ts", "ts", r"function f(p: GSDevTools) { return p; }");
    assert!(!fired(&param_type, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));
}

#[test]
fn rule_gsdevtools_value_use_still_fires() {
    let value_call = analyze("src/a.ts", "ts", r"GSDevTools.create();");
    assert!(fired(&value_call, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));

    let value_import = analyze(
        "src/a.ts",
        "ts",
        r#"import { GSDevTools } from "gsap/GSDevTools"; GSDevTools.create();"#,
    );
    assert!(fired(&value_import, ids::PLUGINS_GSDEVTOOLS_IN_SOURCE));
}

// ---------------------------------------------------------------------------
// Fix 3: markers / scrub+toggleActions gated to GSAP/ScrollTrigger context.
// ---------------------------------------------------------------------------

#[test]
fn rule_markers_unrelated_object_does_not_fire() {
    let unrelated = analyze("src/a.ts", "ts", r"const opts = { markers: true };");
    assert!(!fired(&unrelated, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let tween_vars = analyze("src/a.ts", "ts", r"gsap.to(el, { markers: true });");
    assert!(!fired(&tween_vars, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let tween_scrub = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(el, { scrub: 1, toggleActions: "play none none none" });"#,
    );
    assert!(!fired(
        &tween_scrub,
        ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS
    ));
}

#[test]
fn rule_markers_in_nested_scrolltrigger_fires() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".x", { scrollTrigger: { markers: true } });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let timeline = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline({ scrollTrigger: { trigger: ".x", markers: true } });"#,
    );
    assert!(fired(&timeline, ids::SCROLLTRIGGER_MARKERS_IN_PROD));

    let batch = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.batch(".card", { markers: true });"#,
    );
    assert!(fired(&batch, ids::SCROLLTRIGGER_MARKERS_IN_PROD));
}

#[test]
fn rule_scrub_toggleactions_in_scrolltrigger_create_fires() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"ScrollTrigger.create({ scrub: 1, toggleActions: "play none none none" });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));

    let timeline = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline({ scrollTrigger: { scrub: 1, toggleActions: "play none none none" } });"#,
    );
    assert!(fired(
        &timeline,
        ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS
    ));

    let aliased = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger as ST } from "gsap/ScrollTrigger";
ST.create({ scrub: 1, toggleActions: "play none none none" });"#,
    );
    assert!(fired(&aliased, ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS));
}

#[test]
fn rule_scrub_toggleactions_unrelated_object_does_not_fire() {
    let unrelated = analyze(
        "src/a.ts",
        "ts",
        r#"const opts = { scrub: 1, toggleActions: "play none none none" };"#,
    );
    assert!(!fired(
        &unrelated,
        ids::SCROLLTRIGGER_SCRUB_WITH_TOGGLEACTIONS
    ));
}

// ---------------------------------------------------------------------------
// Fix 4: fromTo layout-prop check inspects both fromVars and toVars.
// ---------------------------------------------------------------------------

#[test]
fn rule_layout_prop_fromto_scans_fromvars() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.fromTo(".b", { top: 0 }, { x: 100 });"#,
    );
    assert!(fired(&bad, ids::CORE_LAYOUT_PROP_ANIMATION));
}

#[test]
fn rule_layout_prop_fromto_scans_tovars() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.fromTo(".b", { x: 0 }, { top: 100 });"#,
    );
    assert!(fired(&bad, ids::CORE_LAYOUT_PROP_ANIMATION));
}

#[test]
fn rule_timeline_tween_calls_are_audited() {
    let alias_layout = analyze(
        "src/a.ts",
        "ts",
        r#"const tl = gsap.timeline(); tl.to(".box", { top: 0 });"#,
    );
    assert!(fired(&alias_layout, ids::CORE_LAYOUT_PROP_ANIMATION));

    let chained_signature = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline().to(".box", 1, { x: 100 });"#,
    );
    assert!(fired(&chained_signature, ids::CORE_GSAP2_SIGNATURE));

    let fluent_layout = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline().to(".a", { x: 100 }).to(".b", { top: 0 });"#,
    );
    assert!(fired(&fluent_layout, ids::CORE_LAYOUT_PROP_ANIMATION));

    let label_then_layout = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline().addLabel("intro").to(".box", { top: 0 });"#,
    );
    assert!(fired(&label_then_layout, ids::CORE_LAYOUT_PROP_ANIMATION));

    let add_then_plugin_vars = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline().add(() => {}).to(".box", { motionPath: true });"#,
    );
    assert!(fired(
        &add_then_plugin_vars,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let positioned_layout = analyze(
        "src/a.ts",
        "ts",
        r#"const tl = gsap.timeline(); tl.to(".box", { top: 0 }, "<");"#,
    );
    assert!(fired(&positioned_layout, ids::CORE_LAYOUT_PROP_ANIMATION));

    let fluent_scrolltrigger = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline().to(".a", { x: 100 }).to(".b", { scrollTrigger: { markers: true } });"#,
    );
    assert!(fired(
        &fluent_scrolltrigger,
        ids::SCROLLTRIGGER_MARKERS_IN_PROD
    ));

    let alias_scrolltrigger = analyze(
        "src/a.ts",
        "ts",
        r#"const tl = gsap.timeline(); tl.to(".box", { scrollTrigger: { markers: true } });"#,
    );
    assert!(fired(
        &alias_scrolltrigger,
        ids::SCROLLTRIGGER_MARKERS_IN_PROD
    ));
}

// ---------------------------------------------------------------------------
// Fix 5: registerPlugin argument handling.
// ---------------------------------------------------------------------------

#[test]
fn rule_plugin_register_via_array_still_fires() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r"gsap.registerPlugin([ScrollTrigger]); ScrollTrigger.create({});",
    );
    assert!(fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

#[test]
fn rule_plugin_register_via_spread_suppresses_check() {
    let clean = analyze(
        "src/a.ts",
        "ts",
        r"gsap.registerPlugin(...plugins); ScrollTrigger.create({});",
    );
    assert!(!fired(&clean, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

// ---------------------------------------------------------------------------
// Fix 6: unscoped-selector traversal reaches loop/try/switch bodies.
// ---------------------------------------------------------------------------

#[test]
fn rule_unscoped_selector_inside_for_loop_fires() {
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    for (let i = 0; i < 3; i++) {
      gsap.to(".box", { x: 1 });
    }
  });
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_UNSCOPED_SELECTOR));
}

// ---------------------------------------------------------------------------
// Fix 8: lagSmoothing(-0) is treated as disabled.
// ---------------------------------------------------------------------------

#[test]
fn rule_lag_smoothing_negative_zero_fires() {
    let bad = analyze("src/a.ts", "ts", r"gsap.ticker.lagSmoothing(-0);");
    assert!(fired(&bad, ids::PERFORMANCE_LAG_SMOOTHING_DISABLED));
}

// ---------------------------------------------------------------------------
// Review fix: ScrollTrigger used via a `scrollTrigger:` tween/timeline config
// (not only `ScrollTrigger.create`) counts as usage for missing-registration.
// ---------------------------------------------------------------------------

#[test]
fn rule_scrolltrigger_config_without_register_fires() {
    // gsap.to(target, { scrollTrigger: {...} }) without registering ScrollTrigger.
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // gsap.timeline({ scrollTrigger: {...} }) likewise.
    let bad_tl = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.timeline({ scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(fired(&bad_tl, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // Registered -> does NOT fire.
    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.registerPlugin(ScrollTrigger); gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(!fired(&clean, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));

    // No scrollTrigger config -> does NOT fire.
    let plain = analyze("src/a.ts", "ts", r#"gsap.to(".x", { x: 100 });"#);
    assert!(!fired(&plain, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER));
}

#[test]
fn rule_plugin_vars_without_register_fire() {
    for vars_key in [
        "motionPath",
        "drawSVG",
        "morphSVG",
        "text",
        "scrollTo",
        "inertia",
        "scrambleText",
        "physics2D",
        "physicsProps",
        "pixi",
    ] {
        let source = format!(r"gsap.to(el, {{ {vars_key}: true }});");
        let bad = analyze("src/a.ts", "ts", &source);
        assert!(
            fired(&bad, ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER),
            "expected plugin-vars finding for {vars_key}, got {bad:#?}"
        );
    }

    let registered = analyze(
        "src/a.ts",
        "ts",
        r"gsap.registerPlugin(MotionPathPlugin); gsap.to(el, { motionPath: true });",
    );
    assert!(!fired(
        &registered,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap } from "@/lib/gsap";
gsap.to(el, { motionPath: true });"#,
    );
    assert!(!fired(
        &configured,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));
}

#[test]
fn rule_configured_gsap_reexports_do_not_require_local_register() {
    let configured_plugin = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger } from "@/lib/gsap";
ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(
        &configured_plugin,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import { ScrollTrigger as ST } from "@/lib/gsap";
ST.create({ trigger: ".x" });"#,
    );
    assert!(!fired(
        &configured_alias,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_gsap = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap } from "@/lib/gsap";
gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(!fired(
        &configured_gsap,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_default_gsap = analyze(
        "src/a.ts",
        "ts",
        r#"import gsap from "@/lib/gsap";
gsap.to(".x", { scrollTrigger: { trigger: ".x" } });"#,
    );
    assert!(!fired(
        &configured_default_gsap,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_default_alias = analyze(
        "src/a.ts",
        "ts",
        r#"import animate from "@/lib/gsap";
animate.to(".x", 1, { top: 0, scrollTrigger: { markers: true } });"#,
    );
    assert!(fired(&configured_default_alias, ids::CORE_GSAP2_SIGNATURE));
    assert!(fired(
        &configured_default_alias,
        ids::CORE_LAYOUT_PROP_ANIMATION
    ));
    assert!(fired(
        &configured_default_alias,
        ids::SCROLLTRIGGER_MARKERS_IN_PROD
    ));
    assert!(!fired(
        &configured_default_alias,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));

    let configured_gsap_direct_plugin = analyze(
        "src/a.ts",
        "ts",
        r#"import { gsap } from "@/lib/gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
ScrollTrigger.create({ trigger: ".x" });"#,
    );
    assert!(!fired(
        &configured_gsap_direct_plugin,
        ids::PLUGINS_PLUGIN_USED_WITHOUT_REGISTER
    ));
}

// ---------------------------------------------------------------------------
// Review fix: useGSAP(cb, []) dependency-array overload is not a scope, so an
// unscoped selector inside it still fires.
// ---------------------------------------------------------------------------

#[test]
fn rule_unscoped_selector_with_dependency_array_fires() {
    // useGSAP(cb, []) -> the array is deps, not a scope -> fires.
    let bad = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, []);
  return null;
}"#,
    );
    assert!(fired(&bad, ids::REACT_UNSCOPED_SELECTOR));

    // useGSAP(cb, { dependencies: [...] }) without a scope key -> still fires.
    let bad_config = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    gsap.to(".box", { x: 100 });
  }, { dependencies: [a] });
  return null;
}"#,
    );
    assert!(fired(&bad_config, ids::REACT_UNSCOPED_SELECTOR));

    let bad_timeline = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => {
    const tl = gsap.timeline();
    tl.to(".box", { x: 100 });
  });
  return null;
}"#,
    );
    assert!(fired(&bad_timeline, ids::REACT_UNSCOPED_SELECTOR));

    let concise_arrow = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  useGSAP(() => gsap.to(".box", { x: 100 }));
  return null;
}"#,
    );
    assert!(fired(&concise_arrow, ids::REACT_UNSCOPED_SELECTOR));

    let logical_guard = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C({ ready }) {
  useGSAP(() => ready && gsap.to(".box", { x: 1 }));
  return null;
}"#,
    );
    assert!(fired(&logical_guard, ids::REACT_UNSCOPED_SELECTOR));

    let conditional_guard = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C({ ready }) {
  useGSAP(() => ready ? gsap.to(".box", { x: 1 }) : null);
  return null;
}"#,
    );
    assert!(fired(&conditional_guard, ids::REACT_UNSCOPED_SELECTOR));

    let aliased_hook = analyze(
        "src/a.tsx",
        "tsx",
        r#"import { useGSAP as useGsap } from "@gsap/react";
function C() {
  useGsap(() => {
    gsap.to(".box", { x: 100 });
  }, []);
  return null;
}"#,
    );
    assert!(fired(&aliased_hook, ids::REACT_UNSCOPED_SELECTOR));

    // gsap.context(cb, scopeRef) -> the bare ref IS a scope -> does NOT fire.
    let clean_context = analyze(
        "src/a.tsx",
        "tsx",
        r#"function C() {
  const ctx = gsap.context(() => {
    gsap.to(".box", { x: 100 });
  }, root);
  return () => ctx.revert();
}"#,
    );
    assert!(!fired(&clean_context, ids::REACT_UNSCOPED_SELECTOR));
}

#[test]
fn rule_state_in_continuous_motion_fires_for_scroll_handler() {
    let bad = analyze(
        "src/Hero.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Hero() {
  const [progress, setProgress] = useState(0);
  return <div onScroll={(e) => setProgress(e.currentTarget.scrollTop)} />;
}
"#,
    );
    assert!(fired(&bad, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_fires_for_gsap_on_update_and_raf() {
    let on_update = analyze(
        "src/Panel.tsx",
        "tsx",
        r#"
import { useState } from "react";
import gsap from "gsap";
export function Panel() {
  const [y, setY] = useState(0);
  gsap.to(".p", { scrollTrigger: { onUpdate: (self) => setY(self.progress) } });
  return null;
}
"#,
    );
    assert!(fired(&on_update, ids::REACT_STATE_IN_CONTINUOUS_MOTION));

    // A callback that schedules the next frame from inside itself is a loop.
    let raf_loop = analyze(
        "src/Ticker.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Ticker() {
  const [t, setT] = useState(0);
  requestAnimationFrame(function tick() {
    setT(performance.now());
    requestAnimationFrame(tick);
  });
  return null;
}
"#,
    );
    assert!(fired(&raf_loop, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_ignores_a_one_shot_animation_frame() {
    // `requestAnimationFrame` runs its callback once unless something
    // reschedules. A single deferred state update is not a per-frame render.
    let findings = analyze(
        "src/Ready.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Ready() {
  const [ready, setReady] = useState(false);
  requestAnimationFrame(() => setReady(true));
  return null;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_ignores_on_update_outside_gsap() {
    // `onUpdate` is an ordinary callback name. Only a GSAP call puts a frame
    // loop behind it, so an unrelated library's config must not be reported.
    let findings = analyze(
        "src/Editor.tsx",
        "tsx",
        r#"
import { useState } from "react";
import { editor } from "some-editor";
export function Editor() {
  const [value, setValue] = useState("");
  editor.configure({ onUpdate: () => setValue("x") });
  return null;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_ignores_on_refresh() {
    // ScrollTrigger fires onRefresh on refresh, not per frame.
    let findings = analyze(
        "src/Refresh.tsx",
        "tsx",
        r#"
import { useState } from "react";
import gsap from "gsap";
export function Refresh() {
  const [n, setN] = useState(0);
  gsap.to(".p", { scrollTrigger: { onRefresh: () => setN(1) } });
  return null;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_resolves_setters_by_symbol() {
    // Two components, each with its own `setProgress`. Only the one actually
    // bound to useState inside a continuous handler may be reported; a
    // same-named prop must not be.
    let findings = analyze(
        "src/Two.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Passthrough({ setProgress }: { setProgress: (n: number) => void }) {
  return <div onScroll={() => setProgress(1)} />;
}
export function Owner() {
  const [progress, setProgress] = useState(0);
  return <div onClick={() => setProgress(1)} />;
}
"#,
    );
    assert!(
        !fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION),
        "a prop named like a setter is not a useState setter: {findings:?}"
    );
}

#[test]
fn rule_state_in_continuous_motion_ignores_discrete_handlers() {
    // A state update per click is ordinary React. The defect is a state update
    // per frame, so a discrete handler in the same component must stay silent.
    let findings = analyze(
        "src/Toggle.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Toggle() {
  const [open, setOpen] = useState(false);
  return <button onClick={() => setOpen(true)} />;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_ignores_a_ref_write() {
    // The prescribed fix must not itself be reported, or the rule pushes
    // authors back toward the defect.
    let findings = analyze(
        "src/Fine.tsx",
        "tsx",
        r#"
import { useRef } from "react";
export function Fine() {
  const ref = useRef(0);
  return <div onScroll={(e) => { ref.current = e.currentTarget.scrollTop; }} />;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_is_catalogued_under_react() {
    let bad = analyze(
        "src/Hero.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Hero() {
  const [p, setP] = useState(0);
  return <div onPointerMove={() => setP(1)} />;
}
"#,
    );
    let finding = bad
        .iter()
        .find(|f| f.id == ids::REACT_STATE_IN_CONTINUOUS_MOTION)
        .expect("finding");
    assert_eq!(finding.category, Category::React);
}

#[test]
fn rule_state_in_continuous_motion_resolves_aliased_use_state_imports() {
    // `import { useState as useReactState }` binds a different local name; a
    // text comparison would miss the setter entirely.
    let findings = analyze(
        "src/Aliased.tsx",
        "tsx",
        r#"
import { useState as useReactState } from "react";
export function Aliased() {
  const [progress, setProgress] = useReactState(0);
  return <div onScroll={(e) => setProgress(e.currentTarget.scrollTop)} />;
}
"#,
    );
    assert!(fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_ignores_a_non_gsap_ticker() {
    // `analytics.ticker.add` shares a property name with gsap.ticker but has
    // no frame loop behind it.
    let findings = analyze(
        "src/Analytics.tsx",
        "tsx",
        r#"
import { useState } from "react";
import { analytics } from "./analytics";
export function Analytics() {
  const [n, setN] = useState(0);
  analytics.ticker.add(() => setN(1));
  return null;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_ignores_a_finite_frame_sequence() {
    // Two scheduled frames is not a loop: neither callback reschedules itself.
    let findings = analyze(
        "src/TwoFrames.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function TwoFrames() {
  const [x, setX] = useState(0);
  requestAnimationFrame(() => {
    setX(1);
    requestAnimationFrame(renderOnce);
  });
  return null;
}
"#,
    );
    assert!(!fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_recognises_aliased_scroll_trigger() {
    let findings = analyze(
        "src/Aliased.tsx",
        "tsx",
        r#"
import { useState } from "react";
import { ScrollTrigger as ST } from "gsap/ScrollTrigger";
export function Aliased() {
  const [x, setX] = useState(0);
  ST.create({ onUpdate: (self) => setX(self.progress) });
  return null;
}
"#,
    );
    assert!(fired(&findings, ids::REACT_STATE_IN_CONTINUOUS_MOTION));
}

#[test]
fn rule_state_in_continuous_motion_downgrades_a_boolean_threshold() {
    // React bails out when the next state equals the current one, so a
    // threshold boolean re-renders only on the flip. Report it, but do not
    // fail a default-severity gate on the prescribed pattern.
    let findings = analyze(
        "src/Stuck.tsx",
        "tsx",
        r#"
import { useState } from "react";
export function Stuck() {
  const [stuck, setStuck] = useState(false);
  return <div onScroll={(e) => setStuck(e.currentTarget.scrollTop > 100)} />;
}
"#,
    );
    let finding = findings
        .iter()
        .find(|f| f.id == ids::REACT_STATE_IN_CONTINUOUS_MOTION)
        .expect("still reported");
    assert_eq!(
        finding.severity,
        gsap_audit_core::types::Severity::Low,
        "a boolean threshold must not trip the default medium gate"
    );
}

#[test]
fn rule_nested_timeline_child_scrolltrigger_fires_and_top_level_tween_does_not() {
    let bad = analyze(
        "src/timeline.ts",
        "ts",
        r#"gsap.timeline().from(".a", { opacity: 0 }).to(".b", { scrollTrigger: { trigger: ".b" }, x: 10 });"#,
    );
    assert!(fired(&bad, ids::SCROLLTRIGGER_NESTED_TIMELINE_CHILD));

    let clean = analyze(
        "src/tween.ts",
        "ts",
        r#"gsap.to(".b", { scrollTrigger: { trigger: ".b" }, x: 10 });"#,
    );
    assert!(!fired(&clean, ids::SCROLLTRIGGER_NESTED_TIMELINE_CHILD));
}

#[test]
fn rule_tween_in_render_fires_and_hook_callback_does_not() {
    let bad = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import React from "react";
import { gsap } from "gsap";
export function Card() {
  gsap.to(ref.current, { x: 10 });
  return <div />;
}"#,
    );
    assert!(fired(&bad, ids::REACT_TWEEN_IN_RENDER));

    let clean = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import React, { useLayoutEffect } from "react";
import { gsap } from "gsap";
export function Card() {
  useLayoutEffect(() => { gsap.to(ref.current, { x: 10 }); }, []);
  return <div onClick={() => gsap.set(ref.current, { x: 0 })} />;
}"#,
    );
    assert!(!fired(&clean, ids::REACT_TWEEN_IN_RENDER));

    // Automatic JSX runtime: no React import, but JSX marks the component.
    let automatic_runtime = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap";
export function Card() {
  gsap.to(ref.current, { x: 10 });
  return <div />;
}"#,
    );
    assert!(fired(&automatic_runtime, ids::REACT_TWEEN_IN_RENDER));
}

#[test]
fn rule_permanent_will_change_fires_and_unrelated_style_does_not() {
    let bad = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.to(".box", { "will-change": "transform", x: 10 });
gsap.timeline({ defaults: { willChange: "transform" } });"#,
    );
    assert!(fired(&bad, ids::PERFORMANCE_WILL_CHANGE_PERMANENT));

    let clean = analyze(
        "src/a.ts",
        "ts",
        r#"const style = { willChange: "transform" }; gsap.to(".box", { x: 10 });"#,
    );
    assert!(!fired(&clean, ids::PERFORMANCE_WILL_CHANGE_PERMANENT));

    // Reset values release the layer instead of holding it.
    let cleanup = analyze(
        "src/a.ts",
        "ts",
        r#"gsap.set(".box", { willChange: "auto" }); gsap.set(".other", { "will-change": "AUTO" });"#,
    );
    assert!(!fired(&cleanup, ids::PERFORMANCE_WILL_CHANGE_PERMANENT));
}

#[test]
fn rule_event_tween_missing_overwrite_fires_and_protected_tween_does_not() {
    let bad = analyze(
        "src/Button.tsx",
        "tsx",
        r"function onPointerEnter() { gsap.to(button, { scale: 1.05 }); }",
    );
    assert!(fired(&bad, ids::CORE_MISSING_OVERWRITE));

    let clean = analyze(
        "src/Button.tsx",
        "tsx",
        r#"const view = <button onPointerEnter={() => gsap.to(button, { scale: 1.05, overwrite: "auto" })} />;"#,
    );
    assert!(!fired(&clean, ids::CORE_MISSING_OVERWRITE));
}

#[test]
fn rule_matchmedia_missing_revert_fires_and_cleanup_does_not() {
    let bad = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; const mm = gsap.matchMedia(); mm.add("(min-width: 800px)", () => {});"#,
    );
    assert!(fired(&bad, ids::REACT_MATCHMEDIA_MISSING_REVERT));

    let clean = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; const mm = gsap.matchMedia(); useEffect(() => () => mm.revert(), []);"#,
    );
    assert!(!fired(&clean, ids::REACT_MATCHMEDIA_MISSING_REVERT));

    // An unrelated useGSAP call elsewhere cannot clean up this context.
    let sibling_usegsap = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; import { useGSAP } from "@gsap/react"; const mm = gsap.matchMedia(); useGSAP(() => { gsap.to(".x", { x: 1 }); });"#,
    );
    assert!(fired(
        &sibling_usegsap,
        ids::REACT_MATCHMEDIA_MISSING_REVERT
    ));

    // A revert on any other object is not cleanup for this binding.
    let foreign_revert = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; const mm = gsap.matchMedia(); const ctx = gsap.context(() => {}); ctx.revert();"#,
    );
    assert!(fired(&foreign_revert, ids::REACT_MATCHMEDIA_MISSING_REVERT));

    // matchMedia created inside the useGSAP callback is auto-cleaned.
    let inside_usegsap = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; import { useGSAP } from "@gsap/react"; useGSAP(() => { const mm = gsap.matchMedia(); mm.add("(min-width: 800px)", () => {}); });"#,
    );
    assert!(!fired(
        &inside_usegsap,
        ids::REACT_MATCHMEDIA_MISSING_REVERT
    ));

    // Deferred creation runs after the useGSAP callback returns, so the
    // context never registers it: still a leak without mm.revert().
    let deferred = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; import { useGSAP } from "@gsap/react"; useGSAP(() => { setTimeout(() => { const mm = gsap.matchMedia(); mm.add("(min-width: 800px)", () => {}); }, 100); });"#,
    );
    assert!(fired(&deferred, ids::REACT_MATCHMEDIA_MISSING_REVERT));

    // Lexical shadowing: a revert in one scope must not cover another
    // scope's binding that merely shares the spelling.
    let shadowed = analyze(
        "src/Card.tsx",
        "tsx",
        r#"import { gsap } from "gsap"; function a() { const mm = gsap.matchMedia(); } function b() { const mm = gsap.matchMedia(); mm.revert(); }"#,
    );
    assert!(fired(&shadowed, ids::REACT_MATCHMEDIA_MISSING_REVERT));
}
