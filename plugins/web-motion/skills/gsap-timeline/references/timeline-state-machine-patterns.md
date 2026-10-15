# Timeline state-machine and playback patterns

Skill: gsap-timeline
Checked at: 2026-06-04

## When To Load

- Read when a timeline is controlled by app state, route state, media queries, or user playback controls.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/Timeline/
- https://gsap.com/docs/v3/GSAP/gsap.context()

## Reference Notes

- Keep the authoritative state in the app or the timeline, not both. A UI toggle should map clearly to play, reverse, seek, progress, or rebuild.
- Use labels and progress values for deterministic tests and reproducible bug reports.
- When timeline state depends on layout, rebuild at the same boundary where layout ownership changes.

## Focused Checks

- Check rapid toggle behavior and final visual state.
- Verify cleanup kills or reverts the timeline and owned child tweens.

## Failure Modes

- Mixing CSS transitions on the same properties controlled by timeline children.
- Recreating timelines every render while keeping old handles alive.


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
