# Expo SDK 56 Animation Notes

Sources: Expo SDK 56 reference, SDK 56 changelog, Expo Reanimated page, Expo UI
`useNativeState`, and Expo Animation docs, checked during the 2026-06-04 audit
pass.

Use this file when the target app is Expo-managed or depends on Expo SDK
version compatibility.

## SDK 56 Baseline

- Expo SDK 56 targets React Native 0.85, React 19.2, Android 7+, Android API
  level 36, iOS 16.4+, and Xcode 26.4+.
- The Expo SDK 56 Reanimated page bundles `react-native-reanimated` 4.3.1 and
  installs it with `react-native-worklets`.
- SDK 56 includes Expo UI stable APIs. Expo UI `useNativeState` can update
  SwiftUI/Jetpack Compose state synchronously from a worklet, but it still
  depends on the app installing Reanimated and Worklets for those UI-thread
  paths.
- SDK 56 prebuilt artifacts can speed native builds for common libraries such
  as Reanimated, but prebuilt environments also affect whether static feature
  flags can be changed.

## Expo Guidance

- Expo projects can use React Native Animated for basic animation.
- Expo docs recommend `react-native-reanimated` for advanced, smoother, and
  maintainable animation.
- Default Expo templates may already include Reanimated. Existing projects
  should install Expo-compatible versions through the repo's Expo command
  policy, usually `expo install` through the repo package-manager wrapper.
- In SDK 56, do not hand-pick latest Reanimated/Worklets versions unless the
  app is intentionally leaving Expo's compatible set and the native runtime is
  rebuilt and tested.

## Audit Implications

- Do not recommend npm latest for Expo apps. Verify the installed Expo SDK and
  Expo-compatible package versions.
- Reanimated setup changes are native-risk. Run the repo's Expo doctor and a
  native runtime smoke when Babel/plugin/native build inputs change.
- Reanimated static feature flags are native build inputs and are not mutable
  in Expo Go or other prebuilt Reanimated environments.
- Expo UI worklet-native-state changes need the same accessibility review as
  Reanimated motion: reduced-motion behavior, focus stability, text input
  cursor behavior, screen-reader output, and platform proof.
- If the app uses Expo web too, verify reduced-motion behavior on native and web
  separately; APIs and haptics differ by platform.
