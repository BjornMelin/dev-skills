# Reanimated 4.1.5 Source Notes

Source inspected with `opensrc 0.7.2`:
`software-mansion/react-native-reanimated` tag `4.1.5`. Current Expo SDK 56
docs bundle Reanimated 4.3.1, so use these notes as source-pinned evidence for
the inspected version and refresh source for 4.3+ version-specific flags.

Use this file when official docs and installed source need to be reconciled.

## Source Findings

- `ReduceMotion` enum values in `src/commonTypes.ts` are `system`, `always`,
  and `never`.
- `withTiming`, `withSpring`, `withDelay`, `withRepeat`, `withSequence`, and
  decay utilities carry `reduceMotion` through animation config paths.
- `useReducedMotion()` reads the system setting once at module initialization
  and returns that startup value.
- `ReducedMotionConfig` sets `ReducedMotionManager` globally while mounted and
  restores the previous setting during cleanup.
- Reanimated `src/workletFunctions.ts` re-exports worklet APIs from
  `react-native-worklets`; the `runOnJS` re-export is marked deprecated in
  favor of direct `react-native-worklets` import.
- `react-native-worklets` docs in the same source tree mark `runOnJS` as
  deprecated in favor of `scheduleOnRN`.
- Reanimated 4.1.5 `staticFlags.json` includes
  `DISABLE_COMMIT_PAUSING_MECHANISM`,
  `ANDROID_SYNCHRONOUSLY_UPDATE_UI_PROPS`,
  `EXPERIMENTAL_CSS_ANIMATIONS_FOR_SVG_COMPONENTS`, and
  `USE_SYNCHRONIZABLE_FOR_MUTABLES`, all defaulting to `false`.

## Audit Implications

- Prefer current docs for public API wording, but use installed source to
  validate deprecations and default flags.
- Do not assume a feature flag exists unless the installed package contains it.
  In particular, Reanimated 4.2+ and 4.4+ docs list flags that are absent from
  the 4.1.5 source snapshot.
- Do not replace all `runOnJS` blindly. First check whether the app has
  `react-native-worklets` with `scheduleOnRN`; then migrate the specific
  UI-runtime-to-RN-runtime scheduling call.
- If the target app is Expo SDK 56, check the installed package source before
  making assertions about 4.3.1 defaults or later Reanimated docs.
