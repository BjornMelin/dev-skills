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

## Operating Guidance

- `gsap.utils.random()` is useful for decorative variance, but it can make
  visual tests and hydration-sensitive initial styles flaky. Prefer deterministic
  values in test fixtures.
- When using `wrap`, `wrapYoyo`, `mapRange`, `normalize`, or `snap`, test
  boundary inputs explicitly. These helpers are often correct only because their
  input domain is constrained elsewhere.
- If a helper returns a reusable function, name and store that function so call
  sites make the transformation obvious.
- Use `selector(scope)` in component code so randomized target selection cannot
  escape the component root.

## Command References

- Run the utilities audit:
  `node scripts/audit.mjs scan --root <repo> --format markdown`
- Use JSON output for deterministic review artifacts:
  `node scripts/audit.mjs scan --root <repo> --format json --output gsap-utils-audit.json`
- Check package/setup facts:
  `node scripts/audit.mjs doctor --root <repo> --format json`

## Validation Notes

- Verify random output is not part of SSR markup unless it is stable.
- Check first, last, overflow, and underflow indexes for cyclic helpers.
- Confirm unit handling for numeric mapping helpers.
