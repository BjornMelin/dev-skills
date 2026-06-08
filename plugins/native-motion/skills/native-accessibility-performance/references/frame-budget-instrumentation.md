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
