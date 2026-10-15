# Responsive and reduced-motion GSAP patterns

Skill: gsap-core
Checked at: 2026-06-04

## When To Load

- Read when GSAP motion depends on breakpoints, user preference, or nonessential decorative motion.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.matchMedia()
- https://agentskills.io/skill-creation/best-practices

## Reference Notes

- Prefer `gsap.matchMedia()` for breakpoint-specific setup because animations created in matching callbacks are collected and reverted with the media context.
- Treat reduced motion as a behavioral branch, not just a shorter duration. Decorative movement can be skipped; functional feedback should remain visible and understandable.
- Keep focus, pointer, and visibility semantics aligned with `autoAlpha`, display changes, and hidden states.

## Focused Checks

- Check `(prefers-reduced-motion: reduce)` handling and manual product motion toggles.
- Verify resize across breakpoints and cleanup with `mm.revert()` or framework lifecycle cleanup.

## Failure Modes

- Animating off-screen entrances for reduced-motion users without a static final state.
- Duplicating matchMedia and context ownership around the same DOM nodes.


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
