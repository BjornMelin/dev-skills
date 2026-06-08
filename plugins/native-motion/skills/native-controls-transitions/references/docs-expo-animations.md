# Animation Boundary Notes

Use this when deciding whether movement belongs to Expo Router/native controls
or Reanimated.

## Native-Owned

Keep these in the navigator or native control layer:

- stack push/pop and back gestures
- modal, page sheet, form sheet, and native sheet presentation
- native header title/search/back-button transitions
- toolbar/menu open, close, and item affordances
- switch, slider, picker, segmented-control, date/time picker, and menu feedback

## App-Owned

Use Reanimated when motion is part of the screen content:

- row/card enter, exit, reorder, or inline expand/collapse
- product-specific disclosure, progress, drag, or gesture behavior
- scroll-driven content effects that do not replace native header behavior
- optimistic state transitions that need interrupt/cancel handling

## Accessibility And Performance

- Respect reduced motion for custom movement. Reanimated exposes
  `ReduceMotion.System`, `.reduceMotion(...)`, and `useReducedMotion()`.
- Prefer transforms and opacity for hot paths.
- Avoid per-frame React state updates.
- List layout animations need narrow validation; Reanimated list
  `itemLayoutAnimation` is single-column only.
- Never layer a Reanimated spatial transition on top of a native stack
  transition for the same screen.
- Keep route lifecycle side effects outside hot transition frames. Use native
  transition events for coordination and Reanimated completion callbacks only
  for the content animation they own.
- Validate interrupted custom content motion: rapid repeats, route unmount,
  back gesture cancel, and reduced-motion changes where the app observes them.
