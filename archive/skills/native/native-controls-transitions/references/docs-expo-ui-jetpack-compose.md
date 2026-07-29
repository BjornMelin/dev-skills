# Expo UI Jetpack Compose Notes

Use this for Android-only Expo UI work that needs Jetpack Compose controls,
Material 3 behavior, modifiers, or hosting behavior not exposed by universal
`@expo/ui`.

## Imports And Boundary

- Import components from `@expo/ui/jetpack-compose`.
- Import modifiers from `@expo/ui/jetpack-compose/modifiers`.
- Wrap every Compose subtree in `Host`.
- Use `RNHostView` only to embed React Native components inside a Compose tree.
  Keep it small and validate touch, focus, accessibility, and layout.

## Host Rules

- `Host` bridges React Native and Jetpack Compose.
- Use `matchContents` for intrinsic-size controls.
- Do not use `matchContents` on the same axis as `LazyRow`, `LazyColumn`,
  `Carousel`, or horizontal/vertical scroll modifiers. Compose scrollables need
  finite max constraints on the scroll axis.
- Use `style` for finite width/height or `flex: 1` when content should fill
  available space.
- `ignoreSafeAreaKeyboardInsets` is a mount-time keyboard avoidance decision.
- `seedColor` can seed a Material 3 palette for descendants. Treat it as theming
  policy, not a one-off styling tweak.

## Transition Boundary

- Prefer Compose controls for Android-native behavior.
- Use Compose sheet/dialog/menu primitives before implementing custom
  Reanimated clones.
- When using a Compose-only control, provide an iOS implementation or fallback
  instead of making Android the hidden source of truth.
