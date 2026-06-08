# CSS transition/keyframe field guide

Skill: web-css-animations
Checked at: 2026-06-04

## When To Load

- Read before implementing ordinary CSS state motion.


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
