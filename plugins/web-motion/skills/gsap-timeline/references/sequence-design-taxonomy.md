# Timeline sequence design taxonomy

Skill: gsap-timeline
Checked at: 2026-06-04

## When To Load

- Read before converting delay chains, staggered entrances, or product choreography into timelines.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/Timeline/
- https://gsap.com/docs/v3/GSAP/gsap.to()

## Reference Notes

- Model the sequence as named phases, labels, and relative positions. Labels document product intent and make later edits safer.
- Use timeline defaults for repeated duration/ease values and explicit child vars only when they differ.
- A timeline should own order; individual child tweens should not smuggle independent delays that make the sequence hard to reason about.

## Focused Checks

- Inspect whether labels map to meaningful UI phases.
- Verify replay, reverse, seek, pause, and interruption semantics if exposed to users.

## Failure Modes

- Constructor `duration` used as if it controlled every child tween.
- Long delay chains instead of position parameters.


## Operating Guidance

GSAP timelines, sequencing, position parameter, labels, nesting, playback controls, defaults, and timeline review.

### Decision Boundaries

- Use gsap-core for one-off tweens.
- Use gsap-scrolltrigger when scroll owns the playhead.
- Use CSS keyframes only for simple fixed loops.

### Workflow Details

1. Model the sequence as labels, relative positions, and defaults.
2. Use position parameters instead of delay chains.
3. Store timeline handles when playback control or cleanup is needed.
4. Verify interruptions and reverse/restart behavior.

### Gotchas

- Timeline constructor duration is not child tween duration.
- Nested ScrollTriggers are usually wrong; attach scroll control to the top-level tween/timeline.
- Labels are a maintainability tool, not just comments.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
