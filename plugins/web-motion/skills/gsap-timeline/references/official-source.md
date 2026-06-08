# Official GreenSock timeline skill source

Skill: gsap-timeline
Checked at: 2026-06-04

## When To Load

- Use this to verify copied upstream timeline behavior.


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
