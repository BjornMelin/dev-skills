# Compute versus render pipeline design

Skill: typegpu
Checked at: 2026-06-04

## When To Load

- Read when choosing TypeGPU compute, render, storage buffer, texture, or shader-function patterns.

## Source Anchors

- https://docs.swmansion.com/TypeGPU/apis/data-schemas/
- https://docs.swmansion.com/TypeGPU/

## Reference Notes

- Use schemas as the shared contract between CPU data layout, TypeScript types, and shader access.
- Choose compute when the output is data for later stages, and render when the output is pixels for a render target or canvas.
- Cache pipelines, bind groups, and buffers by shape/usage; avoid per-frame structural allocation.

## Focused Checks

- Verify buffer usage flags match read/write/bind behavior.
- Add small deterministic fixtures for schema packing and shader input/output shape.

## Failure Modes

- Duplicating layout definitions in TypeScript and WGSL.
- Recreating pipelines or bind groups inside frame loops without measurement.


## Operating Guidance

TypeGPU schemas, typed buffers/textures, shader functions, pipelines, WebGPU capability checks, and CPU/GPU resource ownership.

### Decision Boundaries

- Do not trigger for raw WebGPU without TypeGPU imports.
- Use web-three-r3f for Three/R3F scenes.
- Use native-three-r3f or native-skia for React Native GPU surfaces.

### Workflow Details

1. Check installed typegpu, unplugin-typegpu, @webgpu/types, tsover, and browser/runtime support.
2. Define schemas before resources and shader signatures.
3. Keep root/device/resource ownership explicit.
4. Validate unsupported-browser fallback, reduced-motion/static quality, and GPU cleanup.

### Gotchas

- A d.* schema is the CPU layout, GPU layout, and TypeScript type source of truth.
- TypeScript shader functions require unplugin-typegpu; WGSL-only usage may not.
- Do not allocate buffers, textures, bind groups, or pipelines per frame unless measured and cached.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
