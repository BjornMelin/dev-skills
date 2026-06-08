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

## Decision Boundaries

- Use this skill for TypeGPU schemas, typed buffers/textures, shader functions,
  pipelines, WebGPU capability checks, and CPU/GPU resource ownership.
- Do not trigger for raw WebGPU without TypeGPU imports or `tgpu` API usage.
- Route Three.js/R3F scene graph work to `web-three-r3f`; route React Native GPU
  surfaces to the native motion skills.

## Command References

- Run the TypeGPU audit:
  `node scripts/audit.mjs scan --root <repo> --format markdown`
- Capture JSON findings:
  `node scripts/audit.mjs scan --root <repo> --format json --output typegpu-audit.json`
- Check setup and dependency context:
  `node scripts/audit.mjs doctor --root <repo> --format json`

## Validation Notes

- Confirm adapter/device availability and unsupported-browser fallback.
- Verify buffers, textures, bind groups, and pipelines are not allocated per
  frame unless measured and cached.
- Keep schema, CPU data layout, and GPU resource ownership aligned.
