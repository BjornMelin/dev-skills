# ScrollTrigger and scroll workload review

Skill: gsap-performance
Checked at: 2026-06-04

## When To Load

- Use this for pinned scenes, scrubbed timelines, refresh timing, and scroll callback costs.


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
