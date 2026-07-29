# Gesture Handler and Reanimated integration

Skill: native-motion-core
Checked at: 2026-06-04

## When To Load

- Read when pan, fling, tap, scroll, sheet, carousel, or drag interactions drive Reanimated values.

## Source Anchors

- https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/getting-started
- https://reactnative.dev/docs/animations

## Reference Notes

- Gestures should update UI-thread animation state directly when possible and cross to RN only for durable product state changes.
- Gesture cancellation, velocity, bounds, snap points, and simultaneous/require-fail relationships are part of the animation contract.
- Reduced motion does not mean disabling gestures; it usually means simplifying travel, spring, or parallax behavior.

## Focused Checks

- Test cancel, interruption, nested scroll, simultaneous gestures, and platform back/swipe gestures.
- Verify snap points and bounds with dynamic layout sizes.

## Failure Modes

- Updating React state on every gesture frame.
- Hard-coding snap distances without measuring layout.


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
