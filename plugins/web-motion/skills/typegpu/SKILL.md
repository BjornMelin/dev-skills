---
name: typegpu
description: >-
  Use this skill for TypeGPU schemas, typed buffers/textures, shader functions, pipelines, WebGPU capability checks, and CPU/GPU resource ownership. Trigger on TypeGPU, tgpu, d.struct, use gpu, unplugin-typegpu, typed WebGPU, shader functions. Do not use for near-miss tasks outside these boundaries; route to adjacent motion or platform skills when they own the implementation.
license: MIT
---

# TypeGPU

TypeGPU schemas, typed buffers/textures, shader functions, pipelines, WebGPU capability checks, and CPU/GPU resource ownership.

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

- Do not trigger for raw WebGPU without TypeGPU imports.
- Use web-three-r3f for Three/R3F scenes.
- Use native-three-r3f or native-skia for React Native GPU surfaces.

## Workflow

1. Check installed typegpu, unplugin-typegpu, @webgpu/types, tsover, and browser/runtime support.
2. Define schemas before resources and shader signatures.
3. Keep root/device/resource ownership explicit.
4. Validate unsupported-browser fallback, reduced-motion/static quality, and GPU cleanup.

## Gotchas

- A d.* schema is the CPU layout, GPU layout, and TypeScript type source of truth.
- TypeScript shader functions require unplugin-typegpu; WGSL-only usage may not.
- Do not allocate buffers, textures, bind groups, or pipelines per frame unless measured and cached.

<!-- skill-resources:start -->
## Bundled Resources

- `references/typegpu-codex-playbook.md` - Codex workflow for TypeGPU tasks. Read before writing TypeGPU app code, shader functions, or compute/render pipelines.
- `references/shader-resource-boundaries.md` - Shader/resource ownership rules. Read when code mixes CPU buffers, GPU resources, schemas, bind groups, and shader functions.
- `references/webgpu-runtime-validation.md` - Browser WebGPU validation and fallbacks. Read for secure context, adapter/device, unsupported browser, reduced motion, and teardown proof.
- `references/browser-capability-and-adapter-selection.md` - Browser capability and adapter selection. Read before creating TypeGPU roots, requesting WebGPU adapters/devices, or designing fallbacks.
- `references/compute-vs-render-pipeline-design.md` - Compute versus render pipeline design. Read when choosing TypeGPU compute, render, storage buffer, texture, or shader-function patterns.
- `references/matrices.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/pipelines.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/setup.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/shaders.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/textures.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/types.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/advanced.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/noise.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/sdf.md` - Copied source excerpt. Load only when exact upstream wording or API detail is needed.
- `references/index.md` - Complete reference inventory and routing summary.
- `references/source-ledger.md` - Source list, checked date, and copy policy.
- `references/provenance.json` - Machine-readable source and local-resource metadata.
- `scripts/audit.mjs` - Self-contained audit CLI; run `doctor` before `scan` when setup is unclear.
- `assets/templates/typegpu-audit-report.md` - Audit response/report template.
- `assets/templates/typegpu-review-checklist.md` - Manual review checklist.
- `assets/examples/typegpu-starter.ts` - Starter fixture/example for this skill.
- `evals/trigger-queries.json` - Trigger/near-miss eval set for description tuning.
- `evals/evals.json` - Task-quality evals with assertions.
<!-- skill-resources:end -->

## Audit CLI

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
node scripts/audit.mjs scan --root . --format json --output typegpu-audit.json
```

Treat script findings as leads. Verify every finding against current code before
changing behavior or reporting it as valid.

## Closeout

Before finalizing, run the repo's focused validation command, this skill's audit
CLI when relevant, and any browser/device/manual proof required by the changed
surface. Report commands run, findings fixed, findings skipped with reasons,
and residual risk.
