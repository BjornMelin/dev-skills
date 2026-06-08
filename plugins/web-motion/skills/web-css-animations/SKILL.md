---
name: web-css-animations
description: >-
  Use this skill for Browser CSS transitions, keyframes, scroll-driven animations, registered properties, discrete transitions, reduced motion, and performance-safe CSS motion. Trigger on CSS transition, @keyframes, animation-timeline, prefers-reduced-motion, @starting-style, transition-behavior, CSS animation. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web CSS Animations

Browser CSS transitions, keyframes, scroll-driven animations, registered properties, discrete transitions, reduced motion, and performance-safe CSS motion.

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

- Use CSS first for two-state UI motion.
- Move to WAAPI when an Animation object or seeking is needed.
- Move to GSAP for complex imperative choreography.

## Workflow

1. Identify the state driver and animated properties.
2. Use explicit transition-property lists and product motion tokens.
3. Add reduced-motion behavior beside the motion.
4. Guard new CSS with @supports or local browser policy.

## Gotchas

- transition: all hides expensive accidental properties.
- Unregistered custom properties animate discretely.
- animation shorthand resets animation-timeline, so set timeline after shorthand.

<!-- skill-resources:start -->
## Bundled Resources

- `references/css-motion-field-guide.md` - CSS transition/keyframe field guide. Read before implementing ordinary CSS state motion.
- `references/browser-support-and-accessibility.md` - Support, @supports, and reduced-motion notes. Read for newer CSS features or accessibility review.
- `references/property-performance-matrix.md` - CSS property performance matrix. Read for transform, opacity, layout, paint, filter, shadow, and text animation risk.
- `references/scroll-driven-and-view-timelines.md` - Scroll-driven and view-timeline CSS motion. Read when using `animation-timeline`, `scroll()`, `view()`, timeline ranges, or CSS scroll-driven animation instead of JavaScript scroll handlers.
- `references/discrete-entry-exit-transitions.md` - Discrete entry and exit transitions. Read when animating `display`, `content-visibility`, popovers, dialogs, `@starting-style`, or `transition-behavior`.
- `references/docs-css-modern-motion-notes.md` - Source-backed notes and links. Load when exact upstream API detail may have changed.
- `references/docs-mdn-css-animations.md` - MDN CSS animations routing notes. Load when reviewing `@keyframes` and `animation-*` behavior.
- `references/docs-mdn-css-transitions.md` - MDN CSS transitions routing notes. Load when reviewing `transition-*` behavior.
- `references/docs-mdn-prefers-reduced-motion.md` - MDN reduced-motion routing notes. Load when reviewing CSS reduced-motion behavior.
- `references/docs-browser-motion-performance.md` - Source-backed browser performance notes. Load when checking property or compositor risk.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-css-animations-audit-report.md` - Audit response/report template.
- `assets/templates/web-css-animations-review-checklist.md` - Manual review checklist.
- `assets/examples/web-css-animations-starter.css` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-css-animations-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
