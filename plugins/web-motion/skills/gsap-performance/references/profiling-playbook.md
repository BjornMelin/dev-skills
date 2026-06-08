# GSAP profiling and evidence playbook

Skill: gsap-performance
Checked at: 2026-06-04

## When To Load

- Read when a GSAP issue is described as jank, dropped frames, layout thrash, or slow scroll.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.to()
- https://gsap.com/docs/v3/Plugins/ScrollTrigger/

## Reference Notes

- Start from evidence: browser performance profile, visible reproduction, affected route, device/browser, and animation trigger path.
- Classify costs as JavaScript, style recalculation, layout, paint, composite, image/decode, or scroll workload before suggesting an engine change.
- Use the audit CLI for leads, then verify each finding against runtime behavior.

## Focused Checks

- Capture before/after evidence for at least the hot interaction.
- Check frame budget under reduced CPU or representative device when the surface is user-facing.

## Failure Modes

- Replacing a library without first identifying the bottleneck.
- Treating `will-change` as a blanket fix.


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
