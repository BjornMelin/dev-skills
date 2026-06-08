---
name: web-tailwind-motion
description: >-
  Use this skill for Tailwind CSS v4 transition, animation, duration, easing, motion-safe/motion-reduce, @theme motion tokens, and static class safety. Trigger on Tailwind animation, transition-all, motion-safe, motion-reduce, @theme, animate-, duration-. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web Tailwind Motion

Tailwind CSS v4 transition, animation, duration, easing, motion-safe/motion-reduce, @theme motion tokens, and static class safety.

## Operating Contract

Use this skill as a compact router plus domain checklist. Load references only
when the current task matches their condition. Do not cite local scrape paths,
machine cache paths, or hidden source locations. Verify API details against the
target repo's installed package versions before editing.

## Source Order

1. Inspect the target repo's installed packages, framework/runtime versions,
   local design tokens, accessibility policy, and existing motion patterns.
2. Use the bundled references below for skill-specific gotchas and copied source
   excerpts.
3. Use official current docs/package source as API truth when local code or
   bundled notes are version-sensitive.

## Decision Boundaries

- Use web-css-animations for raw CSS keyframes or browser support policy.
- Use Motion/GSAP when React state or imperative sequencing owns motion.
- Never generate unbounded runtime class strings for Tailwind.

## Workflow

1. Inspect Tailwind version, CSS entrypoint, theme tokens, and class-generation policy.
2. Prefer explicit transition properties and tokenized durations/eases.
3. Use motion-safe/motion-reduce variants for user preference.
4. Validate generated classes are statically discoverable.

## Gotchas

- transition-all can hide expensive properties.
- Tailwind v4 tokens usually live in CSS @theme, not only JS config.
- Dynamic class concatenation can be purged or unsupported by local policy.

<!-- skill-resources:start -->
## Bundled Resources

- `references/tailwind-v4-motion-utilities.md` - Tailwind transition and animation utilities. Read before adding transition, duration, ease, delay, animate, or motion variants.
- `references/token-and-theme-motion.md` - Tailwind v4 @theme motion tokens. Read when adding or reviewing reusable motion tokens.
- `references/class-safety-audit.md` - Static class and runtime string safety. Read when classes are generated from props, CMS data, or maps.
- `references/responsive-motion-variants.md` - Responsive and motion preference variants. Read when adding Tailwind responsive, hover/focus/active, `motion-safe`, or `motion-reduce` animation classes.
- `references/semantic-token-naming.md` - Semantic motion token naming. Read when adding Tailwind v4 `@theme` motion tokens or component-specific animation utilities.
- `references/docs-tailwind-animation.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-tailwind-theme.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-tailwind-transition-property.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-tailwind-v4-release.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/tailwind-motion-field-guide.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-tailwind-motion-audit-report.md` - Audit response/report template.
- `assets/templates/web-tailwind-motion-review-checklist.md` - Manual review checklist.
- `assets/examples/web-tailwind-motion-starter.html` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-tailwind-motion-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
