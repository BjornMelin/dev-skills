# R3F interaction and event boundaries

Skill: web-three-r3f
Checked at: 2026-06-04

## When To Load

- Read when a Three/R3F scene handles pointer, keyboard, scroll, controls, raycasting, or HTML overlays.

## Source Anchors

- https://r3f.docs.pmnd.rs/api/canvas
- https://r3f.docs.pmnd.rs/advanced/pitfalls

## Reference Notes

- R3F pointer events are scene events, not DOM events with identical propagation. Keep overlay DOM, canvas events, and controls ownership explicit.
- Camera controls can fight page scroll and mobile touch. Decide whether the canvas captures, passes through, or conditionally handles gestures.
- Accessible controls and fallback content must exist outside the WebGL canvas when interaction is meaningful.

## Focused Checks

- Test pointer, touch, keyboard, scroll, and overlay hit-testing.
- Verify nonblank canvas pixels and fallback behavior when WebGL fails.

## Failure Modes

- Full-screen canvas swallowing page scroll on mobile without product intent.
- Important labels or controls rendered only inside WebGL text.


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
