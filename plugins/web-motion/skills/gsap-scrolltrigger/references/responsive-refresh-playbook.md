# Responsive refresh and invalidation playbook

Skill: gsap-scrolltrigger
Checked at: 2026-06-04

## When To Load

- Read when ScrollTrigger scenes depend on images, fonts, breakpoints, async content, or route-level layout shifts.

## Source Anchors

- https://gsap.com/docs/v3/Plugins/ScrollTrigger/
- https://gsap.com/docs/v3/GSAP/gsap.matchMedia()

## Reference Notes

- Use refresh/invalidation as an ordered response to layout changes, not a random timeout. Fonts, images, CMS content, accordions, and virtualized lists all need explicit reasoning.
- Responsive scenes should be created and reverted through matchMedia or framework cleanup.
- Pinned scenes need extra proof because pin spacing mutates page geometry.

## Focused Checks

- Test reload, resize, route navigation, late image/font load, and content expansion.
- Enable markers temporarily during development, then remove them before closeout.

## Failure Modes

- Leaving `markers: true` in production code.
- Nested ScrollTriggers inside timeline child tweens.


## Operating Guidance

GSAP ScrollTrigger scroll-linked animation, pinning, scrub, trigger callbacks, refresh/invalidation, responsive matchMedia scenes, and cleanup.

### Decision Boundaries

- Use CSS scroll-driven animations only when native browser support and declarative semantics fit.
- Use gsap-timeline for non-scroll sequencing.
- Use web-three-r3f for 3D scroll scenes.

### Workflow Details

1. Identify scroll container, trigger, start/end, pin, scrub, and responsive ownership.
2. Attach ScrollTrigger to a top-level tween or timeline.
3. Plan refresh ordering after fonts/images/layout changes.
4. Verify resize, route unmount, reduced motion, and mobile scroll.

### Gotchas

- Do not put ScrollTriggers inside child tweens of a nested timeline.
- Pinned scenes affect layout and need refresh proof.
- Route transitions must kill/revert triggers or scope them to a context.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
