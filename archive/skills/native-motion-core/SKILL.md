---
name: native-motion-core
description: >-
  Use this skill for Expo and React Native product motion with Reanimated 4, Worklets, shared values, animated styles/props, gestures, scroll handlers, layout animations, CSS transitions, and migration boundaries. Trigger on react-native-reanimated, react-native-worklets, useSharedValue, withTiming, withSpring, scheduleOnRN, layout animation. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Motion Core

Expo and React Native product motion with Reanimated 4, Worklets, shared values, animated styles/props, gestures, scroll handlers, layout animations, CSS transitions, and migration boundaries.

## Operating Contract

Use this skill as a compact router plus domain checklist. Load references only
when the current task matches their condition. Do not cite local scrape paths,
machine cache paths, or hidden source locations. Verify API details against the
target repo's installed package versions before editing.

## Source Order

1. Inspect the target repo's installed packages, framework/runtime versions,
   local design tokens, accessibility policy, and existing motion patterns.
2. Use the bundled references below for skill-specific gotchas and copied source
   excerpts.
3. Use official current docs/package source as API truth when local code or
   bundled notes are version-sensitive.

## Decision Boundaries

- Use native-validation for command/device proof.
- Use native-skia for canvas-heavy effects.
- Use native-lottie/native-rive for designer-authored assets.

## Workflow

1. Inspect Expo SDK, RN, Reanimated, Worklets, Gesture Handler, Babel config, and New Architecture mode.
2. Pick the smallest primitive: RN state/style, Reanimated CSS, shared values, gestures, scroll, or layout animation.
3. Keep product state in React/store and transient motion in shared values.
4. Validate interruption, unmount, reduced motion, and iOS/Android behavior.

## Gotchas

- Reanimated 4 requires compatible Worklets and New Architecture; Reanimated 3 advice differs.
- Functions passed to scheduleOnRN must be defined on the RN runtime, not inside a worklet.
- Reading sharedValue.value on JS can block; derive/consume on the UI thread.

<!-- skill-resources:start -->
## Bundled Resources

- `references/reanimated-worklets-core.md` - Reanimated 4 and Worklets core workflow. Read before adding shared values, worklets, threading callbacks, or migration changes.
- `references/expo-sdk-compatibility.md` - Expo SDK compatibility matrix. Read before changing package versions or applying npm-latest examples in Expo projects.
- `references/layout-scroll-gesture-patterns.md` - Layout, scroll, and gesture patterns. Read for layout animations, scroll handlers, Gesture Handler 2, and frame callbacks.
- `references/threading-runonjs-scheduleonrn.md` - Reanimated threading and RN callback boundaries. Read when worklets call back to React Native, schedule JS work, or move data between UI and RN runtimes.
- `references/gesture-handler-integration.md` - Gesture Handler and Reanimated integration. Read when pan, fling, tap, scroll, sheet, carousel, or drag interactions drive Reanimated values.
- `references/docs-expo-reanimated.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-4-migration-testing.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-compatibility.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-getting-started.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-performance.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-animation.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-accessibility.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rn-animated-source-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-animation-functions.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-animations-performance.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-canvas-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-canvas-atlas.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-gpu-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-index.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-layout-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-scroll-and-events.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/software-mansion-animations-svg-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-motion-core-audit-report.md` - Audit response/report template.
- `assets/templates/native-motion-core-review-checklist.md` - Manual review checklist.
- `assets/examples/native-motion-core-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-motion-core-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
