---
name: native-styling-boundaries
description: >-
  Use this skill for NativeWind, react-native-css, Tailwind-style classes in React Native, static class safety, design tokens, Reanimated/CSS transition boundaries, and Expo setup. Trigger on NativeWind, react-native-css, Tailwind React Native, className native, motion-safe native, NativeWind animation. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Styling Boundaries

NativeWind, react-native-css, Tailwind-style classes in React Native, static class safety, design tokens, Reanimated/CSS transition boundaries, and Expo setup.

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

- Use native-motion-core for Reanimated implementation logic.
- Use web-tailwind-motion for browser Tailwind.
- Do not generate untrusted runtime class strings.

## Workflow

1. Inspect NativeWind/react-native-css versions and Babel/Metro setup.
2. Keep classes statically discoverable and token-driven.
3. Choose style/class ownership before adding Reanimated or CSS transitions.
4. Validate iOS/Android/web behavior where NativeWind support differs.

## Gotchas

- Web Tailwind assumptions do not always map to React Native style props.
- Runtime string concatenation can break class extraction and policy.
- Animation ownership should not be split between NativeWind classes and Reanimated shared values.

<!-- skill-resources:start -->
## Bundled Resources

- `references/nativewind-v4-boundaries.md` - NativeWind setup and version boundaries. Read for NativeWind/react-native-css setup and class ownership.
- `references/react-native-css-tailwind.md` - React Native CSS and Tailwind compatibility. Read when CSS-like transitions or Tailwind utilities cross native/web boundaries.
- `references/class-safety-and-tokens.md` - Native class safety and token policy. Read for dynamic class generation, theme tokens, and design-system constraints.
- `references/nativewind-metro-babel-pipeline.md` - NativeWind Metro, Babel, and CSS pipeline. Read when NativeWind classes do not apply, hot reload fails, or setup crosses Expo/Metro/Tailwind config.
- `references/cross-platform-style-differences.md` - Cross-platform style and animation differences. Read when a class, transition, transform, color, or layout utility behaves differently on iOS, Android, and web.
- `references/docs-expo-tailwind-setup.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-nativewind-installation.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-styling-boundaries-audit-report.md` - Audit response/report template.
- `assets/templates/native-styling-boundaries-review-checklist.md` - Manual review checklist.
- `assets/examples/native-styling-boundaries-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-styling-boundaries-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
