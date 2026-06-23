# Skia web and Expo boundaries

Skill: native-skia
Checked at: 2026-06-04

## When To Load

- Read for CanvasKit, Expo package compatibility, and platform proof.


## Operating Guidance

React Native Skia canvas animations, drawing primitives, Reanimated integration, CanvasKit/web caveats, performance, memory, and Expo/native validation.

### Decision Boundaries

- Use native-motion-core for ordinary view motion.
- Use native-three-r3f for 3D scenes.
- Use native-lottie/native-rive for designer-authored assets.

### Workflow Details

1. Check Skia, Expo SDK, platform support, and web requirements.
2. Keep drawing state and animation values in the appropriate runtime.
3. Avoid many animated native views when one canvas is the right surface.
4. Validate memory, resize, background/foreground, and platform rendering.

### Gotchas

- Skia web may require CanvasKit setup.
- Canvas output needs accessible surrounding UI.
- Large paths/images/shaders need memory and lifecycle ownership.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
