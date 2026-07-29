# React Native Performance Notes

Sources: React Native 0.85 Performance and Animations docs plus Callstack
performance workflow slices, checked during the 2026-06-04 audit pass.

Use this file when reviewing frame drops, list animation, JS-thread work, or
profiling evidence.

## Frame Model

- Smooth UI targets at least 60 FPS, leaving about 16.67 ms per frame.
- React Native exposes separate JS and UI frame rates in the Perf Monitor.
- JS-thread stalls delay React work, touch processing, and JS-driven animation.
- Native/UI-thread scroll and native transitions can continue while JS is busy,
  but JS event handling can still lag.
- For high-refresh devices, Reanimated and RN guidance may target 120 FPS, but
  120 FPS proof needs device and build support such as iOS ProMotion settings.

## Common Motion Problems

- Normal development mode is slower than release. Do not make final performance
  calls from regular dev mode alone.
- `console.*` in production paths can become a JS-thread bottleneck.
- Large `FlatList` surfaces need list-specific tuning such as stable item
  layout, lean row render work, memoized handlers, and measured render windows.
- React Native Animations docs note that long or looping `Animated` animations
  can block `VirtualizedList` row rendering unless configured with
  `isInteraction: false`.
- Heavy work directly in `onPress` can delay feedback; split expensive work
  after the immediate visual response.
- Moving transparent text over images on Android can trigger alpha-compositing
  cost. Use hardware texture/rasterization only after profiling memory impact.
- Scaling large images is usually cheaper than animating image width/height.

## Audit Implications

- Prefer transform/opacity for hot animation paths.
- Profile list scroll, transitions, and gesture surfaces on representative
  devices or release-like builds.
- When a motion issue appears only in dev mode, reproduce in release or
  `debugOptimized` before adding complexity.
- Validate that touch responsiveness remains good while the animation runs.
- Measure the exact interaction before and after the fix. Component count,
  tree depth, or generic memoization advice is not enough without profiler or
  behavioral evidence.
