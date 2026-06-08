---
name: web-motion-react
description: >-
  Use this skill for Motion React components and hooks: motion, AnimatePresence, layout animations, useScroll, useReducedMotion, gestures, variants, and React/Next boundaries. Trigger on Motion React, motion/react, AnimatePresence, layout animation, useScroll, useReducedMotion, variants. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web Motion React

Motion React components and hooks: motion, AnimatePresence, layout animations, useScroll, useReducedMotion, gestures, variants, and React/Next boundaries.

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

- Use GSAP for imperative timelines and plugin-heavy scenes.
- Use CSS for simple static transitions.
- Use WAAPI for low-level Animation object control outside React.

## Workflow

1. Confirm package import path and React/client boundary.
2. Choose presence, layout, gesture, scroll, or value-based motion deliberately.
3. Respect reduced motion and state ownership.
4. Verify layout projection with resize, interruption, route changes, and hydration.

## Gotchas

- AnimatePresence requires stable keys and actual unmounts.
- Layout animations depend on stable layout boxes and should not fight CSS transitions.
- Do not push high-frequency motion values through React state.

<!-- skill-resources:start -->
## Bundled Resources

- `references/motion-react-presence-layout.md` - Presence and layout workflow. Read for AnimatePresence, layout, shared layout, and exit transitions.
- `references/scroll-gestures-reduced-motion.md` - Scroll, gesture, and reduced-motion hooks. Read for useScroll, useTransform, whileHover/tap/drag, and useReducedMotion.
- `references/react-ssr-client-boundaries.md` - React and SSR/client boundaries. Read for Next.js, server components, hydration, and route-level validation.
- `references/variants-and-motion-values.md` - Variants and MotionValue state ownership. Read when Motion React variants, MotionValues, transforms, gestures, or high-frequency values are involved.
- `references/next-router-presence-boundaries.md` - Next.js and router presence boundaries. Read when AnimatePresence, route transitions, layout animations, or shared layout effects cross routing/SSR boundaries.
- `references/docs-motion-animate-presence.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-motion-layout-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-motion-react.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-motion-use-reduced-motion.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-motion-use-scroll.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/motion-react-field-guide.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-motion-react-audit-report.md` - Audit response/report template.
- `assets/templates/web-motion-react-review-checklist.md` - Manual review checklist.
- `assets/examples/web-motion-react-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-motion-react-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
