---
name: native-controls-transitions
description: >-
  Use this skill for Expo Router Stack/native-stack transitions, react-native-screens boundaries, Expo UI SwiftUI/Jetpack Compose controls, native control animation ownership, and validation. Trigger on Expo Router Stack transition, native-stack animation, react-native-screens, Expo UI, SwiftUI control, Jetpack Compose control. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Controls Transitions

Expo Router Stack/native-stack transitions, react-native-screens boundaries, Expo UI SwiftUI/Jetpack Compose controls, native control animation ownership, and validation.

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

- Use native-motion-core for Reanimated-owned product motion.
- Use native-styling-boundaries for NativeWind style ownership.
- Use native-validation for EAS/device gates.

## Workflow

1. Identify whether navigation, native control, or app state owns the transition.
2. Prefer platform-native transition knobs before custom overlays.
3. Keep Expo UI controls as leaf native controls.
4. Validate iOS and Android behavior when native controls or navigation config change.

## Gotchas

- Navigation transitions can fight screen-level Reanimated transitions.
- Expo UI control props are not arbitrary React Native View animation surfaces.
- Route params and unmount timing affect transition cleanup.

<!-- skill-resources:start -->
## Bundled Resources

- `references/expo-router-and-screens-transitions.md` - Expo Router and screens transition guide. Read for Stack/native-stack options, route transitions, and screen lifecycle.
- `references/expo-ui-control-boundaries.md` - Expo UI native control boundaries. Read when SwiftUI or Jetpack Compose leaf controls are involved.
- `references/native-navigation-validation.md` - Navigation transition validation. Read before finalizing route, stack, or native control animation changes.
- `references/platform-transition-option-map.md` - Platform transition option map. Read before changing Expo Router Stack, native-stack, presentation, gesture, header, or modal transition options.
- `references/expo-ui-worklets-state.md` - Expo UI worklets and native state boundaries. Read when Expo UI controls, SwiftUI/Jetpack Compose leaf controls, or worklet-backed native state are involved.
- `references/docs-expo-router-stack.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-ui.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-react-native-screens.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-react-navigation-native-stack.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-controls.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-ui-jetpack-compose.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-ui-swift-ui.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-controls-transitions-audit-report.md` - Audit response/report template.
- `assets/templates/native-controls-transitions-review-checklist.md` - Manual review checklist.
- `assets/examples/native-controls-transitions-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-controls-transitions-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
