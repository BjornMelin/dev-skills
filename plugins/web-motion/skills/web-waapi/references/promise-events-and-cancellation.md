# Animation promises, events, and cancellation

Skill: web-waapi
Checked at: 2026-06-04

## When To Load

- Read when WAAPI code awaits `finished`, reacts to `finish`/`cancel`, or coordinates interruption.

## Source Anchors

- https://developer.mozilla.org/docs/Web/API/Web_Animations_API/Using_the_Web_Animations_API
- https://developer.mozilla.org/docs/Web/API/Animation/commitStyles

## Reference Notes

- WAAPI animations have lifecycle state. Treat `finished` promises, event listeners, cancellation, and route/component teardown as first-class control flow.
- A cancelled animation and a finished animation are different outcomes; cleanup and final styles should reflect that distinction.
- Avoid dangling promises that update UI after the owner unmounts.

## Focused Checks

- Test cancel, finish, reverse, rapid restart, and unmount.
- Check that event listeners are removed or tied to the owner lifecycle.

## Failure Modes

- Awaiting `animation.finished` without handling cancellation.
- Calling `commitStyles()` after a cancelled animation without verifying desired final state.


## Operating Guidance

Browser Web Animations API: Element.animate(), Animation, KeyframeEffect, playback control, generated keyframes, cancel/finish, commitStyles, and cleanup.

### Decision Boundaries

- Use CSS for simple state transitions.
- Use Motion/GSAP when framework state or timelines dominate.
- Use WAAPI when code needs an Animation object, seeking, cancellation, or generated keyframes.

### Workflow Details

1. Check browser support and local fallback policy.
2. Create keyframes/options with explicit duration, fill, easing, and composite behavior.
3. Own animation cancellation and finish behavior.
4. Verify rapid interruptions, route unmount, reduced motion, and commitStyles usage.

### Gotchas

- commitStyles persists computed styles and should be followed by cancel when appropriate.
- fill: forwards can retain stacking/style side effects.
- Multiple animations on the same property need composite/replace intent.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
