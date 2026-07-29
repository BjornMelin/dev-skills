# Reanimated Performance And Worklets Notes

Sources: Reanimated 4 performance, feature-flag, Worklets threading docs, Expo
SDK 56 Reanimated page, Callstack performance workflow, Software Mansion
animation slices, and local 4.1.5 package-source checks from the 2026-06-04
audit pass.

Use this file when reviewing Reanimated jank, New Architecture behavior,
threading, shared values, or native feature flags.

## Runtime Rules

- Keep high-frequency animation math in worklets/UI runtime.
- Avoid reading `sharedValue.value` on the RN/JS runtime in render or hot paths;
  derive and consume values in worklets where possible.
- Avoid per-frame `runOnJS`, React state writes, logging, or allocation.
- In Reanimated 4 with `react-native-worklets`, prefer `scheduleOnRN` for
  UI-runtime-to-RN-runtime scheduling when available. If `runOnJS` remains,
  import it from `react-native-worklets`; the Reanimated re-export is
  deprecated in 4.1.5 source.
- Functions scheduled from UI/runtime code must be defined in RN/JS scope, not
  inside the worklet callback.
- Memoize `useFrameCallback` worklets and React Native Gesture Handler gesture
  objects unless React Compiler or local code proves stable identity.
- Follow measure, optimize, re-measure, validate. Do not call a memoization or
  stale-closure issue unless profiler evidence or a clear stale-read path
  supports it.

## Feature Flags

Reanimated 4 static feature flags require native rebuilds and must match the
installed Reanimated/RN versions. Expo SDK 56 bundles Reanimated 4.3.1, but
current Reanimated 4.x docs include flags added in 4.2 and 4.4, so installed
source is the final authority.

- `ANDROID_SYNCHRONOUSLY_UPDATE_UI_PROPS` applies a fast path for supported
  non-layout styles on Android. It can improve smoothness, but transformed
  touch targets may require gesture-handler `Pressable` because Fabric touch
  hit testing may not follow the fast-path visual transform.
- `IOS_SYNCHRONOUSLY_UPDATE_UI_PROPS` is the matching iOS fast path in newer
  Reanimated 4.x versions; confirm the installed version before suggesting it.
- `DISABLE_COMMIT_PAUSING_MECHANISM` targets scroll/jitter issues but is safe
  only with React Native's `preventShadowTreeCommitExhaustion` support enabled
  in compatible RN versions.
- `USE_SYNCHRONIZABLE_FOR_MUTABLES` is intended to reduce shared-value read
  synchronization cost on the RN runtime.
- `USE_COMMIT_HOOK_ONLY_FOR_REACT_COMMITS`,
  `FORCE_REACT_RENDER_FOR_SETTLED_ANIMATIONS`, `USE_ANIMATION_BACKEND`, and
  `IOS_CSS_CORE_ANIMATION` are version-sensitive. Confirm availability and
  release notes/source before recommending them.
- Do not cargo-cult flags. Confirm current docs/source and record the native
  rebuild proof. Expo Go and other prebuilt Reanimated environments cannot
  change static feature flags.

## Hot Paths

- Prefer `transform` and `opacity`; layout, shadows, blur, SVG path morphing,
  large images, and many animated native views need measurement.
- Memoize gesture objects in list rows unless React Compiler or local patterns
  already prove stability.
- Keep list row animation simple. If many elements animate at once, reduce the
  effect, virtualize more aggressively, or move drawing-heavy visuals to a
  canvas strategy.
- Use release or `debugOptimized` builds for performance evidence.
- Reanimated docs cite rough simultaneous-animation limits: low-end Android can
  degrade around 100 animated components, while iOS tolerates more. Treat this
  as a measurement trigger, not a hard rule.

## Red Flags

- `runOnJS` or `scheduleOnRN` inside scroll/frame callbacks without throttling.
- `useFrameCallback` doing allocation or JS scheduling every frame.
- `useFrameCallback` or `Gesture.*` objects recreated every render in list rows.
- `withRepeat` without cancellation or reduced-motion handling.
- Transform fast paths applied to pressable/touchable targets without device
  touch proof.
- Feature flags changed without iOS/Android rebuild evidence.
- Animated text counters implemented with frequent React state or live-region
  updates instead of settled announcements or UI-thread/native text strategies.
