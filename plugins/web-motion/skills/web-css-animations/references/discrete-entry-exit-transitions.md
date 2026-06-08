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


## Operating Guidance

Browser CSS transitions, keyframes, scroll-driven animations, registered properties, discrete transitions, reduced motion, and performance-safe CSS motion.

### Decision Boundaries

- Use CSS first for two-state UI motion.
- Move to WAAPI when an Animation object or seeking is needed.
- Move to GSAP for complex imperative choreography.

### Workflow Details

1. Identify the state driver and animated properties.
2. Use explicit transition-property lists and product motion tokens.
3. Add reduced-motion behavior beside the motion.
4. Guard new CSS with @supports or local browser policy.

### Gotchas

- transition: all hides expensive accidental properties.
- Unregistered custom properties animate discretely.
- animation shorthand resets animation-timeline, so set timeline after shorthand.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
