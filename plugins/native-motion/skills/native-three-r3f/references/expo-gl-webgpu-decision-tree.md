# Expo GL, WebGPU, and Three decision tree

Skill: native-three-r3f
Checked at: 2026-06-04

## When To Load

- Read when choosing expo-gl, WebGPU, Three.js, R3F native, or a fallback rendering surface.

## Source Anchors

- https://docs.pmnd.rs/react-three-fiber/getting-started/installation#react-native
- https://developer.mozilla.org/docs/Web/API/WebGPU_API

## Reference Notes

- Native Three/R3F work is a runtime decision, not just an import decision. Confirm Expo SDK, renderer support, development build requirements, and target platforms first.
- Use native 3D only when the product surface needs 3D. Use Skia for 2D canvas and Reanimated/native views for ordinary UI motion.
- Web examples often assume DOM, browser WebGL extension behavior, and asset loaders not present on native.

## Focused Checks

- Check iOS/Android build and runtime proof requirements before coding deeply.
- Verify fallback behavior for unsupported devices or GPU errors.

## Failure Modes

- Copying browser R3F Canvas examples into native without renderer/package review.
- Treating TypeScript success as GPU runtime proof.


## Operating Guidance

React Three Fiber native, Three.js in Expo/React Native, expo-gl/WebGPU boundaries, GLTF/assets, native GPU lifecycle, and platform validation.

### Decision Boundaries

- Use web-three-r3f for browser Three/R3F.
- Use typegpu for web TypeGPU code.
- Use native-skia for 2D canvas drawing.

### Workflow Details

1. Inspect Expo SDK, GL/WebGPU package, R3F/Three versions, and asset pipeline.
2. Choose native R3F only when a 3D scene is the product surface.
3. Own canvas dimensions, DPR/quality, loaders, and cleanup.
4. Validate on device/development build for native GPU risk.

### Gotchas

- Browser R3F examples often assume DOM/WebGL APIs absent on native.
- Native asset loading and decoder paths differ from web.
- GPU runtime changes need device proof, not just TypeScript.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
