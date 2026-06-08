---
name: web-waapi
description: >-
  Use this skill for Browser Web Animations API: Element.animate(), Animation, KeyframeEffect, playback control, generated keyframes, cancel/finish, commitStyles, and cleanup. Trigger on Element.animate, WAAPI, Web Animations API, KeyframeEffect, Animation object, commitStyles. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web WAAPI

Browser Web Animations API: Element.animate(), Animation, KeyframeEffect, playback control, generated keyframes, cancel/finish, commitStyles, and cleanup.

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

- Use CSS for simple state transitions.
- Use Motion/GSAP when framework state or timelines dominate.
- Use WAAPI when code needs an Animation object, seeking, cancellation, or generated keyframes.

## Workflow

1. Check browser support and local fallback policy.
2. Create keyframes/options with explicit duration, fill, easing, and composite behavior.
3. Own animation cancellation and finish behavior.
4. Verify rapid interruptions, route unmount, reduced motion, and commitStyles usage.

## Gotchas

- commitStyles persists computed styles and should be followed by cancel when appropriate.
- fill: forwards can retain stacking/style side effects.
- Multiple animations on the same property need composite/replace intent.

<!-- skill-resources:start -->
## Bundled Resources

- `references/waapi-lifecycle.md` - Animation object lifecycle. Read for play, pause, reverse, finish, cancel, commitStyles, and cleanup.
- `references/keyframe-effect-options.md` - Keyframes and timing options. Read for KeyframeEffect, composite, fill, pseudoElement, iterations, and generated values.
- `references/playback-testing.md` - Playback and interruption validation. Read for reduced motion, rapid toggles, route unmount, and deterministic testing.
- `references/promise-events-and-cancellation.md` - Animation promises, events, and cancellation. Read when WAAPI code awaits `finished`, reacts to `finish`/`cancel`, or coordinates interruption.
- `references/testing-with-getanimations.md` - Testing with getAnimations and deterministic fixtures. Read when testing or auditing WAAPI effects programmatically.
- `references/docs-mdn-web-animations-api.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-waapi-audit-report.md` - Audit response/report template.
- `assets/templates/web-waapi-review-checklist.md` - Manual review checklist.
- `assets/examples/web-waapi-starter.ts` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-waapi-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
