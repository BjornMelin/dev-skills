# Asset & 3D motion (Lottie, Rive, R3F)

These three libraries run **alongside** Reanimated, not instead of it. For
code-driven product motion — transitions, gestures, layout, scroll — reach for
Reanimated (see [Reanimated core](./reanimated-core.md)); it owns the common case
on the UI thread. Reach for the libraries below **only** for the asset and 3D
cases they exist to handle: a designer-authored vector animation (Lottie), an
interactive stateful illustration (Rive), or a 3D scene that is itself the
product surface (R3F). Each is a separate native dependency with its own asset
contract and lifecycle, so add it deliberately. **Moti is inactive** — it is a
legacy Reanimated-3 wrapper and is **not recommended for new code**; use the
Reanimated 4 API directly.

For every library: inspect the target repo's installed versions before applying
any snippet here, and route deep work to the official docs / context7 rather than
this brief.

## Lottie

**When:** a designer hands you an After Effects vector animation (loaders,
success checks, onboarding illustrations) exported as Lottie JSON or `.lottie`.
Playback is timeline-driven, not interactive.

**Package:** `lottie-react-native` (renders `LottieView`); `.lottie`
(dotLottie) is the compact, multi-animation container format.

Bundle the asset through the app's asset pipeline with a static `require` — do
not use a web-style URL. Let the owning component hold the `ref` and control
play/pause/reset; never expose a globally reachable animation handle.

```tsx
import { useEffect, useRef } from 'react';
import { AccessibilityInfo, View } from 'react-native';
import LottieView from 'lottie-react-native';

export function SuccessCheck() {
  const ref = useRef<LottieView>(null);

  useEffect(() => {
    let active = true;
    AccessibilityInfo.isReduceMotionEnabled().then((reduce) => {
      if (!active) return;
      // Under reduced motion, jump to the final frame instead of looping.
      if (reduce) ref.current?.play(150, 150);
      else ref.current?.play();
    });
    return () => {
      active = false;
      ref.current?.reset(); // stop + free the timeline on unmount
    };
  }, []);

  return (
    <View accessible accessibilityLabel="Payment confirmed" accessibilityRole="image">
      <LottieView
        ref={ref}
        source={require('../assets/success.lottie')}
        autoPlay={false}
        loop={false}
        style={{ width: 160, height: 160 }}
      />
    </View>
  );
}
```

**Cleanup + reduced motion:** pause/reset on unmount and on screen blur so a
loop does not keep running behind a hidden route; under reduced motion, skip the
loop or jump to the end frame. Large JSON animations hurt startup and memory —
prefer `.lottie` and trim unused layers. The animation view is not accessible on
its own, so wrap it with a labelled `accessible` container, and never use
animation progress as the only signal of completion.

**Depth:** Expo Lottie docs (`docs.expo.dev/versions/latest/sdk/lottie`) and the
`lottie-react-native` README via context7.

## Rive

**When:** an interactive vector animation or stateful UI illustration — a toggle,
a reactive mascot, a progress widget — where app state drives the visual through
a **state machine**, not a fixed timeline.

**Package:** `@rive-app/react-native` (Nitro runtime). Rive needs native modules,
so it requires a **development build** and will not run in Expo Go.

The `.riv` file's **state machine name and input names are the asset contract** —
they must match exactly. Drive boolean / number / trigger inputs from app state
and reset them on unmount.

```tsx
import { useEffect, useRef } from 'react';
import { View } from 'react-native';
import Rive, { RiveRef } from 'rive-react-native';

export function LikeButton({ liked }: { liked: boolean }) {
  const ref = useRef<RiveRef>(null);

  useEffect(() => {
    // Map app state onto a named boolean input in the named state machine.
    ref.current?.setInputState('LikeMachine', 'isLiked', liked);
  }, [liked]);

  useEffect(() => () => ref.current?.reset(), []); // reset on unmount

  return (
    <View accessible accessibilityRole="button" accessibilityLabel="Like">
      <Rive
        ref={ref}
        resourceName="like" // bundled .riv (no extension)
        stateMachineName="LikeMachine"
        autoplay
        style={{ width: 56, height: 56 }}
      />
    </View>
  );
}
```

**Cleanup + reduced motion / accessibility:** reset inputs and let the component
tear down on unmount; the rendered surface is canvas-like and exposes no
semantics, so supply surrounding accessible roles/labels and a non-animated
fallback for the state it represents. Under reduced motion, set inputs to their
resting state rather than triggering elaborate transitions.

**Depth:** Rive React Native docs (`rive.app/docs`) and `@rive-app/react-native`
via context7; confirm input/state-machine names against the actual `.riv`.

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
