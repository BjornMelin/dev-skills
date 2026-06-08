# Discrete entry and exit transitions

Skill: web-css-animations
Checked at: 2026-06-04

## When To Load

- Read when animating `display`, `content-visibility`, popovers, dialogs, `@starting-style`, or `transition-behavior`.

## Source Anchors

- https://developer.mozilla.org/docs/Web/CSS/transition-behavior
- https://developer.mozilla.org/docs/Web/CSS/CSS_Transitions/Using_CSS_transitions

## Reference Notes

- Discrete transitions need explicit support reasoning. `@starting-style` handles entry setup; `transition-behavior: allow-discrete` handles discrete property transitions where supported.
- Do not let visual entry/exit become the only source of open/closed state. DOM state, focus, and ARIA still need to be correct.
- Prefer simple opacity/transform transitions when discrete support is outside the repo browser policy.

## Focused Checks

- Test first render, reopen, close, focus return, and reduced motion.
- Check dialog/popover semantics separately from animation state.

## Failure Modes

- `transition: all` around `display` and layout properties.
- Hiding focusable content visually while leaving it reachable.

## Review Steps

1. Confirm `@starting-style` and `transition-behavior: allow-discrete` are supported or guarded.
2. Keep open/closed ownership in DOM state, not only in visual animation state.
3. Test keyboard focus, focus return, and reduced-motion behavior during entry and exit.
4. Prefer opacity and transform when discrete transition support is outside the target browser policy.
