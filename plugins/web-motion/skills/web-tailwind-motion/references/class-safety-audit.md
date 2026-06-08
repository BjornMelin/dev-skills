# Static class and runtime string safety

Skill: web-tailwind-motion
Checked at: 2026-06-04

## When To Load

- Read when classes are generated from props, CMS data, or maps.


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
