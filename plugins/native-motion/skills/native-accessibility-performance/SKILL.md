---
name: native-accessibility-performance
description: >-
  Use this skill for Expo/React Native motion accessibility, reduced motion, haptics, UI-thread performance, frame pressure, gestures, and manual device proof. Trigger on React Native animation performance, reduced motion native, AccessibilityInfo, haptics, dropped frames, UI thread. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Accessibility Performance

Expo/React Native motion accessibility, reduced motion, haptics, UI-thread performance, frame pressure, gestures, and manual device proof.

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

- Use native-motion-core for implementation patterns.
- Use native-validation for command and device validation gates.
- Use native-skia or native-three-r3f for canvas/GPU-specific performance.

## Workflow

1. Inspect platform, Expo SDK, animation engine, and accessibility setting ownership.
2. Classify UI-thread, JS-thread, layout, and GPU work.
3. Add reduced-motion or static behavior without removing functional feedback.
4. Validate on iOS/Android or document skipped device proof.

## Gotchas

- Reduced motion should not remove essential progress, focus, pressed, or error feedback.
- Haptics are feedback, not a substitute for visible state.
- Per-frame React state updates can destroy native animation performance.

<!-- skill-resources:start -->
## Bundled Resources

- `references/reduced-motion-haptics-policy.md` - Reduced motion and haptics policy. Read when user preference, haptics, or accessible feedback is involved.
- `references/native-performance-audit.md` - Native performance audit guide. Read for JS/UI thread, layout, canvas, GPU, and list animation review.
- `references/manual-device-checks.md` - Manual iOS/Android proof checklist. Read before finalizing changes that need simulator/device evidence.
- `references/gesture-feedback-and-motion-sickness.md` - Gesture feedback and motion sickness review. Read when gestures, haptics, parallax, carousels, shared transitions, or large native motion can affect comfort.
- `references/frame-budget-instrumentation.md` - Native frame budget instrumentation. Read when reports mention dropped frames, slow gestures, list jank, or JS/UI thread contention.
- `references/docs-reanimated-performance.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rn-accessibility.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rn-accessibilityinfo.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-rn-performance.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-animation.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-expo-haptics.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-accessibility.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-reanimated-source-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-accessibility-performance-audit-report.md` - Audit response/report template.
- `assets/templates/native-accessibility-performance-review-checklist.md` - Manual review checklist.
- `assets/examples/native-accessibility-performance-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-accessibility-performance-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
