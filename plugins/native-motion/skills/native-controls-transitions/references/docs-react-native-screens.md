# React Native Screens Notes

Use this when Expo Router/native-stack behavior depends on native screen
containers, back gestures, hardware back, or native-stack lifecycle.

## Role In This Skill

- Expo Router Stack and React Navigation native-stack rely on native screen
  primitives for route containers and platform gestures.
- Treat route push/pop, modal dismissal, and back gestures as native-stack/RN
  screens behavior unless the app has explicitly opted into custom navigation.
- Do not replace RN screens gestures with a full-screen Reanimated gesture just
  to block or customize back navigation.

## Review Points

- Confirm `react-native-screens` is installed at the Expo-compatible version
  before changing stack assumptions.
- Keep `beforeRemove`, transition listeners, and guarded navigation scoped to
  the affected screen.
- Validate iOS edge swipe, Android hardware back, accessibility escape/back
  actions, modal dismissal, and deep-link restoration.
- Check interrupted gestures: partial back swipe cancel, rapid double back,
  route replacement, and unmount during a transition.
- Keep expensive work out of transition frames; listener callbacks should
  coordinate state, not run heavy work synchronously.

## Boundary

- RN screens/native-stack owns screen lifecycle and route movement.
- Reanimated may own content after the route is mounted, such as row
  enter/exit/reorder or inline expansion.
- If a custom navigator, custom header, or experimental stack is already in
  place, inspect installed types and current docs before assuming native-stack
  options are supported.
