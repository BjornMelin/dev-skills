# Native frame budget instrumentation

Skill: native-accessibility-performance
Checked at: 2026-06-04

## When To Load

- Read when reports mention dropped frames, slow gestures, list jank, or JS/UI thread contention.

## Source Anchors

- https://reactnative.dev/docs/animations
- https://docs.swmansion.com/react-native-reanimated/docs/guides/performance/

## Reference Notes

- Classify work by JS thread, UI thread, layout, image decode, list virtualization, and GPU/canvas load before proposing a fix.
- Per-frame React state updates are a common source of animation jank; prefer UI-thread animation values where appropriate.
- Measure on representative devices or simulators when native motion is the user-visible surface.

## Focused Checks

- Check development mode versus release/development-build behavior.
- Inspect lists, gestures, images, and expensive derived values near the animation.

## Failure Modes

- Treating simulator-only proof as enough for GPU-heavy native surfaces.
- Adding more animation wrappers around a list instead of fixing virtualization/layout work.


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
