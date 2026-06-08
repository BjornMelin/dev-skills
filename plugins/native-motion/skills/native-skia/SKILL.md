---
name: native-skia
description: >-
  Use this skill for React Native Skia canvas animations, drawing primitives, Reanimated integration, CanvasKit/web caveats, performance, memory, and Expo/native validation. Trigger on React Native Skia, @shopify/react-native-skia, Skia Canvas, CanvasKit, Skia animation. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Skia

React Native Skia canvas animations, drawing primitives, Reanimated integration, CanvasKit/web caveats, performance, memory, and Expo/native validation.

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

- Use native-motion-core for ordinary view motion.
- Use native-three-r3f for 3D scenes.
- Use native-lottie/native-rive for designer-authored assets.

## Workflow

1. Check Skia, Expo SDK, platform support, and web requirements.
2. Keep drawing state and animation values in the appropriate runtime.
3. Avoid many animated native views when one canvas is the right surface.
4. Validate memory, resize, background/foreground, and platform rendering.

## Gotchas

- Skia web may require CanvasKit setup.
- Canvas output needs accessible surrounding UI.
- Large paths/images/shaders need memory and lifecycle ownership.

<!-- skill-resources:start -->
## Bundled Resources

- `references/skia-canvas-patterns.md` - Skia canvas animation patterns. Read when building or reviewing Skia drawing/animation code.
- `references/skia-performance-lifecycle.md` - Skia performance and lifecycle. Read for memory, resource, image/path/shader, and frame pressure review.
- `references/skia-web-expo-boundaries.md` - Skia web and Expo boundaries. Read for CanvasKit, Expo package compatibility, and platform proof.
- `references/skia-reanimated-interoperability.md` - Skia and Reanimated interoperability. Read when Skia drawing state is animated by Reanimated shared values, gestures, or UI-thread updates.
- `references/image-font-shader-resource-cache.md` - Skia image, font, shader, and resource cache policy. Read when Skia code loads images, fonts, paths, shaders, color filters, or large resources.
- `references/docs-react-native-skia-api-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-react-native-skia-installation.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-software-mansion-canvas-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-software-mansion-canvas-atlas.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-skia-audit-report.md` - Audit response/report template.
- `assets/templates/native-skia-review-checklist.md` - Manual review checklist.
- `assets/examples/native-skia-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-skia-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
