# 3D asset pipeline, compression, and loader policy

Skill: web-three-r3f
Checked at: 2026-06-04

## When To Load

- Read when loading GLTF/GLB, textures, Draco/Meshopt/KTX2 assets, or remote 3D content.

## Source Anchors

- https://drei.docs.pmnd.rs/loaders/gltf-use-gltf
- https://threejs.org/manual/#en/cleanup
- https://r3f.docs.pmnd.rs/advanced/scaling-performance

## Reference Notes

- Model loading is both a runtime and build/deploy concern. Decoder paths, public asset URLs, cache headers, suspense fallback, and bundle size all matter.
- Use the project asset pipeline and avoid unreviewed remote models unless the security/cache policy allows them.
- Dispose of externally created geometries, materials, textures, render targets, and controls when the owner unmounts.

## Focused Checks

- Verify missing asset, decode failure, slow network, and route unmount behavior.
- Check texture size, compression format, DPR, and memory budget.

## Failure Modes

- Loading large uncompressed GLB/textures directly in the first route without fallback.
- Creating Three objects outside R3F ownership without cleanup.


## Operating Guidance

Three.js, React Three Fiber, Drei, Canvas/createRoot lifecycle, loaders, GLTF, useFrame, disposal, SSR/client boundaries, DPR, and browser proof.

### Decision Boundaries

- Use typegpu for typed WebGPU pipelines.
- Use native-three-r3f for Expo/React Native.
- Use CSS 3D transforms only for simple DOM transforms.

### Workflow Details

1. Inspect Three/R3F/Drei/React versions and asset pipeline.
2. Choose Canvas/R3F or plain Three renderer based on local ownership.
3. Reserve stable layout, DPR, fallback, and reduced-motion behavior.
4. Verify nonblank pixels, resize, interaction, asset errors, and cleanup.

### Gotchas

- Undefined parent height causes blank canvases.
- React state inside useFrame causes frame-loop reconciliation.
- R3F auto-disposal does not cover every primitive or externally created object.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
