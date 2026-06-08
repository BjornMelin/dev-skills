# Drei useGLTF Source Notes

Source: https://drei.docs.pmnd.rs/loaders/gltf-use-gltf

Checked: 2026-06-04.

Use this file when the task involves GLTF loading, decoder paths, preloading,
cache clearing, generated model components, or asset-pipeline boundaries.

## Loader Boundary

- Prefer `useGLTF` for GLTF/GLB assets in R3F apps that already use Drei.
  Prefer `useLoader(GLTFLoader, url)` when the repo avoids Drei or needs a
  lower-level loader boundary.
- Keep decoder and extension configuration at the loader hook boundary:
  Draco path/config, MeshOpt decoder, and `extendLoader` customizations belong
  with the model-loading owner, not inside render-time object construction.
- Suspense should own the loading UI for route/scene assets. Use Drei
  `useProgress` or app-level loading state only when the product needs progress
  details.

## Cache And Preload

- `useGLTF.preload(url)` is useful when the route is about to need the model.
  Do not preload every model globally without route-level value.
- `useGLTF.clear(url)` should be scoped to obsolete URLs owned by the current
  route/scene. Avoid clearing shared cached assets while another scene may
  still reference them.
- Generated GLTFJSX components commonly include `dispose={null}` to prevent
  recursive disposal of shared generated assets. Treat that as an ownership
  decision: either the generated component owns stable shared resources, or a
  parent route owns cleanup when the assets become obsolete.

## Deployment Fit

- Decoder/CDN paths must follow the app's CSP, offline, asset-prefix, and
  deployment rules.
- Asset URLs should be stable and public/importable according to the target
  bundler. If a GLB request returns HTML or a redirect, fix the asset pipeline
  before changing scene code.
