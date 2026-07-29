# Reanimated 4 Migration And Testing Notes

Use this when a task touches Reanimated 3-to-4 migration, worklet setup, or
unit tests. Prefer installed package versions and current official docs when
they disagree with these notes.

## Current Reanimated 4 Setup

- Reanimated 4 is New Architecture-only. Apps still on Legacy Architecture
  should stay on Reanimated 3 until the architecture migration is in scope.
- Expo SDK 56 targets React Native 0.85 and bundles
  `react-native-reanimated` 4.3.1. Use Expo-compatible install/fix commands in
  Expo repos instead of blindly installing latest npm releases.
- For the Expo SDK 56 baseline, `react-native-reanimated` 4.3.1 expects
  `react-native-worklets` 0.8.x; `expo@56.0.8` package metadata lists
  `react-native-worklets` 0.8.3. Latest upstream Reanimated compatibility docs
  may include newer Reanimated/Worklets pairs, so do not apply latest ranges to
  an Expo SDK 56 app unless the app is intentionally opting out of the Expo
  bundle.
- Reanimated 4 requires a compatible `react-native-worklets` package. Expo
  projects should install both through the repo's Expo-compatible package
  command and rebuild native dependencies when required.
- Reanimated 3 should not have `react-native-worklets` installed. Treat that
  combination as a setup bug until proven otherwise.
- Expo configures the Worklets Babel plugin through `babel-preset-expo` when
  Reanimated is installed. RN CLI apps should use
  `react-native-worklets/plugin` as the last Babel plugin.
- Replace the old `react-native-reanimated/plugin` path during Reanimated 4
  setup or migration.

## Threading And Migration

- Import worklet-specific threading APIs from `react-native-worklets` in
  Reanimated 4 code.
- Replace `runOnJS(fn)(...args)` with `scheduleOnRN(fn, ...args)` when the app
  is on Reanimated 4. Keep the scheduled function defined in RN-runtime scope,
  such as component or module scope, not inside a UI-runtime callback.
- Official Reanimated 4 migration docs describe old worklet functions as moved
  to `react-native-worklets`, re-exported for compatibility, deprecated, and
  planned for removal. Software Mansion's copied animation skill slices are
  stricter and say never to use `runOnJS`; use that strict rule for new
  Reanimated 4 work, but do not file Reanimated 3 maintenance findings solely
  because `runOnJS` exists.
- Replace `runOnUI`, `runOnRuntime`, and `executeOnUIRuntimeSync` with the
  matching `scheduleOnUI`, `scheduleOnRuntime`, and `runOnUISync` APIs when
  migrating to Reanimated 4.
- Replace removed `useAnimatedGestureHandler` usages with Gesture Handler 2's
  `Gesture` API.
- Replace deprecated `useScrollViewOffset` usages with `useScrollOffset` unless
  the installed Reanimated version requires the older name.
- Re-check `withSpring` configs from Reanimated 3. `restDisplacementThreshold`
  and `restSpeedThreshold` were replaced by `energyThreshold`, and duration
  semantics changed in Reanimated 4.

## Reduced Motion

- `useReducedMotion()` reads the system setting synchronously but reports the
  value from app startup; it does not rerender when the system setting changes.
- Use `ReduceMotion.System` as the default animation config. Use
  `ReduceMotion.Always` for decorative motion and `ReduceMotion.Never` only
  when motion is essential and has no static equivalent.
- Use React Native `AccessibilityInfo` or an app-owned accessibility setting
  when UI must respond to mid-session reduced-motion changes.

## Jest

- Follow the target repo's existing Jest setup first.
- Reanimated's Jest guide uses
  `require('react-native-reanimated').setUpTests()` in the Jest setup file.
- Use fake timers (`jest.useFakeTimers()`, `jest.advanceTimersByTime(...)`) for
  time-based animation assertions.
- If tests touch `react-native-svg` animated props, check whether the repo needs
  Reanimated's SVG mock.

## Upstream Sources

- https://docs.swmansion.com/react-native-reanimated/docs/guides/migration-from-3.x/
- https://docs.swmansion.com/react-native-reanimated/docs/guides/compatibility/
- https://docs.swmansion.com/react-native-reanimated/docs/guides/worklets/
- https://docs.swmansion.com/react-native-worklets/docs/threading/scheduleOnRN/
- https://docs.swmansion.com/react-native-reanimated/docs/device/useReducedMotion/
- https://docs.swmansion.com/react-native-reanimated/docs/guides/testing/
- https://docs.expo.dev/versions/latest/
- https://docs.expo.dev/versions/latest/sdk/reanimated/
- https://docs.expo.dev/guides/new-architecture/
