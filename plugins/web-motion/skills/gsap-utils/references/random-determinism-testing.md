# Randomness, wrapping, and deterministic testing

Skill: gsap-utils
Checked at: 2026-06-04

## When To Load

- Read when `random`, `shuffle`, `wrap`, `wrapYoyo`, or function-based values affect visual output.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/UtilityMethods/
- https://agentskills.io/skill-creation/best-practices

## Reference Notes

- Use deterministic inputs or seeded alternatives in tests and visual review fixtures when random output would make failures hard to reproduce.
- Document whether randomness is product behavior, decorative variance, or a placeholder to remove.
- Wrap helpers are useful for cyclic indexes and carousel-like ranges, but they need explicit bounds and behavior at negative values.

## Focused Checks

- Check snapshot/visual tests for nondeterministic animation values.
- Verify cyclic values at first, last, overflow, and underflow indexes.

## Failure Modes

- Random values in SSR-rendered markup or hydration-sensitive initial styles.
- Function-based values with side effects.


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
