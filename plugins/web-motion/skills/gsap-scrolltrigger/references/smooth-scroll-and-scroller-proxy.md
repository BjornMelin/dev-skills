# Smooth-scroll and scroller proxy boundary

Skill: gsap-scrolltrigger
Checked at: 2026-06-04

## When To Load

- Read when ScrollTrigger is combined with Lenis, Locomotive, custom scroll containers, overflow panels, or transformed parents.

## Source Anchors

- https://gsap.com/docs/v3/Plugins/ScrollTrigger/
- https://gsap.com/docs/v3/GSAP/gsap.matchMedia()

## Reference Notes

- Identify the real scroll owner before adding triggers. Window scroll, nested overflow containers, and smooth-scroll libraries have different refresh and proxy requirements.
- Keep smooth-scroll integration centralized; do not configure proxy behavior ad hoc in individual components.
- Transformed ancestors, pinned elements, and custom scrollers change measurement assumptions.

## Focused Checks

- Verify trigger positions after resize, content load, route transition, and smooth-scroll start/stop.
- Check mobile touch scroll, keyboard scroll, and reduced-motion behavior.

## Failure Modes

- Multiple components each creating their own smooth-scroll proxy.
- Pins inside transformed containers without geometry proof.


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
