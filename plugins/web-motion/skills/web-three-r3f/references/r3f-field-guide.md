# R3F Field Guide

Use this compact guide before loading the larger bundled source captures.

## Current Snapshot

- Source refresh: 2026-06-04.
- Confirm installed versions first. R3F is tied to React's major renderer line:
  `@react-three/fiber@8` with React 18, `@react-three/fiber@9` with React 19.
- The source checks behind this guide use `@react-three/fiber` 9.6.1,
  `@react-three/drei` 10.7.7, and `three` 0.184.0 as the current package
  baseline.
- Package registry checks on 2026-06-04 confirmed those as latest npm releases.
  Repository HEAD pins are tracked separately in `references/source-ledger.md`
  because HEAD can differ from the published package gitHead.

## Canvas

- `<Canvas>` owns renderer creation, default camera/scene/raycaster, resize
  measurement, events, and most declarative cleanup.
- Key props to decide intentionally:
  - `dpr`: default is automatic `[1, 2]`; clamp heavy scenes lower.
  - `frameloop`: `always`, `demand`, or `never`; use `demand` for idle scenes.
  - `fallback`: DOM fallback for unsupported WebGL.
  - `camera`, `shadows`, `gl`, `events`, `eventSource`, `eventPrefix`, and
    `onCreated`: use only when local requirements demand them.
- `frameloop="demand"` renders on detected prop changes. If imperative code
  mutates objects, call `invalidate()` to request a frame. Drei controls already
  handle this for common camera controls.
- For custom `createRoot(canvas)`, resizing is caller-owned. Do not assume the
  behavior of `<Canvas>`.
- Frame the surface before debugging render code: parent dimensions, aspect
  ratio or viewport ownership, camera near/far/fov/position, object scale, and
  canvas z-index/visibility all need concrete values. Zero-height parents and
  off-camera models are common false render failures.
- Product-critical canvases need a DOM or image fallback for unsupported WebGL
  and an error-boundary path for context creation failure or asset errors.

## Assets

- Prefer `useLoader`, Drei `useGLTF`, `useTexture`, and Suspense over ad hoc
  loader instances in component render.
- R3F caches `useLoader` results for the same URL. Drei `useGLTF` exposes
  `preload`, `clear`, decoder path, Draco, MeshOpt, and `extendLoader` hooks.
- Use `gltfjsx` or an equivalent repo-approved pipeline when static GLTF
  assets should become reusable, typed components.
- Decoder/CDN paths must match the app's CSP, offline, and deployment posture.
- Keep asset loading behind one boundary per route/scene. Do not mix raw
  loader instances, Drei hooks, generated GLTFJSX components, and manual
  cache-clearing without a named owner for preloading, errors, and disposal.
- Clear Drei/R3F loader caches only for genuinely obsolete URLs; clearing a
  shared cached asset can break another mounted scene that still references it.

## Ownership And Cleanup

- R3F disposes many declarative objects on unmount. It also disconnects events,
  clears render lists, can force context loss, and disposes the scene during
  root unmount.
- R3F deliberately does not dispose `<primitive object={...}>` because the
  primitive may be owned outside React. Assign an owner and cleanup path.
- Plain Three.js code must stop loops and release resources itself:
  `cancelAnimationFrame`, `renderer.setAnimationLoop(null)`, `renderer.dispose`,
  control/pass `dispose`, listener removal, and explicit disposal for obsolete
  geometries, materials, textures, render targets, and skeletons.
- Disposing a material does not dispose its textures. ImageBitmap CPU-side
  resources also need application-level `close()` when applicable.
- Use `renderer.info.memory` and route mount/unmount cycles to detect leaks.
- Plain renderer owners should also own resize handlers, WebGL context-loss
  handlers, postprocessing composer passes, animation mixers/actions, and any
  worker or decoder setup associated with the scene.

## Performance

- Avoid React state in `useFrame`, intervals, and high-frequency pointer events.
  Mutate refs and use `delta` for refresh-rate-independent motion.
- Avoid allocating temporary `Vector3`, `Color`, `Matrix4`, geometry, material,
  or texture objects inside render or per-frame loops.
- Reduce GPU cost before adding effects: clamp DPR, cap shadows, reduce
  postprocessing, compress/resize textures, use LOD/nested loading, instance or
  merge repeated meshes, and adapt quality with Drei `PerformanceMonitor`.
- Treat every mesh as a draw call unless instanced/merged. Thousands of meshes
  need a deliberate batching strategy.

## SSR And Accessibility

- In Next.js and other SSR frameworks, isolate WebGL imports, R3F hooks, loader
  calls, and browser APIs inside client components.
- Keep the client boundary small. Do not convert a whole server page to a client
  component just to host a canvas.
- Respect `prefers-reduced-motion`: disable nonessential auto-rotation,
  camera drift, parallax, particles, and long travel. Static lighting/model
  inspection can remain.
- Product-critical information cannot exist only in a canvas. Provide DOM text,
  alt/fallback media, or equivalent controls.

## Verification

- Compile/typecheck the changed app.
- Browser-check nonblank canvas pixels, stable framing, resize behavior,
  interaction, reduced motion, unsupported WebGL fallback, asset error fallback,
  route unmount/remount, and mobile DPR/performance.
- For Three/R3F bugs, inspect source for the installed version before relying on
  memory. Use `opensrc path @react-three/fiber @react-three/drei three`.
