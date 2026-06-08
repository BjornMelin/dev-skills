# Scroll-driven and view-timeline CSS motion

Skill: web-css-animations
Checked at: 2026-06-04

## When To Load

- Read when using `animation-timeline`, `scroll()`, `view()`, timeline ranges, or CSS scroll-driven animation instead of JavaScript scroll handlers.

## Source Anchors

- https://developer.mozilla.org/docs/Web/CSS/CSS_Animations/Using_CSS_animations
- https://developer.mozilla.org/docs/Web/CSS/CSS_Transitions/Using_CSS_transitions

## Reference Notes

- Use native scroll-driven animation when declarative support matches the browser policy and the effect does not need imperative playback state.
- Feature-detect newer syntax with `@supports` and route unsupported browsers to a static or simpler fallback.
- Keep scroll effects nonessential unless accessibility and keyboard/assistive navigation have been verified.

## Focused Checks

- Check browser support policy and a reduced-motion branch.
- Verify behavior with keyboard scroll, mobile scroll, and content resizing.

## Failure Modes

- Using scroll-driven CSS for critical state changes that need ARIA/state synchronization.
- Combining CSS scroll timelines and JS scroll libraries on the same element without an owner.


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
