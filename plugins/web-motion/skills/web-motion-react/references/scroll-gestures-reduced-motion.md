# Scroll, gesture, and reduced-motion hooks

Skill: web-motion-react
Checked at: 2026-06-04

## When To Load

- Read for useScroll, useTransform, whileHover/tap/drag, and useReducedMotion.


## Operating Guidance

Motion React components and hooks: motion, AnimatePresence, layout animations, useScroll, useReducedMotion, gestures, variants, and React/Next boundaries.

### Decision Boundaries

- Use GSAP for imperative timelines and plugin-heavy scenes.
- Use CSS for simple static transitions.
- Use WAAPI for low-level Animation object control outside React.

### Workflow Details

1. Confirm package import path and React/client boundary.
2. Choose presence, layout, gesture, scroll, or value-based motion deliberately.
3. Respect reduced motion and state ownership.
4. Verify layout projection with resize, interruption, route changes, and hydration.

### Gotchas

- AnimatePresence requires stable keys and actual unmounts.
- Layout animations depend on stable layout boxes and should not fight CSS transitions.
- Do not push high-frequency motion values through React state.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
