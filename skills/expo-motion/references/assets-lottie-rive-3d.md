# Asset & 3D motion (Lottie, Rive, R3F)

These three libraries run **alongside** Reanimated, not instead of it. For
code-driven product motion — transitions, gestures, layout, scroll — reach for
Reanimated (see [Reanimated core](./reanimated-core.md)); it owns the common case
on the UI thread. Reach for the libraries below **only** for the asset and 3D
cases they exist to handle: a designer-authored vector animation (Lottie), an
interactive stateful illustration (Rive), or a 3D scene that is itself the
product surface (R3F). Each is a separate native dependency with its own asset
contract and lifecycle, so add it deliberately. Do not add a wrapper solely for
declarative syntax; if an app already uses Moti or another wrapper, follow the
installed package's migration guidance before changing ownership.

For every library: inspect the target repo's installed versions before applying
any snippet here, and route deep work to the official docs / context7 rather than
this brief.

## Lottie

**When:** a designer hands you an After Effects vector animation (loaders,
success checks, onboarding illustrations) exported as Lottie JSON.
Playback is timeline-driven, not interactive.

**Package:** `lottie-react-native` (renders `LottieView`). Use the source format
documented by the installed package; this example uses JSON. **UNVERIFIED for
the renderer:** do not assume that a `.lottie`/dotLottie asset is accepted
without checking the installed `lottie-react-native` release.

Bundle the asset through the app's asset pipeline with a static `require` — do
not use a web-style URL. Let the owning component hold the `ref` and control
play/pause/reset; never expose a globally reachable animation handle.

```tsx
import { useEffect, useRef, useState } from 'react';
import { AccessibilityInfo, View } from 'react-native';
import LottieView from 'lottie-react-native';

export function SuccessCheck() {
  const ref = useRef<LottieView>(null);
  const [progress, setProgress] = useState<number>();

  useEffect(() => {
    let active = true;
    AccessibilityInfo.isReduceMotionEnabled().then((reduce) => {
      if (!active) return;
      // Under reduced motion, keep a stable frame; choose an asset-specific
      // completion frame only when that frame is part of the asset contract.
      if (reduce) {
        setProgress(1); // Lottie progress is normalized; this asset ends in success.
      } else {
        setProgress(undefined);
        ref.current?.play();
      }
    });
    return () => {
      active = false;
      ref.current?.reset(); // stop playback and return to the initial frame
    };
  }, []);

  return (
    <View accessible accessibilityLabel="Payment confirmed" accessibilityRole="image">
      <LottieView
        ref={ref}
        source={require('../assets/success.json')}
        autoPlay={false}
        loop={false}
        progress={progress}
        style={{ width: 160, height: 160 }}
      />
    </View>
  );
}
```

**Cleanup + reduced motion:** pause/reset on unmount and on screen blur so a
loop does not keep running behind a hidden route; under reduced motion, skip the
loop or jump to the end frame. Large animation assets hurt startup and memory —
trim unused layers. The animation view is not accessible on
its own, so wrap it with a labelled `accessible` container, and never use
animation progress as the only signal of completion.

**Depth:** the [`lottie-react-native` API reference](https://github.com/lottie-react-native/lottie-react-native/blob/master/docs/api.md)
and Expo's [asset guidance](https://docs.expo.dev/versions/latest/sdk/asset/).

## Rive

**When:** an interactive vector animation or stateful UI illustration — a toggle,
a reactive mascot, a progress widget — where app state drives the visual through
a **state machine**, not a fixed timeline.

**Stable lane:** `@rive-app/react-native` v0.4.19+ with
`react-native-nitro-modules`. Its async API uses `RiveView`, `useRiveFile`,
`useRive`, and data-binding hooks; it is a different API from the legacy
`rive-react-native` package. The Nitro-based v0.5 runtime is currently a beta;
follow the [Rive migration guide](https://rive.app/docs/runtimes/react-native/migration-guide)
before opting into it. Rive needs native modules, so check the target Expo
SDK's Expo Go list and use a **development build** when it is not bundled or
native configuration is required.

The `.riv` file's **state machine, view-model, and property names are the asset
contract** — they must match exactly. Use the installed runtime's data-binding
hooks to drive boolean / number / trigger inputs from app state and let the view
reset when it unmounts.

For a local `.riv` `require(...)`, add `riv` to Metro's `resolver.assetExts`
before using the hook (or verify the target repo's existing asset pipeline):

```js
const { getDefaultConfig } = require('expo/metro-config');

const config = getDefaultConfig(__dirname);
config.resolver.assetExts.push('riv');

module.exports = config;
```

```tsx
import { useState } from 'react';
import { Pressable, Text, View } from 'react-native';
import { useReducedMotion } from 'react-native-reanimated';
import {
  Fit,
  RiveView,
  useRiveBoolean,
  useRiveFile,
  useViewModelInstance,
} from '@rive-app/react-native';

export function LikeButton() {
  const [liked, setLiked] = useState(false);
  const reduce = useReducedMotion();
  const {
    riveFile,
    isLoading: isFileLoading,
    error: fileError,
  } = useRiveFile(require('../assets/like.riv'));
  const {
    instance,
    isLoading: isInstanceLoading,
    error: instanceError,
  } = useViewModelInstance(riveFile, { async: true });
  const [riveError, setRiveError] = useState<string | null>(null);
  const { setValue: setRiveLiked } = useRiveBoolean('liked', instance);
  const loading = isFileLoading || isInstanceLoading;
  const errorMessage =
    riveError ?? fileError?.message ?? instanceError?.message ?? null;

  const toggle = () => {
    const next = !liked;
    setLiked(next);
    if (!reduce && instance && !errorMessage) setRiveLiked(next);
  };

  return (
    <Pressable
      accessibilityRole="button"
      accessibilityLabel={liked ? 'Unlike' : 'Like'}
      accessibilityState={{ checked: liked }}
      onPress={toggle}
    >
      {reduce || loading || errorMessage || !riveFile || !instance ? (
        <View
          style={{
            width: 56,
            height: 56,
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <Text>{liked ? '♥' : '♡'}</Text>
        </View>
      ) : (
        <View pointerEvents="none">
          <RiveView
            file={riveFile}
            dataBind={instance}
            fit={Fit.Layout}
            style={{ width: 56, height: 56 }}
            onError={(error) => setRiveError(error.message)}
          />
        </View>
      )}
      {loading ? (
        <Text accessibilityLiveRegion="polite">Loading animation…</Text>
      ) : null}
      {errorMessage ? (
        <Text accessibilityLiveRegion="polite">Animation unavailable</Text>
      ) : null}
      <Text>{liked ? 'Unlike' : 'Like'}</Text>
    </Pressable>
  );
}
```

This sample assumes the asset's default state machine binds a boolean view-model
property named `liked`; replace that contract with the names defined by the
`.riv` file. `useViewModelInstance(..., { async: true })` and the typed
`useRiveBoolean` hook are the current data-binding path. The outer `Pressable`
owns the button semantics and keeps the native canvas out of the touch target.
`useReducedMotion()` is a snapshot for this example; when it is enabled, the
button keeps the React/accessibility state and renders a static glyph instead of
updating the animated Rive state machine.
The file and view-model hooks expose loading/error state, so the example gates
`RiveView`, keeps the button functional with a static fallback while assets load
or fail, and records runtime failures through `RiveView`'s `onError` callback.

**Legacy migration lane:** `rive-react-native` exposes `Rive`/`RiveRef` and
methods such as `setInputState`; keep that API only while following the
[Rive migration guide](https://rive.app/docs/runtimes/react-native/migration-guide)
for an existing app. New work should use the current runtime and its data
binding contract.

**Cleanup + reduced motion / accessibility:** reset inputs and let the component
tear down on unmount; the rendered surface is canvas-like and exposes no
semantics, so supply surrounding accessible roles/labels and a non-animated
fallback for the state it represents. Under reduced motion, keep the functional
state update but do not trigger an animated state-machine transition.

**Depth:** [Rive React Native](https://rive.app/docs/runtimes/react-native/react-native),
[Rive ref methods](https://rive.app/docs/runtimes/react-native/rive-ref-methods),
and the migration guide above; confirm input/state-machine names against the
actual `.riv`.

## R3F native

**When:** a real-time 3D scene is *the product surface* (a product viewer,
configurator, game-like view) — not for decorating 2D UI, where Reanimated or
Skia is the right tool. 3D on device carries real GPU and battery cost; reach for
it deliberately.

**Package:** `@react-three/fiber/native` + `three`, on top of a GL/WebGPU
surface (`expo-gl`, or `react-native-wgpu` for WebGPU). Asset loaders differ from
web — load GLTF/textures through Expo's asset module, not browser URLs.

```tsx
import { Canvas, useFrame } from '@react-three/fiber/native';
import { useRef } from 'react';
import type { Mesh } from 'three';

function SpinningBox() {
  const mesh = useRef<Mesh>(null);
  useFrame((_, delta) => {
    if (mesh.current) mesh.current.rotation.y += delta; // per-frame, on the GL thread
  });
  return (
    <mesh ref={mesh}>
      <boxGeometry args={[1, 1, 1]} />
      <meshStandardMaterial color="orange" />
    </mesh>
  );
}

export function Viewer() {
  return (
    <Canvas dpr={1.5} camera={{ position: [0, 0, 4] }}>
      <ambientLight />
      <directionalLight position={[2, 4, 2]} />
      <SpinningBox />
    </Canvas>
  );
}
```

**Cleanup + quality:** own DPR/quality yourself (device pixel ratios vary widely)
and **dispose GPU resources** — geometries, materials, textures, loaded models —
on unmount; the `Canvas` unmount cleans the renderer, but assets you create or
load must be released or they leak GPU memory. Pause `useFrame` work when the
screen is not focused, and respect reduced motion by stopping idle rotation /
auto-orbit. Provide a non-3D fallback for accessibility since the canvas has no
semantics.

**Depth:** R3F docs (`r3f.docs.pmnd.rs`), Three.js docs, and the Expo GL /
WebGPU guides via context7. **Browser R3F examples assume DOM/WebGL APIs that do
not exist on native** — never copy a web example unchanged. GPU changes need
proof on a real device / development build, not just a passing type-check.

## Related references

- [Reanimated core](./reanimated-core.md)
- [Skia](./skia.md)
- [Decision matrix](./decision-matrix.md)
- [Validation](./validation.md)
