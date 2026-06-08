# Reanimated threading and RN callback boundaries

Skill: native-motion-core
Checked at: 2026-06-04

## When To Load

- Read when worklets call back to React Native, schedule JS work, or move data between UI and RN runtimes.

## Source Anchors

- https://docs.swmansion.com/react-native-reanimated/docs/guides/worklets
- https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/getting-started

## Reference Notes

- Worklets execute on the UI runtime. Functions that touch React state, navigation, analytics, or ordinary JS objects must be scheduled through the supported RN/JS boundary.
- Define callbacks on the RN runtime before passing them into scheduling helpers; do not create non-worklet functions inside worklets and call them synchronously.
- Avoid reading shared values on JS hot paths when derived UI-thread values would work.

## Focused Checks

- Search for `runOnJS`, `scheduleOnRN`, shared value reads, and functions declared inside worklets.
- Verify callbacks cannot fire after component unmount or route change.

## Failure Modes

- Synchronous calls from UI worklets to non-worklet functions.
- Using shared values as a general cross-thread state store.


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
