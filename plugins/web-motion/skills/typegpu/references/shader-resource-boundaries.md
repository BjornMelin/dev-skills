# Shader/resource ownership rules

Skill: typegpu
Checked at: 2026-06-04

## When To Load

- Read when code mixes CPU buffers, GPU resources, schemas, bind groups, and shader functions.


## Operating Guidance

For full workflow, commands, and validation notes, read
`references/typegpu-codex-playbook.md`. This file focuses on shader/resource
ownership boundaries.

### Decision Boundaries

- Use this file when the task crosses shader code, schema definitions, buffers,
  bind groups, textures, pipelines, or CPU/GPU ownership.
- Use `references/typegpu-codex-playbook.md` for general routing, command
  references, and broad validation flow.
- Route scene graph or material work without TypeGPU resource ownership to
  `web-three-r3f`.

### Ownership Checks

1. Identify the source of truth for each `d.*` schema and the CPU values that
   populate it.
2. Confirm every buffer, texture, sampler, bind group, and pipeline has a clear
   owner and disposal path.
3. Check whether resources are allocated during setup, resize, or per-frame
   work; per-frame allocation needs measurement and caching justification.
4. Verify shader function inputs match the schema and resource binding shape.

### Common Failure Modes

- A d.* schema is the CPU layout, GPU layout, and TypeScript type source of truth.
- TypeScript shader functions require unplugin-typegpu; WGSL-only usage may not.
- Do not allocate buffers, textures, bind groups, or pipelines per frame unless measured and cached.
- Resizing a canvas or texture can invalidate dependent resources; rebuild only
  the resources that depend on the changed size.
- Capability checks must happen before TypeGPU initialization code assumes a
  usable WebGPU adapter/device.

## Validation Notes

- Run `node scripts/audit.mjs doctor --root <repo> --format json` before
  interpreting scan results when dependency setup is unclear.
- Run `node scripts/audit.mjs scan --root <repo> --format markdown` for
  repeatable static findings, then verify each finding against current code.
- Smoke-test unsupported-browser fallback and resource cleanup paths when the
  change affects runtime behavior.
