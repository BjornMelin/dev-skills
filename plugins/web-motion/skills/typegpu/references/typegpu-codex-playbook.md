# Codex workflow for TypeGPU tasks

Skill: typegpu
Checked at: 2026-06-04

## When To Load

- Read before writing TypeGPU app code, shader functions, or compute/render pipelines.

## Codex Workflow

1. Confirm the task uses TypeGPU or `tgpu` APIs, not raw WebGPU alone.
2. Inspect installed `typegpu`, `unplugin-typegpu`, and browser WebGPU support.
3. Define `d.*` schemas before buffers, textures, shader signatures, and pipelines.
4. Keep root, device, and resource disposal ownership explicit in the answer or patch.
5. Validate unsupported-browser fallback, reduced-motion/static quality, and GPU cleanup.
