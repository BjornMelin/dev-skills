# Skia and Reanimated interoperability

Skill: native-skia
Checked at: 2026-06-04

## When To Load

- Read when Skia drawing state is animated by Reanimated shared values, gestures, or UI-thread updates.

## Source Anchors

- https://shopify.github.io/react-native-skia/docs/animations/animations
- https://docs.swmansion.com/react-native-reanimated/docs/fundamentals/getting-started

## Reference Notes

- Keep high-frequency drawing values off React render state. Reanimated shared values can drive Skia properties without forcing React reconciliation.
- Separate drawing primitives, animation values, and app state so each runtime owns the correct work.
- Skia effects that replace many native views should still expose accessible labels and actions through surrounding React Native UI.

## Focused Checks

- Test gesture-driven updates, app background/foreground, resize, and reduced motion.
- Inspect whether derived drawing values allocate objects per frame.

## Failure Modes

- Pushing Skia animation through React state on every frame.
- Canvas-only controls with no accessible native control layer.


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
