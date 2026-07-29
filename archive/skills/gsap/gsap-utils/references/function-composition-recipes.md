# gsap.utils function composition recipes

Skill: gsap-utils
Checked at: 2026-06-04

## When To Load

- Read when clamp, mapRange, normalize, interpolate, snap, pipe, unitize, wrap, or selector utilities feed animation values.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/UtilityMethods/
- https://gsap.com/docs/v3/GSAP/gsap.to()

## Reference Notes

- Prefer reusable composed functions for pointer, scroll, drag, and progress mappings so boundaries are explicit and testable.
- Keep unit conversion at the edge. Numeric helpers should not silently accept CSS unit strings unless `unitize` or a specific parser owns that boundary.
- Use scoped selector helpers in component systems to avoid cross-component targeting.

## Focused Checks

- Test min, max, below-range, above-range, and NaN inputs for mapping helpers.
- Check whether helper functions allocate work inside frame loops.

## Failure Modes

- Mapping unit strings with numeric helpers.
- Creating new helper functions on every frame or render when a stable function would work.


## Operating Guidance

gsap.utils helpers such as clamp, mapRange, normalize, interpolate, random, snap, toArray, selector, wrap, pipe, unitize, and function-based value review.

### Decision Boundaries

- Use plain JavaScript helpers when GSAP is not already part of the animation stack.
- Use gsap-core when helper values feed tweens.
- Use gsap-scrolltrigger when helpers map scroll progress.

### Workflow Details

1. Identify whether the helper should return a reusable function or immediate value.
2. Keep unit handling explicit.
3. Scope selector helpers in component code.
4. Test boundary inputs and cyclic values.

### Gotchas

- mapRange and normalize operate on numbers, not unit strings.
- Omitting the final value returns a reusable function; this is often the intended pattern.
- selector(scope) prevents cross-component targeting mistakes.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
