# Native 3D asset loader recipes

Skill: native-three-r3f
Checked at: 2026-06-04

## When To Load

- Read when loading GLTF/GLB, textures, HDR/environment maps, or decoder-backed assets in native Three/R3F.

## Source Anchors

- https://docs.pmnd.rs/react-three-fiber/getting-started/installation#react-native
- https://threejs.org/manual/#en/cleanup
- https://drei.docs.pmnd.rs/loaders/gltf-use-gltf

## Reference Notes

- Native asset loading needs bundler-safe imports, URI resolution, and platform proof. Public web URLs and decoder paths may not map to native.
- Large meshes/textures need memory budget and loading fallback. Compression helps only when the decoder path is available.
- Scene cleanup must include externally created materials, geometries, textures, controls, and renderer resources not owned automatically.

## Focused Checks

- Test missing asset, reload, route unmount, memory pressure, and app background/foreground.
- Verify dimensions and DPR/quality on device.

## Failure Modes

- Remote unpinned model URLs in production UI.
- Large uncompressed textures loaded before the screen needs them.


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
