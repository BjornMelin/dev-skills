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

## Determinism Review

1. Replace random values with deterministic fixtures in tests unless randomness is the behavior under review.
2. Document any intentional decorative variance so reviewers do not treat it as flaky output.
3. Validate wrap and wrapYoyo helpers with first, last, overflow, and negative indexes.
4. Keep selector helpers scoped when randomization targets component-local elements.
