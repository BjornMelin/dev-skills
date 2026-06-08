# Expo UI Notes

Use this when deciding whether `@expo/ui` should own a native control, sheet, or
platform UI subtree.

## Selection

- Use Expo UI universal components when one component tree should run on
  Android, iOS, and web while preserving platform-native controls on mobile.
- Use `@expo/ui/swift-ui` or `@expo/ui/jetpack-compose` directly when the
  universal API lacks a platform-specific control, modifier, or behavior.
- Use React Native built-ins when they already provide the native control with
  less dependency or hosting overhead.
- Use Reanimated for product-specific content motion, not for cloning controls
  that Expo UI or the platform already provides.

## Universal Components

Universal `@expo/ui` components delegate to Jetpack Compose on Android, SwiftUI
on iOS, and web/RN implementations on web. Current categories include:

- container: `Host`
- layout: `Column`, `Row`, `Spacer`, `ScrollView`
- display: `Text`, `Icon`
- controls: `Button`, `Switch`, `Checkbox`, `Slider`, `TextInput`, `Picker`
- presentation: `BottomSheet`, `Collapsible`
- collections/forms: `List`, `ListItem`, `FieldGroup`

Every universal subtree needs `Host` imported from `@expo/ui`.

## Host Constraints

- `Host` wraps native platform UI on Android/iOS and a view-like fallback on web.
- `matchContents` is useful for intrinsic controls. Do not use it on the same
  axis as scrollable content; give scrollable hosts finite size.
- `ignoreSafeArea`, `layoutDirection`, `colorScheme`, and viewport measurement
  are boundary decisions. Validate them with keyboard, safe areas, RTL, dark
  mode, and dynamic type.
- Installed package types are the final source when docs and local packages
  disagree.
- Keep one obvious Host owner per native UI subtree. Avoid wrapping a whole
  screen in a Host when only a leaf control needs native ownership.
- Do not silently drop platform-specific props from a shared wrapper; make
  Android/iOS/web branches visible when parity is not available.

## Bottom Sheets

- Universal `BottomSheet` is controlled by `isPresented` and `onDismiss`.
- Use semantic `snapPoints` such as `'half'` and `'full'` for parity. Android
  maps precise `{ fraction }` and `{ height }` forms to its limited Material
  states.
- Wrap tall content in `ScrollView`.
- If migrating from a community bottom sheet, check whether Expo UI's drop-in
  replacement supports the used API before replacing call sites.
- Validate dismissal gestures, detent changes, keyboard overlap, safe area, and
  scroll handoff on both Android and iOS.
