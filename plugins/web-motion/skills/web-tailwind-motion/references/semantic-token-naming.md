# Semantic motion token naming

Skill: web-tailwind-motion
Checked at: 2026-06-04

## When To Load

- Read when adding Tailwind v4 `@theme` motion tokens or component-specific animation utilities.

## Source Anchors

- https://tailwindcss.com/docs/theme
- https://tailwindcss.com/docs/transition-property

## Reference Notes

- Name motion tokens by product intent and scope, not implementation detail alone. `--ease-enter-panel` is more useful than another generic cubic-bezier token.
- Keep duration/ease/distance token ownership close to the design system; component overrides should be measured exceptions.
- Token changes are shared behavior changes and should be validated across representative components.

## Focused Checks

- Search for existing duration/ease/animation tokens before adding new ones.
- Check generated utility names and static class discoverability.

## Failure Modes

- Hard-coded one-off `duration-[...]` values repeated across components.
- Dynamic class strings assembled from untrusted CMS or user data.


## Operating Guidance

Tailwind CSS v4 transition, animation, duration, easing, motion-safe/motion-reduce, @theme motion tokens, and static class safety.

### Decision Boundaries

- Use web-css-animations for raw CSS keyframes or browser support policy.
- Use Motion/GSAP when React state or imperative sequencing owns motion.
- Never generate unbounded runtime class strings for Tailwind.

### Workflow Details

1. Inspect Tailwind version, CSS entrypoint, theme tokens, and class-generation policy.
2. Prefer explicit transition properties and tokenized durations/eases.
3. Use motion-safe/motion-reduce variants for user preference.
4. Validate generated classes are statically discoverable.

### Gotchas

- transition-all can hide expensive properties.
- Tailwind v4 tokens usually live in CSS @theme, not only JS config.
- Dynamic class concatenation can be purged or unsupported by local policy.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
