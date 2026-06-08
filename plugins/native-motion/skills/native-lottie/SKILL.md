---
name: native-lottie
description: >-
  Use this skill for lottie-react-native and dotLottie native assets, Expo compatibility, asset bundling, refs, playback control, accessibility, reduced motion, and platform validation. Trigger on lottie-react-native, dotLottie React Native, LottieView, .lottie native, After Effects native animation. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Lottie

lottie-react-native and dotLottie native assets, Expo compatibility, asset bundling, refs, playback control, accessibility, reduced motion, and platform validation.

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

- Use web-lottie for browser Lottie.
- Use native-rive for interactive Rive state machines.
- Use native-motion-core for code-driven Reanimated motion.

## Workflow

1. Check Expo SDK package compatibility and asset format.
2. Bundle assets through the app asset pipeline.
3. Own playback refs, pause/stop behavior, and unmount cleanup.
4. Validate Android/iOS rendering, reduced motion, and accessibility labels.

## Gotchas

- Large JSON animations can hurt startup and memory.
- Autoplay loops need pause/reduced-motion behavior.
- Native asset paths differ from web URLs and need bundler-safe imports.

<!-- skill-resources:start -->
## Bundled Resources

- `references/native-lottie-asset-lifecycle.md` - Native Lottie asset lifecycle. Read for LottieView refs, source formats, asset imports, and playback control.
- `references/dotlottie-native-boundaries.md` - dotLottie native boundaries. Read when using .lottie assets or LottieFiles native packages.
- `references/accessibility-performance.md` - Native Lottie accessibility and performance. Read for labels, reduced motion, looping, asset size, and platform proof.
- `references/designer-handoff-and-feature-support.md` - Native Lottie designer handoff and feature support. Read when a Lottie asset is new, visually wrong on device, large, or different from the design preview.
- `references/native-playback-control-refs.md` - Native Lottie playback refs and lifecycle. Read when code controls LottieView refs, imperative playback, progress, segments, or component unmount.
- `references/docs-lottie-react-native-readme.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-lottie-audit-report.md` - Audit response/report template.
- `assets/templates/native-lottie-review-checklist.md` - Manual review checklist.
- `assets/examples/native-lottie-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-lottie-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
