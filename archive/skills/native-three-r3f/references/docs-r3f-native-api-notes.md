# R3F Native API Notes

Use this file when implementing or reviewing native Three.js / R3F scenes. It
summarizes current official docs and package source evidence; verify installed
versions in the target repo before relying on exact APIs.

## Authority Map

- R3F native: `@react-three/fiber/native`.
- Drei native helpers: `@react-three/drei/native`.
- Native GL surface: `expo-gl` `GLView`, wrapped by R3F native `Canvas`.
- Asset bridge: `expo-asset`, plus Metro asset extensions.
- Three.js scene graph: `three`.
- Legacy/manual bridge: `expo-three`, only when the repo already owns manual
  `GLView` renderer code or needs its helper utilities.

R3F is a React renderer and pairs with the React major version. Use R3F v8 with
React 18 and R3F v9 with React 19. Drei must also satisfy the installed R3F
peer range.

## Native Entry Points

Use native imports in React Native files:

```tsx
import { Canvas, useFrame } from '@react-three/fiber/native';
import { useGLTF } from '@react-three/drei/native';
```

Avoid web imports in native files:

```tsx
import { Canvas } from '@react-three/fiber';
import { Html } from '@react-three/drei';
```

Drei's native route does not export every web helper; `Html` and `Loader` are
web-only. Check the installed package before assuming a helper is native-safe.

## Canvas Setup

R3F native `Canvas` wraps `expo-gl` `GLView`, measures layout with React Native,
sets DPR from `PixelRatio`, creates a native-compatible canvas shim, and calls
`gl.endFrameEXP()` after render. Prefer this path over hand-written `GLView`
renderers unless the repo already owns manual Three.js rendering.

Native `Canvas` still needs app-owned layout discipline:

- wrap it in a measured RN view with stable dimensions;
- avoid placing essential labels or controls inside the GL scene;
- keep route transitions and conditional mounting from repeatedly recreating
  expensive scenes without cleanup;
- use a fallback view for missing GL context, failed asset load, unsupported
  device, or reduced-motion preference when the 3D motion is decorative.

For manual `GLView` work:

- set the renderer size from `gl.drawingBufferWidth` and
  `gl.drawingBufferHeight`;
- call `gl.endFrameEXP()` after each presented frame;
- handle resize/layout changes explicitly;
- cancel `requestAnimationFrame` or scheduler loops on unmount;
- dispose the renderer and native GL objects on unmount where supported.

## Assets

Use static imports or `require()` for local GLB/GLTF/textures so Metro can
bundle them. Add only needed extensions to Metro `assetExts`, commonly:

```js
['glb', 'gltf', 'bin', 'png', 'jpg', 'jpeg', 'ktx2', 'obj', 'mtl']
```

`expo-asset` can embed configured files at build time and can load runtime
assets with `Asset.loadAsync()` or `useAssets()`. A downloaded asset exposes a
`localUri`; Expo GL texture upload paths often require a local file URI, not a
remote or packager URI.

`useLoader()` caches loaded resources for the same URL. Shared cached resources
need an explicit disposal/lifetime policy before using `dispose={null}`.

For Drei `useGLTF` and `useTexture` on native, verify the helper exists on
`@react-three/drei/native`, keep the import static when bundling local files,
and route remote assets through an app-owned cache/error/offline policy before
release.

## Performance

- Prefer `frameloop="demand"` for static or interaction-only scenes.
- Call `invalidate()` after imperative camera/control changes in demand mode.
- Use refs and Three object mutation in `useFrame`; do not write React state
  every frame.
- Reuse geometries/materials/textures. Use `instancedMesh` for repeated meshes.
- Use progressive loading and level of detail for heavy GLTFs.
- Keep draw calls, material count, texture sizes, shadows, post-processing, and
  DPR within mobile budgets.

## Cleanup

R3F disposes many declarative scene resources when it owns them. Manual objects
created outside JSX still need explicit cleanup:

- controls and event listeners;
- geometries, materials, textures, render targets, mixers, and custom loaders;
- animation loops, timers, subscriptions, and remote asset requests;
- manual `GLView` renderers and GL objects.

For `AnimationMixer`, create one mixer per animated root, update it from
`useFrame((_state, delta) => mixer.update(delta))`, stop/uncache actions when
the animated root is removed if the mixer owns them, and avoid recreating
actions on every React render.

Route unmount/remount is the fastest way to catch leaks, stale event handlers,
and context exhaustion.

## Accessibility And UX

A GL canvas does not provide a screen-reader tree. Put controls, labels,
buttons, focus targets, and explanatory state in React Native views layered over
or around the canvas. Provide reduced-motion and non-3D fallbacks when animation
is decorative or the GL scene is product-critical.

## Validation Checklist

- Package alignment: Expo SDK, React Native, React, R3F, Drei, Three,
  `expo-gl`, `expo-asset`, and Metro asset extensions.
- Native build: development build or EAS proof when adding native dependencies,
  config plugins, or asset pipeline changes.
- Runtime: iOS and Android rendering, physical iOS device when simulator GL is
  unreliable, model/texture load, resize/orientation, route unmount/remount,
  background/foreground, memory, frame rate, and error/fallback states.
- Web parity: only if the same components are expected to render on Expo web or
  a separate web app.
