# Official GreenSock skill source and license gate

Skill: gsap-core
Checked at: 2026-06-04

## When To Load

- Use this when verifying which upstream GSAP skill text was copied and how the MIT skill license differs from the GSAP package license.


## Operating Guidance

Core GSAP tweens, transforms, eases, staggers, matchMedia, accessibility, and DOM/SVG tween review.

### Decision Boundaries

- Use CSS for simple declarative state transitions.
- Use gsap-timeline for multi-step choreography.
- Use gsap-react when React lifecycle owns the target nodes.

### Workflow Details

1. Inspect installed GSAP version and framework ownership.
2. Prefer official transform aliases and explicit eases/durations.
3. Add matchMedia or reduced-motion handling for nonessential motion.
4. Run the audit CLI and verify findings manually.

### Gotchas

- Do not animate raw transform strings when GSAP aliases express the same effect.
- Set immediateRender intentionally when stacking from/fromTo tweens.
- Store returned tween handles when playback control or cleanup is needed.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
