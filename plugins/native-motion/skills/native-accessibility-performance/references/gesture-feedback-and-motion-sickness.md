# Gesture feedback and motion sickness review

Skill: native-accessibility-performance
Checked at: 2026-06-04

## When To Load

- Read when gestures, haptics, parallax, carousels, shared transitions, or large native motion can affect comfort.

## Source Anchors

- https://reactnative.dev/docs/accessibility
- https://docs.swmansion.com/react-native-reanimated/docs/guides/accessibility/

## Reference Notes

- Reduced motion should preserve orientation, focus, progress, pressed, success, and error feedback while reducing vestibular movement.
- Haptics can reinforce state but cannot replace visible or screen-reader-accessible feedback.
- Large parallax, zoom, rotation, and full-screen movement deserve stricter device proof than small opacity/scale affordances.

## Focused Checks

- Check system reduced-motion setting and any app-level motion preference.
- Verify screen reader labels and focus movement around animated state changes.

## Failure Modes

- Removing all feedback for reduced-motion users.
- Using haptics as the only confirmation of a state change.


## Operating Guidance

Expo/React Native motion accessibility, reduced motion, haptics, UI-thread performance, frame pressure, gestures, and manual device proof.

### Decision Boundaries

- Use native-motion-core for implementation patterns.
- Use native-validation for command and device validation gates.
- Use native-skia or native-three-r3f for canvas/GPU-specific performance.

### Workflow Details

1. Inspect platform, Expo SDK, animation engine, and accessibility setting ownership.
2. Classify UI-thread, JS-thread, layout, and GPU work.
3. Add reduced-motion or static behavior without removing functional feedback.
4. Validate on iOS/Android or document skipped device proof.

### Gotchas

- Reduced motion should not remove essential progress, focus, pressed, or error feedback.
- Haptics are feedback, not a substitute for visible state.
- Per-frame React state updates can destroy native animation performance.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
