# R3F Cinematic Art-Direction

Look-dev recipes for making an **already-working** R3F scene cinematic:
postprocessing, HDRI/PBR lighting, shadows, transmissive materials, tone mapping,
and camera work. Setup, lifecycle, disposal, and DPR belong to `web-three-r3f` —
this reference is the aesthetic layer only.

> **Verify against the repo's installed versions before editing.** APIs and props
> below were confirmed 2026-07 against the versions in the table. See
> `provenance.json` for sources.

## Version baseline (2026)

| Package | Version | Notes |
|---|---|---|
| `three` | 0.185.1 (r185) | — |
| `@react-three/fiber` | 9.6.1 | React `>=19 <19.3` |
| `@react-three/drei` | 10.7.7 | R3F v9 / React 19 |
| `@react-three/postprocessing` | 3.0.4 | — |
| `postprocessing` (core) | 6.39.2 | peer **`three >=0.168 <0.186`** |

⚠️ **Load-bearing pin:** `postprocessing` core caps at `three < 0.186`, so
`three@0.185` is the last supported release. Do not bump three to 0.186+ in a
scene that uses `@react-three/postprocessing` until pmndrs ships a new
`postprocessing`. Pin `three` to `~0.185`.

## 1. Postprocessing quality ladder

Named exports from `@react-three/postprocessing`; enums from core `postprocessing`.
Effects apply in **child order**; `EffectComposer` merges compatible effects into
one pass. Effects have no `enabled` prop — conditionally render/unmount the child.

```jsx
import { EffectComposer, N8AO, DepthOfField, Bloom, ToneMapping, SMAA, Vignette } from '@react-three/postprocessing'
import { ToneMappingMode } from 'postprocessing'
import * as THREE from 'three'

<EffectComposer
  multisampling={8}                       // MSAA; use 0 + <SMAA/> for cheaper AA
  frameBufferType={THREE.HalfFloatType}    // HDR buffer — needed for correct bloom/tonemap
>
  <N8AO aoRadius={6} distanceFalloff={1} intensity={2} quality="high" halfRes />
  <DepthOfField focusDistance={0} focalLength={0.02} bokehScale={4} />
  <Bloom mipmapBlur intensity={1.2} luminanceThreshold={0.9} luminanceSmoothing={0.025} />
  <ToneMapping mode={ToneMappingMode.AGX} />
  <SMAA />
  <Vignette offset={0.3} darkness={0.6} />
</EffectComposer>
```

- **`N8AO`** is the current-best ambient occlusion — prefer it over legacy `SSAO`.
  It computes normals from depth, so it does **not** need `enableNormalPass`
  (legacy `SSAO` does). `quality`: `performance | low | medium | high | ultra`.
- **`Bloom`**: set `mipmapBlur` for the modern soft cinematic bloom;
  `luminanceThreshold` gates what glows.
- **`DepthOfField`**: `focusDistance` is normalized 0–1, or use `target={[x,y,z]}`
  for continuous autofocus.
- **`ToneMapping`** default mode is `AGX`. Don't double-tonemap — if you tone-map
  here, set the renderer to `NoToneMapping` (`<Canvas flat>`); see §3.
- **`SMAA`** + `multisampling={0}` swaps MSAA for cheaper post-AA.

**Selective bloom (preferred, cheapest):** bloom is selective by default — lift
emissive/color out of the 0–1 range and mark it `toneMapped={false}`:

```jsx
<meshStandardMaterial emissive="cyan" emissiveIntensity={3} toneMapped={false} />
```

The explicit `<Selection>` + `<Select enabled>` + `<SelectiveBloom lights={[…]} />`
path still works but is heavier and requires a `lights` prop.

## 2. drei look-dev helpers

Named exports from `@react-three/drei`.

```jsx
import {
  Environment, Lightformer, ContactShadows, AccumulativeShadows,
  RandomizedLight, MeshTransmissionMaterial, CameraControls, Float, Stage,
} from '@react-three/drei'
```

- **`Environment`** — HDRI image-based lighting (+ optional background):
  `<Environment preset="studio" background backgroundBlurriness={0.5} environmentIntensity={1} />`
  or `files="/hdri/studio_1k.hdr"`. Presets: `city sunset dawn night warehouse
  forest apartment studio park lobby`. Place `<Lightformer form="rect" .../>`
  children inside for custom studio softboxes.
- **`ContactShadows`** — cheap grounded shadow: `<ContactShadows position={[0,-0.99,0]}
  opacity={0.5} scale={10} blur={2} far={4} resolution={256} />` (`frames={1}` to
  bake once for a static subject).
- **`AccumulativeShadows` + `RandomizedLight`** — raytraced-looking soft shadows,
  free once converged (needs `shadows` on `<Canvas>`):
  ```jsx
  <AccumulativeShadows temporal frames={100} scale={10} opacity={0.85} alphaTest={0.9}>
    <RandomizedLight amount={8} radius={5} ambient={0.5} intensity={1} position={[5,5,-10]} bias={0.001} />
  </AccumulativeShadows>
  ```
- **`MeshTransmissionMaterial`** — glass/refraction (extends `meshPhysicalMaterial`):
  `<MeshTransmissionMaterial transmission={1} thickness={0.5} roughness={0}
  chromaticAberration={0.03} anisotropicBlur={0.1} samples={6} resolution={256} />`.
  For many glass objects, share one FBO (`transmissionSampler` / a manual
  `useFBO()`) to avoid N render passes.
- **`CameraControls`** — imperative `setLookAt`/`dollyTo`/`rotateTo` via ref:
  `<CameraControls makeDefault />`.
- **`Float`** — idle drift: `<Float speed={1} rotationIntensity={1} floatIntensity={1}>`.
- **`Stage`** — one-liner studio (auto-centers, frames camera, lights, grounds):
  `<Stage adjustCamera={1.5} shadows="contact" environment="city" preset="rembrandt">`.

## 3. Tone mapping + color management (three r185)

Constants on `THREE`: `NoToneMapping, LinearToneMapping, ReinhardToneMapping,
CineonToneMapping, ACESFilmicToneMapping, AgXToneMapping, NeutralToneMapping`.

**Two "default" answers — state both, don't conflate:**

- three-core `WebGLRenderer.toneMapping` default = **`NoToneMapping`**.
- R3F `<Canvas>` overrides it to **`ACESFilmicToneMapping`** (and
  `outputColorSpace = SRGBColorSpace`) to match Blender/Photoshop. `<Canvas flat>`
  switches back to `NoToneMapping`.

```jsx
// AgX / Neutral give more accurate highlight rolloff than ACES (which crushes saturated colors).
<Canvas gl={{ toneMapping: THREE.AgXToneMapping, toneMappingExposure: 1.1 }}>…</Canvas>
```

- `THREE.ColorManagement.enabled` is **`true`** by default (since r152) — leave it
  on for PBR. `outputColorSpace` defaults to `SRGBColorSpace`.
- Set `colorSpace={THREE.SRGBColorSpace}` on **color/albedo** textures; leave data
  maps (normal/roughness/metalness) linear (`NoColorSpace`).
- For heaviest control, tone-map in postprocessing (`<ToneMapping>`) and set the
  renderer to `flat`/`NoToneMapping` so you don't tonemap twice.

## 4. WebGPU + TSL (caveat)

`WebGPURenderer` (from `three/webgpu`) is production-usable and auto-falls back to
WebGL2 (`forceWebGL: true` to force). R3F v9's **async `gl` prop** enables it
(WebGPU needs `await renderer.init()`). **TSL** (`three/tsl`) is a
renderer-agnostic node shader language that replaces hand-written GLSL.

⚠️ **`@react-three/postprocessing` is WebGL-only** — it does not run on the WebGPU
backend. For post FX under WebGPU use three's own TSL `PostProcessing`
(`three/webgpu` + `three/tsl` passes), not `<EffectComposer>`. Choose your effect
stack around the renderer you commit to.

## 5. R3F v9 / React 19 notes

- R3F v9 targets **React 19** (v8 was React 18); drei v10 pairs with it.
- The global `JSX.IntrinsicElements` namespace for three elements is deprecated —
  extend via the **`ThreeElements`** interface from `@react-three/fiber`.
- `physicallyCorrectLights` / `useLegacyLights` are **removed** — lighting is
  physically correct by default.

## Deprecated / renamed (flag list)

| Old | New | Since |
|---|---|---|
| `outputEncoding` | `outputColorSpace` (`SRGBColorSpace`) | three r152 |
| `Texture.encoding` | `Texture.colorSpace` | three r152 |
| `sRGBEncoding`/`LinearEncoding` | `SRGBColorSpace`/`LinearSRGBColorSpace` | three r152 |
| `<ToneMapping adaptive middleGrey/>` | `mode={ToneMappingMode.…}` | postprocessing 6.3x |
| legacy `SSAO` | `N8AO` | current |
| `physicallyCorrectLights` | (removed; default) | three r155/165 |
| pmndrs `postprocessing` on WebGPU | three/webgpu `PostProcessing` + TSL | — |

## Reduced motion & quality ladder

Cinematic effects are the heaviest and most motion-sick-prone layer. Always:

- Gate expensive effects (N8AO, DoF, high `samples`) behind a device/DPR quality
  tier; drop to a lighter stack on low-power devices.
- Provide a reduced-motion branch: cut camera travel, idle `Float`, and heavy
  parallax; keep the lit, graded still. Read the preference and branch the ladder.
- Keep text/UI legible through bloom and DoF. See `performance-accessibility.md`.
