# GLTF/texture loaders and fallbacks

Skill: web-three-r3f
Checked at: 2026-06-04

## When To Load

- Read for useGLTF/useTexture, Suspense, decoder paths, and asset errors.


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
