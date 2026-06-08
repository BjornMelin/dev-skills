---
name: native-three-r3f
description: >-
  Use this skill for React Three Fiber native, Three.js in Expo/React Native, expo-gl/WebGPU boundaries, GLTF/assets, native GPU lifecycle, and platform validation. Trigger on @react-three/fiber/native, Expo Three, expo-gl, react-native-wgpu, native Three.js, R3F native. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# Native Three R3F

React Three Fiber native, Three.js in Expo/React Native, expo-gl/WebGPU boundaries, GLTF/assets, native GPU lifecycle, and platform validation.

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

- Use web-three-r3f for browser Three/R3F.
- Use typegpu for web TypeGPU code.
- Use native-skia for 2D canvas drawing.

## Workflow

1. Inspect Expo SDK, GL/WebGPU package, R3F/Three versions, and asset pipeline.
2. Choose native R3F only when a 3D scene is the product surface.
3. Own canvas dimensions, DPR/quality, loaders, and cleanup.
4. Validate on device/development build for native GPU risk.

## Gotchas

- Browser R3F examples often assume DOM/WebGL APIs absent on native.
- Native asset loading and decoder paths differ from web.
- GPU runtime changes need device proof, not just TypeScript.

<!-- skill-resources:start -->
## Bundled Resources

- `references/r3f-native-installation.md` - R3F native installation and lifecycle. Read for @react-three/fiber/native setup and scene ownership.
- `references/expo-webgpu-three-boundary.md` - Expo GL/WebGPU/Three boundary notes. Read when expo-gl, react-native-wgpu, or WebGPU interop appears.
- `references/native-gpu-validation.md` - Native GPU validation checklist. Read before closing R3F/Three native changes.
- `references/expo-gl-webgpu-decision-tree.md` - Expo GL, WebGPU, and Three decision tree. Read when choosing expo-gl, WebGPU, Three.js, R3F native, or a fallback rendering surface.
- `references/native-asset-loader-recipes.md` - Native 3D asset loader recipes. Read when loading GLTF/GLB, textures, HDR/environment maps, or decoder-backed assets in native Three/R3F.
- `references/docs-expo-webgpu-three.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-native-api-notes.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-react-native-installation.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-three-creating-a-scene.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-r3f-introduction.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/docs-software-mansion-gpu-animations.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/native-three-r3f-audit-report.md` - Audit response/report template.
- `assets/templates/native-three-r3f-review-checklist.md` - Manual review checklist.
- `assets/examples/native-three-r3f-starter.tsx` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output native-three-r3f-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
