# Expo UI SwiftUI Notes

Use this for iOS/tvOS-only Expo UI work that needs SwiftUI controls, modifiers,
or hosting behavior not exposed by universal `@expo/ui`.

## Imports And Boundary

- Import components from `@expo/ui/swift-ui`.
- Import modifiers from `@expo/ui/swift-ui/modifiers`.
- Wrap every SwiftUI subtree in `Host`.
- Use `RNHostView` only to embed React Native components inside a SwiftUI tree.
  Keep that boundary small and validate touch, focus, layout, and accessibility.

## Host Rules

- `Host` is a React Native `View` that hosts SwiftUI through UIKit.
- Use `matchContents` for intrinsic-size controls such as simple buttons/text.
- Do not use `matchContents` on the same axis as `ScrollView`, `List`, `Form`,
  `LazyHStack`, or `LazyVStack`. Use finite size through `style` or per-axis
  matching.
- Use `style={{ flex: 1 }}` or explicit dimensions when the SwiftUI content
  should fill available space.
- `ignoreSafeArea="keyboard"` is useful when React Native already owns keyboard
  avoidance; `ignoreSafeArea="all"` is for intentional edge-to-edge surfaces.
  These settings can only be set once on mount.

## Transition Boundary

- Prefer SwiftUI controls for native fidelity and OS behavior.
- Do not mix SwiftUI transitions and Reanimated on the same visual element.
  Pick one owner and validate interruption/unmount behavior.
- For platform-only controls, provide a matching Android implementation or an
  explicit fallback rather than forcing visual parity.
