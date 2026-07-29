# Tween interruption and overwrite patterns

Skill: gsap-core
Checked at: 2026-06-04

## When To Load

- Read when tweens can overlap, replay, reverse, or leave inline styles behind.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.to()
- https://gsap.com/docs/v3/GSAP/Timeline/

## Reference Notes

- Store returned Tween handles when a user interaction, route transition, or component cleanup must pause, reverse, kill, or inspect progress.
- Use overwrite intent deliberately. `overwrite: "auto"` is useful for overlapping active tweens on the same target, but it is not a substitute for one clear animation owner.
- Use `clearProps` only when CSS classes or external state should regain ownership after the tween completes.

## Focused Checks

- Find multiple tweens targeting the same element and property and identify which one owns interruption behavior.
- Verify rapid toggles, replay, route unmount, and final inline styles.

## Failure Modes

- Delay chains that should be a timeline.
- Fire-and-forget tweens in event handlers with no handle or cleanup path.


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
