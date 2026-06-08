---
name: web-three-r3f
description: >-
  Use this skill for Three.js, React Three Fiber, Drei, Canvas/createRoot lifecycle, loaders, GLTF, useFrame, disposal, SSR/client boundaries, DPR, and browser proof. Trigger on Three.js, THREE, @react-three/fiber, @react-three/drei, R3F Canvas, useFrame, GLTF, WebGLRenderer. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Web Three R3F

Three.js, React Three Fiber, Drei, Canvas/createRoot lifecycle, loaders, GLTF, useFrame, disposal, SSR/client boundaries, DPR, and browser proof.

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

- Use typegpu for typed WebGPU pipelines.
- Use native-three-r3f for Expo/React Native.
- Use CSS 3D transforms only for simple DOM transforms.

## Workflow

1. Inspect Three/R3F/Drei/React versions and asset pipeline.
2. Choose Canvas/R3F or plain Three renderer based on local ownership.
3. Reserve stable layout, DPR, fallback, and reduced-motion behavior.
4. Verify nonblank pixels, resize, interaction, asset errors, and cleanup.

## Gotchas

- Undefined parent height causes blank canvases.
- React state inside useFrame causes frame-loop reconciliation.
- R3F auto-disposal does not cover every primitive or externally created object.

<!-- skill-resources:start -->
## Bundled Resources

- `references/r3f-scene-lifecycle.md` - Canvas/createRoot and scene lifecycle. Read for Canvas props, custom root ownership, SSR/client boundaries, and resize.
- `references/asset-loaders-and-fallbacks.md` - GLTF/texture loaders and fallbacks. Read for useGLTF/useTexture, Suspense, decoder paths, and asset errors.
- `references/three-disposal-performance.md` - Three.js disposal and performance guide. Read for renderer cleanup, render targets, materials, textures, DPR, frameloop, and profiling.
- `references/interaction-and-event-boundaries.md` - R3F interaction and event boundaries. Read when a Three/R3F scene handles pointer, keyboard, scroll, controls, raycasting, or HTML overlays.
- `references/asset-pipeline-compression.md` - 3D asset pipeline, compression, and loader policy. Read when loading GLTF/GLB, textures, Draco/Meshopt/KTX2 assets, or remote 3D content.
- `references/docs-drei-gltf-use-gltf.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-canvas.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-pitfalls.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-scaling-performance.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-three-disposal.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-introduction.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-three-creating-a-scene.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/r3f-field-guide.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/web-three-r3f-audit-report.md` - Audit response/report template.
- `assets/templates/web-three-r3f-review-checklist.md` - Manual review checklist.
- `assets/examples/web-three-r3f-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output web-three-r3f-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
