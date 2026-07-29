# Reanimated Accessibility Notes

Sources: Reanimated 4 accessibility, `useReducedMotion`, and
`ReducedMotionConfig` docs, checked during the 2026-06-04 audit pass. Expo SDK
56 bundles Reanimated 4.3.1, so verify installed source before relying on
later 4.x implementation details.

Use this file when reviewing reduced-motion behavior in Reanimated code.

## Reduced Motion Model

Reanimated supports `ReduceMotion` values:

- `ReduceMotion.System`: follow the device reduced-motion setting.
- `ReduceMotion.Always`: disable the animation.
- `ReduceMotion.Never`: keep the animation enabled.

By default, Reanimated animations use system reduced-motion behavior. Audit
code that overrides this default.

## Animation Behavior Under Reduced Motion

- `withTiming` and `withSpring` jump to the target value immediately.
- `withDecay` returns the current value immediately, respecting clamp.
- `withDelay` starts the next animation immediately.
- `withRepeat` may not start for infinite/even reversed repetitions, otherwise
  it runs once.
- `withSequence` only starts children that have reduced motion disabled.
- Entering/keyframe/layout animations reach endpoints immediately.
- Exiting animations and shared transitions are omitted.

## APIs

- Use per-animation `reduceMotion` config for specific effects.
- Use `.reduceMotion(...)` on layout animation builders when layout animation
  behavior differs from the default.
- `useReducedMotion()` returns the value from app startup synchronously. It does
  not rerender components when the system setting changes.
- `ReducedMotionConfig` applies globally and should be rare. Prefer local
  configuration unless the app has an explicit product-wide policy.
- If the product needs to react when the setting changes during a session, pair
  Reanimated with React Native `AccessibilityInfo` instead of relying only on
  the startup value from `useReducedMotion()`.

## Review Rules

- Treat `ReduceMotion.Never` as a design exception. Verify the animation is
  essential and has no static equivalent.
- Do not rely on animation completion callbacks to reveal essential content
  under reduced motion unless the reduced-motion behavior is tested.
- For decorative loops, use `Always`, a static poster, or no playback.
- For large spatial movement, provide a reduced variant rather than only a
  faster animation.
