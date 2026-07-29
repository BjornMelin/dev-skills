# Expo Router and screens transition guide

Skill: native-controls-transitions
Checked at: 2026-06-04

## When To Load

- Read for Stack/native-stack options, route transitions, and screen lifecycle.


## Operating Guidance

Expo Router Stack/native-stack transitions, react-native-screens boundaries, Expo UI SwiftUI/Jetpack Compose controls, native control animation ownership, and validation.

### Decision Boundaries

- Use native-motion-core for Reanimated-owned product motion.
- Use native-styling-boundaries for NativeWind style ownership.
- Use native-validation for EAS/device gates.

### Workflow Details

1. Identify whether navigation, native control, or app state owns the transition.
2. Prefer platform-native transition knobs before custom overlays.
3. Keep Expo UI controls as leaf native controls.
4. Validate iOS and Android behavior when native controls or navigation config change.

### Gotchas

- Navigation transitions can fight screen-level Reanimated transitions.
- Expo UI control props are not arbitrary React Native View animation surfaces.
- Route params and unmount timing affect transition cleanup.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
