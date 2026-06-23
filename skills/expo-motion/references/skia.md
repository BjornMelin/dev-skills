# React Native Skia

`@shopify/react-native-skia` exposes the Skia 2D graphics engine — the renderer behind Chrome, Android, and Flutter — as a declarative React component tree backed by a single GPU-accelerated `Canvas`. On Expo and React Native (Reanimated 4 on the New Architecture), Skia is the right surface when ordinary animated views run out of road: custom vector graphics, charts, particles, generative art, per-pixel shaders, and high-frequency batch drawing of hundreds of elements rendered in a single draw pass.

Skia is a **native module shipped as prebuilt binaries**, not a JS library, so it needs a development build (or the Expo SDK that bundles a compatible version) and does not run in bare Expo Go. This reference covers the component model, direct interop with Reanimated shared values, resource lifecycle/memory, web/CanvasKit boundaries, and accessibility. It does **not** re-explain shared values, worklets, or threading — those live in sibling references and are cross-linked, not duplicated.

## Contents

- [When Skia beats plain Reanimated views](#when-skia-beats-plain-reanimated-views)
- [Setup and assumptions](#setup-and-assumptions)
- [Canvas and core primitives](#canvas-and-core-primitives)
- [Skia ↔ Reanimated interop](#skia--reanimated-interop)
- [Color: use Skia's own `interpolateColors`](#color-use-skias-own-interpolatecolors)
- [Animated paths: `usePathValue` / `usePathInterpolation`](#animated-paths-usepathvalue--usepathinterpolation)
- [Canonical animated Skia component](#canonical-animated-skia-component)
- [Runtime shaders (SKSL)](#runtime-shaders-sksl)
- [Atlas: batched sprites and tiles](#atlas-batched-sprites-and-tiles)
- [Resource lifecycle and memory](#resource-lifecycle-and-memory)
- [Web / CanvasKit boundary](#web--canvaskit-boundary)
- [Accessibility: the canvas is opaque](#accessibility-the-canvas-is-opaque)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## When Skia beats plain Reanimated views

Reanimated animating native views is the default for product motion. Reach for Skia only when the scene exceeds what views handle well:

- **Custom vector graphics** — shapes, gradients, masks, clips, and path effects that would be awkward or impossible with RN `View`s and `react-native-svg`.
- **Many animated elements** — past roughly 100 animated views on low-end Android (~500 on iOS), per-view overhead dominates. Skia renders everything to one canvas, so all elements share one draw-call pipeline.
- **Charts and data visualization** — axes, lines, areas, and labels redrawn together on a single surface.
- **Particles, generative art, sprite sheets** — entities created and destroyed per frame.
- **Per-pixel effects** — blurs, color grading, displacement, and procedural patterns via SKSL runtime shaders.
- **High-frequency batch drawing** — `Atlas` draws hundreds of instances of one texture in a single GPU call.

If the task is ordinary view motion, route to [Reanimated core](./reanimated-core.md) instead. For designer-authored vector assets prefer Lottie/Rive; for true 3D use React Three Fiber.

## Setup and assumptions

- **Reanimated 4 + New Architecture (Fabric, RN 0.76+).** Reanimated 3 / old-architecture guidance does not apply.
- **Development build required.** Skia runs a `postinstall` script that copies prebuilt binaries into the native projects. Package managers that block lifecycle scripts (Bun `trustedDependencies`, Yarn Berry `enableScripts`) fail with missing-native-binary errors. Install through the repo's package policy; with Expo, use `expo install` so the version resolves to the SDK-bundled native module — do not hand-bump it without a native rebuild.
- **Version pinning.** Current Skia requires `react-native@>=0.79` and `react@>=19`; for `react-native@<=0.78` / React 18 stacks, pin `@shopify/react-native-skia@1.12.4` or below. Confirm the version the target repo's Expo SDK bundles before editing — do not assume.
- **Platform minimums.** iOS 14+, Android API 21+ for basic rendering; video/experimental Graphite need Android API 26+. Bundle cost ~+6 MB iOS, +4 MB Android, +2.9 MB web.

## Canvas and core primitives

`Canvas` is the drawing root and a mounted native GPU surface. It accepts normal RN view props (including accessibility props) and needs a non-zero layout size via `style`, explicit width/height, or a measured parent.

```tsx
import {
  Canvas,
  Group,
  Path,
  Circle,
  Rect,
  Paint,
  LinearGradient,
  Image,
  Text,
  Skia,
  vec,
  useImage,
  useFont,
} from '@shopify/react-native-skia';
```

The scene is declared as a React element tree (**retained mode**). Skia compiles it into a display list, so animating a property has near-zero FFI cost — the structure stays fixed while values change. Core building blocks:

- **`Group`** — applies `transform`, `clip`, `layer`, blend mode, and opacity to children. **Its transform origin is the top-left corner, not the center** (unlike RN views) — use the `origin` prop to pivot. **All rotations are in radians.**
- **`Path`** — strokes/fills an `SkPath`. Build once with `Skia.Path.Make()`, `Skia.Path.MakeFromSVGString(...)`, or `addRRect`/`addCircle`; style with `style="stroke"`, `strokeWidth`, `strokeCap`, `strokeJoin`.
- **`Circle`, `Rect`, `RoundedRect`, `Line`, `Oval`** — geometric primitives accepting color and paint props.
- **`Paint`** — describes how subsequent siblings draw (color, style, blend, opacity, filters); primitives can also take paint props inline.
- **`Image`** — draws a decoded `SkImage` with `fit`/`rect`/`sampling`; load via `useImage()`.
- **`Text`** — draws a string with a `Skia.Font`; reuse one font instance (see [memory](#resource-lifecycle-and-memory)).
- **`Atlas`** — batched instanced drawing of one texture (see below).
- **`Shader` / `RuntimeShader`** — per-pixel SKSL effects compiled with `Skia.RuntimeEffect.Make()`.

Paint effects (`LinearGradient`, `BlurMask`, color filters) are declared as children of the element they apply to. `Picture` and `SVG` do **not** inherit `Group` paint — apply effects via a parent `Group`'s `layer` prop. For canvas measurement use the `onSize` prop (shared value, updates on resize) or `useCanvasSize()` (JS-thread state); `onLayout` is deprecated on Fabric. Use `makeImageSnapshotAsync()` for snapshots that include textures.

## Skia ↔ Reanimated interop

This is the defining feature. **Skia components accept Reanimated shared values and derived values directly as props — no wrapper.** There is no `createAnimatedComponent` and no `useAnimatedProps`; pass the shared value straight in and Skia consumes its updates on the UI thread without a React re-render.

```tsx
import { Canvas, Circle, Group } from '@shopify/react-native-skia';
import { useDerivedValue, useSharedValue, withRepeat, withTiming } from 'react-native-reanimated';
import { useEffect } from 'react';

function PulsingCircles() {
  const size = 256;
  const r = useSharedValue(0);
  // useDerivedValue computes a prop from one or more shared values on the UI thread.
  const c = useDerivedValue(() => size - r.value);

  useEffect(() => {
    r.value = withRepeat(withTiming(size * 0.33, { duration: 1000 }), -1);
  }, [r]);

  return (
    <Canvas style={{ flex: 1 }}>
      <Group blendMode="multiply">
        {/* shared value `r` passed DIRECTLY as a prop — no wrapper component */}
        <Circle cx={r} cy={r} r={r} color="cyan" />
        <Circle cx={c} cy={r} r={r} color="magenta" />
        <Circle cx={size / 2} cy={c} r={r} color="yellow" />
      </Group>
    </Canvas>
  );
}
```

Use `useDerivedValue` to compute any prop (a transform array, color, or rect) from shared values. For time-driven animation that needs no gesture or `withTiming`, `useClock()` returns a continuously incrementing shared value (ms since activation) you feed into a derived value:

```tsx
import { Canvas, Circle, useClock, vec } from '@shopify/react-native-skia';
import { useDerivedValue } from 'react-native-reanimated';

function Lissajous() {
  const t = useClock();
  const transform = useDerivedValue(() => {
    const scale = (2 / (3 - Math.cos(2 * t.value))) * 200;
    return [
      { translateX: scale * Math.cos(t.value) },
      { translateY: scale * (Math.sin(2 * t.value) / 2) },
    ];
  });
  return (
    <Canvas style={{ flex: 1 }}>
      <Circle c={vec(0, 0)} r={50} color="cyan" transform={transform} />
    </Canvas>
  );
}
```

Gestures use the standard `react-native-gesture-handler` pattern: wrap the `Canvas` in `GestureDetector`, update shared values in gesture callbacks, and Skia consumes them on the UI thread. Gestures hit the whole canvas; to target one drawn element, overlay an invisible `Animated.View` mirroring its transform and attach the gesture there. The threading rules (where `.value` is safe, `'worklet'` directives, bridging completion to JS) are exactly Reanimated's — see [Worklets & threading](./worklets-threading.md). Keep high-frequency drawing math in shared values and keep app/domain state in React; route changes and backgrounding can remount the canvas and recreate GPU resources, so domain state must live outside it.

## Color: use Skia's own `interpolateColors`

**Skia stores colors in a different format from Reanimated. Reanimated's `interpolateColor` produces wrong results inside Skia.** Use `interpolateColors` (plural) imported from `@shopify/react-native-skia`.

```tsx
import { Canvas, Fill, LinearGradient, interpolateColors, vec } from '@shopify/react-native-skia';
import { useDerivedValue, useSharedValue, withRepeat, withTiming } from 'react-native-reanimated';
import { useEffect } from 'react';

const startColors = ['rgba(34,193,195,0.4)', 'rgba(63,94,251,1)'];
const endColors = ['rgba(0,212,255,0.4)', 'rgba(252,70,107,1)'];

function AnimatedGradient() {
  const progress = useSharedValue(0);
  useEffect(() => {
    progress.value = withRepeat(withTiming(1, { duration: 4000 }), -1, true);
  }, [progress]);

  const colors = useDerivedValue(() => [
    interpolateColors(progress.value, [0, 1], startColors),
    interpolateColors(progress.value, [0, 1], endColors),
  ]);

  return (
    <Canvas style={{ flex: 1 }}>
      <Fill>
        <LinearGradient start={vec(0, 0)} end={vec(256, 256)} colors={colors} />
      </Fill>
    </Canvas>
  );
}
```

The mirror rule: Reanimated's `interpolateColor` is correct for *RN view* `backgroundColor`/`color` (see [Reanimated core](./reanimated-core.md)); Skia's `interpolateColors` is correct for *Skia* color props. They are not interchangeable.

## Animated paths: `usePathValue` / `usePathInterpolation`

Two purpose-built hooks animate paths on the UI thread without re-creating the path each frame.

- **`usePathInterpolation(progress, inputRange, outputPaths)`** morphs between path shapes. **Every path must have the same number and types of commands**, or interpolation is undefined — for structurally different paths, generate compatible intermediates with [flubber](https://github.com/veltman/flubber).

  ```tsx
  const path = usePathInterpolation(progress, [0, 0.5, 1], [angry, normal, happy]);
  // <Path path={path} style="stroke" strokeWidth={5} strokeCap="round" />
  ```

- **`usePathValue(modifier, basePath)`** applies imperative worklet commands to a base path each frame — transforms, including 3D via `processTransform3d`. The base path is built once outside render; the worklet mutates a working copy.

  ```tsx
  const rrct = Skia.Path.Make();
  rrct.addRRect(Skia.RRectXY(Skia.XYWHRect(0, 0, 100, 100), 10, 10));

  const clip = usePathValue((path) => {
    'worklet';
    path.transform(processTransform3d([
      { translate: [50, 50] },
      { perspective: 300 },
      { rotateY: rotateY.value },
      { translate: [-50, -50] },
    ]));
  }, rrct);
  // <Path path={clip} />
  ```

When reusing one mutable path across frames in immediate mode, call `path.rewind()` to clear it and keep its allocated capacity rather than building a fresh `Skia.Path` each frame.

## Canonical animated Skia component

A complete, idiomatic component: resources created once, a shared value driven by a gesture with spring inertia, the shared value passed straight into a Skia prop, and the canvas wrapped in accessible RN semantics.

```tsx
import { useMemo } from 'react';
import { View, useWindowDimensions } from 'react-native';
import { Canvas, Circle, Fill, Group } from '@shopify/react-native-skia';
import { Gesture, GestureDetector } from 'react-native-gesture-handler';
import { useSharedValue, withDecay } from 'react-native-reanimated';

const RADIUS = 28;

export function DraggableOrb() {
  const { width } = useWindowDimensions();
  const x = useSharedValue(width / 2);

  // Static paint config built once, never per frame/render.
  const gradientStops = useMemo(() => ['#22d3ee', '#3b82f6'], []);

  const gesture = Gesture.Pan()
    .onChange((e) => {
      x.value += e.changeX;
    })
    .onEnd((e) => {
      // Momentum + clamp to screen bounds, all on the UI thread.
      x.value = withDecay({ velocity: e.velocityX, clamp: [RADIUS, width - RADIUS] });
    });

  return (
    // Accessible wrapper: the canvas itself is opaque to assistive tech.
    <View
      style={{ flex: 1 }}
      accessible
      accessibilityRole="adjustable"
      accessibilityLabel="Draggable orb"
      accessibilityHint="Swipe left or right to fling the orb"
    >
      <GestureDetector gesture={gesture}>
        <Canvas style={{ flex: 1 }}>
          <Fill color="#0b1220" />
          <Group>
            {/* shared value `x` flows directly into cx — no animated wrapper */}
            <Circle cx={x} cy={120} r={RADIUS} color={gradientStops[0]} />
          </Group>
        </Canvas>
      </GestureDetector>
    </View>
  );
}
```

## Runtime shaders (SKSL)

Skia exposes SKSL (a GLSL-like shading language) for per-pixel effects. Compile **once** with `Skia.RuntimeEffect.Make()` at module scope (or in `useMemo`), check for `null` (compile failure), and animate by passing shared values as `uniforms`.

```tsx
import { Canvas, Fill, Shader, Skia, vec } from '@shopify/react-native-skia';
import { useClock, useDerivedValue } from '@shopify/react-native-skia';

const source = Skia.RuntimeEffect.Make(`
uniform float2 resolution;
uniform float time;

half4 main(float2 pos) {
  float2 uv = pos / resolution;
  float d = length(uv - 0.5);
  float pulse = 0.5 + 0.5 * sin(d * 20.0 - time * 3.0);
  return half4(uv.x, pulse, uv.y, 1.0);
}`);

if (source === null) {
  // Make() returns null on compile failure — never render without checking.
  throw new Error('Failed to compile Skia runtime shader');
}

export function ShaderRipple({ width, height }: { width: number; height: number }) {
  const clock = useClock();
  const uniforms = useDerivedValue(() => ({
    resolution: [width, height],
    time: clock.value / 1000,
  }));

  return (
    <Canvas style={{ flex: 1 }}>
      <Fill>
        <Shader source={source} uniforms={uniforms} />
      </Fill>
    </Canvas>
  );
}
```

A `RuntimeShader` used as an **image filter** receives the filtered drawing as a `shader image` uniform (`image.eval(xy)`). It does **not** account for pixel density — for crisp output, supersample by `PixelRatio.get()` before filtering and scale back down after.

## Atlas: batched sprites and tiles

`Atlas` renders many instances of one texture in a single draw call with per-instance RSXform transforms (`[scos, ssin, tx, ty]`) — ideal for tile maps, sprite grids, and hundreds of similar objects. Build the texture with `useTexture` (UI thread), keep sprite rectangles stable with `useMemo`, and animate transforms with `useRSXformBuffer`. Per-frame array allocation erases the draw-call win — allocate buffers once and mutate in place inside the worklet, and confirm the texture is non-null before drawing.

## Resource lifecycle and memory

Images, fonts, paths, shaders, and runtime effects are **resources with load/cache/invalidation cost** — the single biggest source of Skia bugs is recreating them per render or per frame.

- **`useImage()` is async** — returns `null` until the decode finishes, accepts an error handler. Render explicit loading/error/fallback states; never assume the image is ready. Sources can be `require(...)`, native bundle names, network URLs, or encoded bytes. **Skia's image cache evicts at roughly 10 MB** — large or many decoded images get evicted and re-decoded, causing churn; budget decode size deliberately on low-end devices. Never fetch or decode inside a worklet/frame callback — load on a supported path, then pass a stable handle into the scene.
- **Fonts** — create one `Skia.Font()` (or one `useFont(...)` per family/size) and reuse it across all `Text`. Allocating a font per render leaks and stutters.
- **Paths** — build `Skia.Path` instances outside render (module scope) or wrap in `useMemo`. For per-frame reuse in immediate mode, `rewind()` an existing path instead of allocating a new one.
- **Shaders** — compile `Skia.RuntimeEffect.Make()` once (module scope or `useMemo`); recompiling per frame is a severe stall.
- **`LinearGradient` / `Image` re-render leaks** — re-creating gradient/image elements on every render can accumulate native memory. Keep their inputs stable (memoized colors/positions, stable image handles) so the display list does not thrash.
- **Canvas lifecycle** — the canvas is a native GPU surface; route transitions, conditional rendering, and backgrounding can destroy and recreate its resources. Keep domain state outside the canvas and rebuild scene resources intentionally on remount.

Validate memory and lifecycle explicitly: missing/failed resources, theme changes, screen unmount/remount, app background/foreground, resize, and memory-heavy surfaces on representative low-end Android. See [Validation](./validation.md).

## Web / CanvasKit boundary

Skia web is **not** the same module as native — it runs CanvasKit (WASM) loaded **asynchronously**, so Skia web components must not be imported before CanvasKit initializes.

- Use `WithSkiaWeb` for code-splitting, or `LoadSkiaWeb()` to defer root registration until WASM is ready.
- Expo Router dev mode can evaluate files under `app/` before CanvasKit loads — keep lazily loaded Skia components **outside** `app/` on that lane.
- After upgrading the package, run `setup-skia-web` when relying on a local `canvaskit.wasm`; CDN-loaded CanvasKit must match the installed version.
- Browsers cap active WebGL contexts; for many static canvases, `__destroyWebGLContextAfterRender` exists but has a performance cost.

**Web behavior never proves native behavior** and vice versa — rendering, memory, and shader results can differ. Validate each platform you ship. Jest uses Skia's `@shopify/react-native-skia/jestEnv.js` environment and `jestSetup.js`, which load a CanvasKit-backed mock.

## Accessibility: the canvas is opaque

**A Skia canvas is pixels, not semantic UI.** Screen readers see nothing inside it — no labels, no focus targets, no hit areas. Treat drawn content as decorative-by-default and supply meaning through the surrounding React Native tree:

- Wrap the canvas (or its container) with accessible RN semantics: `accessible`, `accessibilityRole`, `accessibilityLabel`/`accessibilityHint`, `accessibilityValue` for charts/sliders, and `accessibilityState`.
- For interactive drawn elements, overlay real RN controls (`Pressable`, `Animated.View`) that carry the labels, focus, and hit targets — the same overlay pattern used for per-element gestures.
- Honor reduced motion: pause, simplify, or replace decorative loops, animated images, particle systems, and shader motion when the system setting is on. Keep functional feedback.

Full reduced-motion, haptics, frame-budget, and device-proof guidance lives in [Accessibility & performance](./accessibility-performance.md) — this section only states the canvas-specific caveat.

## Pitfalls / Do-not

- **Do not recreate Images, Fonts, Paths, or shaders per render or per frame.** Load `useImage()` once and handle its `null`/error states; reuse a single `Skia.Font()`; build `Skia.Path` in module scope or `useMemo` (`rewind()` to reuse); compile `Skia.RuntimeEffect.Make()` once and check for `null`. Recreating them stalls frames and leaks native memory (the image cache evicts ~10 MB).
- **Do not use Reanimated's `interpolateColor` inside Skia.** Skia's color format differs; use `interpolateColors` from `@shopify/react-native-skia`. The reverse is also wrong — don't use Skia's color interp on RN views.
- **Do not ship an opaque canvas without accessibility.** Wrap it in accessible RN semantics and overlay real controls for interactive elements; the canvas has no semantic nodes of its own.
- **Do not assume web == native.** CanvasKit (WASM, async-loaded) is a separate path with its own setup, memory, and WebGL-context limits. Validate each platform; one is not proof of the other.
- **Do not push per-frame Skia values through React `setState`.** Drive them with shared values consumed directly by Skia props on the UI thread.
- **Do not allocate inside frame worklets** (`useDerivedValue`, `usePathValue`, `useRSXformBuffer`, immediate-mode `Picture`). Precompute constants, mutate buffers in place, and `rewind()` reused paths.
- **Do not forget Skia is a native module.** It needs a development build and lifecycle scripts; it is not pure JS and does not work in bare Expo Go without the bundled version.
- **Do not assume RN view conventions.** Skia `Group` transform origin is top-left (use `origin`), and all rotations are in radians.

## Related references

- [Reanimated core](./reanimated-core.md)
- [Worklets & threading](./worklets-threading.md)
- [Accessibility & performance](./accessibility-performance.md)
- [Validation](./validation.md)
- [Recipes](./recipes.md)
