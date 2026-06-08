# R3F Canvas Source Notes

Source: https://r3f.docs.pmnd.rs/api/canvas

Checked: 2026-06-04.

Use this file when a task depends on Canvas defaults, fallback behavior, custom
renderer creation, or root ownership.

## API Boundaries

- `<Canvas>` is the normal React portal into Three.js. It creates renderer,
  scene, camera, raycaster, events, and resize measurement for the React tree.
- Important props to decide intentionally: `fallback`, `gl`, `camera`, `scene`,
  `shadows`, `raycaster`, `frameloop`, `resize`, `orthographic`, `dpr`,
  `events`, `eventSource`, `eventPrefix`, and `onCreated`.
- Default DPR is automatic `[1, 2]`; heavy scenes often need a lower clamp or
  adaptive policy.
- `frameloop` supports `always`, `demand`, and `never`. Use `demand` for idle
  scenes, and call `invalidate()` when imperative code mutates visible state.
- `<Canvas fallback={...}>` handles unsupported WebGL. Use an error boundary
  for context creation failures, GPU-disabled browsers, and asset failures.

## Renderer Lifecycle

- `<Canvas>` owns the default WebGL renderer lifecycle. Avoid duplicating that
  lifecycle in nested effects unless the app has a deliberate custom renderer.
- Custom `gl` callbacks may create WebGL or WebGPU renderers. If the callback
  returns app-created objects, confirm the installed R3F version's cleanup
  behavior and define ownership for resources not managed by R3F.
- Custom `createRoot(canvas)` is lower-level than `<Canvas>`: the caller must
  configure, size, render, and unmount the root. Do not assume automatic resize.

## Framing Checks

- Confirm the canvas parent has a stable nonzero size before debugging scene
  code.
- Confirm camera fov/near/far/position, object scale, and model origin place
  the subject inside the frustum.
- Screenshot and canvas-pixel checks should prove both nonblank rendering and
  useful framing, not just that WebGL initialized.
