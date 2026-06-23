# Expo SDK compatibility matrix

Skill: native-motion-core
Checked at: 2026-06-04

## When To Load

- Read before changing package versions or applying npm-latest examples in Expo projects.


## Operating Guidance

Expo and React Native product motion with Reanimated 4, Worklets, shared values, animated styles/props, gestures, scroll handlers, layout animations, CSS transitions, and migration boundaries.

### Decision Boundaries

- Use native-validation for command/device proof.
- Use native-skia for canvas-heavy effects.
- Use native-lottie/native-rive for designer-authored assets.

### Workflow Details

1. Inspect Expo SDK, RN, Reanimated, Worklets, Gesture Handler, Babel config, and New Architecture mode.
2. Pick the smallest primitive: RN state/style, Reanimated CSS, shared values, gestures, scroll, or layout animation.
3. Keep product state in React/store and transient motion in shared values.
4. Validate interruption, unmount, reduced motion, and iOS/Android behavior.

### Gotchas

- Reanimated 4 requires compatible Worklets and New Architecture; Reanimated 3 advice differs.
- Functions passed to scheduleOnRN must be defined on the RN runtime, not inside a worklet.
- Reading sharedValue.value on JS can block; derive/consume on the UI thread.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
