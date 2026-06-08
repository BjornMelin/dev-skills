# Three.js Disposal Source Notes

Source: https://threejs.org/manual/en/how-to-dispose-of-objects.html

Checked: 2026-06-04.

Use this file when a task creates or replaces Three.js resources outside R3F's
declarative ownership.

## Disposal Rules

- Three.js cannot infer application lifetime for objects created by user code.
  The app must call disposal APIs when resources become obsolete.
- Dispose obsolete `BufferGeometry`, `Material`, `Texture`, `WebGLRenderTarget`,
  `Skeleton`, controls, postprocessing passes, and renderers according to their
  APIs.
- Removing a mesh from a scene does not dispose its geometry or material.
- `Material.dispose()` does not dispose textures; textures may be shared.
- `Texture.dispose()` does not close an `ImageBitmap`; app code that creates
  ImageBitmap sources owns `ImageBitmap.close()` when appropriate.
- `renderer.info.memory` is useful for leak checks, but some internally cached
  Three.js resources may remain visible there without being leaks.

## Plain Renderer Lifecycle

- Stop the render loop before disposing: cancel RAF, stop controls or mixers,
  or call `renderer.setAnimationLoop(null)` for animation-loop renderers.
- Remove resize, pointer, visibility, context-loss, and custom DOM listeners
  owned by the scene.
- Dispose postprocessing composers/passes, render targets, controls, and
  renderer-owned resources when their route/component owner unmounts.
- Treat renderer disposal as terminal. Recreate controls/renderers after
  disposal instead of reusing disposed instances.
