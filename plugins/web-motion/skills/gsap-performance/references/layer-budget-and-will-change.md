# Layer budget and will-change discipline

Skill: gsap-performance
Checked at: 2026-06-04

## When To Load

- Read before adding transform hacks, force3D, will-change, or GPU promotion advice.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.to()
- https://developer.mozilla.org/docs/Web/CSS/CSS_Transitions/Using_CSS_transitions

## Reference Notes

- Promote only hot animated elements and only for the lifetime of the effect when possible.
- Large layers, filters, shadows, video, and text rendering can shift cost from layout to memory/compositing instead of making it free.
- Use transform/opacity as the default fast path, but measure paint-heavy visual effects.

## Focused Checks

- Search for persistent `will-change` on many elements.
- Review memory, layer count, text clarity, and paint cost when adding GPU promotion.

## Failure Modes

- Global CSS that sets `will-change` across component classes.
- Animating filter or shadow while assuming transform-like compositing cost.


## Operating Guidance

GSAP performance audits: transform/opacity, layout thrash, ScrollTrigger batching, repeat loops, will-change, and frame pressure.

### Decision Boundaries

- Use web-three-r3f or typegpu for GPU/canvas rendering performance.
- Use web-css-animations for CSS-only transition audits.
- Use gsap-scrolltrigger for scroll scene semantics.

### Workflow Details

1. Find hot paths and animated properties.
2. Classify layout, paint, composite, and JavaScript costs.
3. Run audit scan and inspect high-confidence findings.
4. Recommend transform/opacity, batching, throttling, or engine changes only with evidence.

### Gotchas

- will-change is a scoped hint, not a global optimization.
- ScrollTrigger refresh calls after layout changes need ordering, not random timeouts.
- Infinite tweens need reduced-motion and cleanup behavior.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
