# Expo Router Stack Notes

Use this when work touches route-level transitions, native headers, search bars,
toolbars, back buttons, stack presentations, or transition listeners.

## Current Shape

- `Stack` renders a native stack navigator and accepts React Navigation native
  stack options through `Stack.Screen options`.
- `Stack.Header`, `Stack.Title`, `Stack.SearchBar`, `Stack.Screen.BackButton`,
  and `Stack.Toolbar` are composition helpers that write native-stack options.
  If multiple instances configure the same screen, the last rendered one wins.
- `Stack.SearchBar` integrates with the native header and makes the header
  visible.
- `Stack.Toolbar` is alpha. It is available on iOS in Expo SDK 55+ and Android
  in Expo SDK 56+. It renders only on Android and iOS; web needs an app-owned
  fallback.

## Transition Ownership

- Use `Stack.Screen options` for screen-level `animation`, `presentation`,
  gestures, headers, and sheet options.
- Use navigation listeners such as `transitionStart`, `transitionEnd`,
  `gestureCancel`, and `sheetDetentChange` for route lifecycle coordination.
- Do not use component-local Reanimated motion to simulate push/pop, modal,
  form-sheet, or native back-gesture behavior.
- If using Expo Router's experimental stack, verify current support first. SDK
  56 source warns that custom headers, presentation, animation, sheets, and
  status bar options are not all available there.
- Prefer `animation: 'none'` when disabling native-stack animation. Treat
  `animationEnabled` as stale unless installed types prove otherwise.
- Keep route-shell movement in the navigator even when screen content uses
  Reanimated layout/entering/exiting after mount.

## Toolbar Rules

- `placement="left"` and `placement="right"` render in the header and force the
  header visible.
- Bottom toolbar placement belongs in page components, not layout components.
- Android spacers need explicit `width`; iOS can use flexible spacers.
- SF Symbols are iOS-only. Android icons must be image sources, commonly
  Material Symbols XML assets. Branch icon values by platform.
- Badges and label primitives are iOS header-placement features; Android drops
  label/badge children for toolbar buttons.
- `Stack.Toolbar.SearchBarSlot` is iOS-only; use `Stack.SearchBar` for
  cross-platform native header search.
- Put menu action labels in children, not a `title` prop.
- Icon-only toolbar actions need an accessible label or an adjacent visible
  label supported by the platform.

## Verification

Check both platforms for header visibility, icon tinting, dark mode, rapid
navigation, back gestures, modal dismissal, keyboard overlap, large-title
collapse, and web fallback if the route is universal.
