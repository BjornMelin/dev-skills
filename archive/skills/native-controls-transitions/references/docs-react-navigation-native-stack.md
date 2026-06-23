# React Navigation Native Stack Notes

Expo Router Stack delegates many screen options to React Navigation's native
stack, so use these notes when `Stack.Screen options` are the implementation
surface.

## Transition Options

- Use `animation` for native-stack screen movement. Common values include
  `default`, `fade`, `fade_from_bottom`, `fade_from_right`,
  `reveal_from_bottom`, `scale_from_center`, `slide_from_right`,
  `slide_from_left`, `slide_from_bottom`, `flip`, `simple_push`, and `none`.
- Use `presentation` for route semantics such as card, modal, transparent modal,
  contained modal, full-screen modal, form sheet, and related platform
  presentations.
- Use `animation: 'none'` instead of older `animationEnabled: false` patterns.
- Some native transitions and presentations do not allow custom durations. Do
  not promise exact timing parity across platforms.
- Confirm option support against installed Expo Router/native-stack types when
  working with SDK-sensitive sheet, gesture, or presentation options.

## Events And Coordination

- Listen for `transitionStart` and `transitionEnd` when work must be coordinated
  with screen movement.
- Listen for gesture lifecycle events such as `gestureCancel` when custom state
  depends on an interrupted back gesture.
- Keep expensive work out of transition frames. Trigger data or side effects
  from lifecycle events only when the UX requires it.

## Boundary

- Native-stack transitions should own the full screen shell.
- Reanimated should own content inside the route after the route transition is
  settled, or independent inline content transitions.
- If a screen blocks back navigation, validate hardware back, iOS edge swipe,
  accessibility escape/back actions, and deep-link restoration.
- If component-local Reanimated motion shares the same file as native-stack
  options, verify it targets content after mount and not the route shell.
