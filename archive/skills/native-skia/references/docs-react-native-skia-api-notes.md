# React Native Skia API Notes

Use this file when implementing or reviewing Skia code. It summarizes the
current upstream docs and package source without replacing them.

## Version And Install

- Expo SDK 56 bundles `@shopify/react-native-skia@2.6.2` for Android, iOS,
  tvOS, web, and Expo Go. Prefer this version for Expo Go-compatible SDK 56
  apps unless a custom dev build justifies a newer native module.
- Upstream latest checked on 2026-06-04 was
  `@shopify/react-native-skia@2.6.4`. It depends on
  `canvaskit-wasm@0.41.0` and Skia binary packages `147.1.0`, with peer
  metadata for React 19, React Native, and Reanimated. See `source-ledger.md`
  before changing pins.
- Current Skia docs require modern React Native and React for current releases;
  the docs minimum is safer than package metadata when they differ. Older
  RN/React stacks may need an older Skia package.
- The package runs a postinstall script to copy native binaries. Package
  managers that block lifecycle scripts can produce missing-native-binary
  failures.
- Video support and experimental Graphite have stricter Android API
  requirements than basic Skia rendering.

## Canvas Size And Snapshots

- `Canvas` is the Skia drawing root and accepts normal RN view props, including
  accessibility props.
- Use `style`, explicit width/height, or a measured parent so the canvas has a
  nonzero layout size.
- Treat `Canvas` as a mounted native surface with its own renderer. Route
  transitions, keys, backgrounding, and conditional rendering can destroy and
  recreate GPU resources; keep domain state outside the canvas and rebuild
  scene resources intentionally.
- `onSize` accepts a Reanimated shared value and updates when the canvas size
  changes. Prefer it when drawing math belongs on the UI thread.
- `useCanvasSize()` returns a ref plus JS-thread size for cases that truly need
  React state or JS measurements.
- `onLayout` canvas measurement is deprecated on Fabric; use `onSize` or
  `useCanvasSize()`.
- Prefer `makeImageSnapshotAsync()` for snapshots that include textures because
  it runs on the UI thread.
- `androidWarmup` is opt-in. Use it only for static or fully opaque drawings
  after Android proof.

## Reanimated Integration

- Skia supports Reanimated shared and derived values as component props.
  `createAnimatedComponent` and `useAnimatedProps` are usually unnecessary.
- Use `interpolateColors` from `@shopify/react-native-skia`; Reanimated
  `interpolateColor` uses a different color representation.
- Skia animation hooks include path interpolation/value hooks, `useClock`,
  `useRectBuffer`, and `useRSXformBuffer`.
- Keep high-frequency calculations in worklets/shared values. Avoid reading
  `.value` on the JS thread in render paths, feeding React state every frame,
  or allocating fresh arrays/objects inside derived values unless the API
  requires it and the cost is measured.
- Derived values should close over stable inputs only. Move changing props,
  dimensions, colors, and asset handles into shared values, memoized constants,
  or explicit dependencies.
- Gesture handlers should update shared values on the UI thread and let Skia
  consume them directly. Use RN overlays for per-element gestures and hit
  targets when the canvas drawing itself has no semantic node.

## Images And Shaders

- `useImage()` returns `null` until loading finishes and accepts an optional
  error handler. Render loading/error/fallback states deliberately.
- `useAnimatedImageValue()` returns a shared value for GIF/WebP frames and can
  take a shared paused flag.
- Asset sources can be JS-bundled `require(...)`, native bundle names, network
  URLs, encoded bytes, or generated pixel data. Check decode size, cache
  pressure, retry/error handling, and placeholder layout before animating.
- Do not fetch, decode, or allocate image data inside worklets or frame-driven
  callbacks. Load on the supported asset path, then pass stable image/texture
  handles into the scene.
- `ImageShader` can draw an image as a shader with fit/rect/transform/sampling
  controls.
- `Skia.RuntimeEffect.Make()` returns `null` on shader compile failure. Check it
  before rendering a `Shader` or `RuntimeShader`.
- Runtime shader image filters need pixel-density testing because upstream docs
  call out scaling behavior.

## Web And Tests

- Web uses CanvasKit WASM loaded asynchronously. Skia web components must not be
  imported before CanvasKit is initialized.
- Use `WithSkiaWeb` for code-splitting or `LoadSkiaWeb()` to defer root
  registration.
- Expo Router dev mode can evaluate files under `app/` before CanvasKit loads;
  put lazily loaded Skia components outside `app/` when using that lane.
- Run `setup-skia-web` after package upgrades when relying on a local
  `canvaskit.wasm`; CDN loading must match the installed CanvasKit version.
- Browsers limit active WebGL contexts. For many static canvases, Skia exposes
  `__destroyWebGLContextAfterRender`, but it has a performance cost.
- Jest support uses Skia's custom environment and setup file:
  `@shopify/react-native-skia/jestEnv.js` and
  `@shopify/react-native-skia/jestSetup.js`.
- Validation should cover native rendering, resize, remount/unmount, memory,
  reduced motion, image fallback/error states, shader compile failure, and web
  CanvasKit only when web is in scope.

## Accessibility And Reduced Motion

- Treat canvas content as pixels, not semantic UI. Use RN overlays or adjacent
  controls for labels, focus targets, hit areas, and screen-reader content.
- Pause, simplify, or replace decorative loops, animated images, particles, and
  shader motion when reduced motion is enabled.
