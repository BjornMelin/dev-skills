# lottie-web Compact Source Notes

Sources:

- https://github.com/airbnb/lottie-web
- https://raw.githubusercontent.com/airbnb/lottie-web/v5.13.0/package.json
- https://raw.githubusercontent.com/airbnb/lottie-web/v5.13.0/index.d.ts

Checked at: 2026-06-04. These notes paraphrase official `lottie-web` README,
package, and type declaration details. Use installed package types/source first
when exact runtime behavior matters.

## Package Pin

- npm package: `lottie-web@5.13.0`.
- Source/tag pin: `airbnb/lottie-web@bede03d25d232826e0c9dca1733d542d8a7754fb`
  via `refs/tags/v5.13.0`.
- Published entrypoints: `main` is `./build/player/lottie.js`; `types` is
  `./index.d.ts`; license is MIT.

## Runtime Surface

- `lottie.loadAnimation(options)` creates an animation instance. The container
  DOM element and either `animationData` or `path` are the important ownership
  decisions.
- Common renderers are `svg`, `canvas`, and `html`; SVG is the usual default
  for web UI because it keeps vector DOM and renderer metadata available.
- `loop` accepts boolean or number. `autoplay` should be explicit in production
  wrappers.
- The returned animation item owns playback methods such as `play`, `pause`,
  `stop`, `setSpeed`, `setDirection`, `goToAndStop`, `goToAndPlay`,
  `playSegments`, `setSubframe`, `getDuration`, `resize`, and `destroy`.
- Relevant event names from the published types include `complete`,
  `loopComplete`, `drawnFrame`, `enterFrame`, `segmentStart`, `config_ready`,
  `data_ready`, `data_failed`, `loaded_images`, `DOMLoaded`, `error`, and
  `destroy`.

## Wrapper Implications

- Keep one animation instance per mounted player component unless a higher-level
  controller intentionally owns several instances.
- Call `destroy()` on unmount or route disposal. If custom listeners were added,
  remove them as part of wrapper cleanup.
- Avoid broad global methods such as unqualified `lottie.destroy()` unless the
  component intentionally owns every animation on the page.
- For repeated instances that share a parsed JSON object, clone asset data when
  the asset/runtime shows mutation issues, especially with repeaters.
- Prefer local versioned JSON assets. Remote `path` values need fallback UI,
  cache/CSP review, and `data_failed`/`error` handling.

## Asset And Performance Notes

- Bodymovin exports can include image assets next to the JSON. Inspect JSON for
  external images/fonts and make paths deterministic before production use.
- Heavy masks, large shape counts, and large JSON payloads can hurt real-time
  rendering performance. Use renderer selection, lazy loading, posters, or
  route-level splitting when needed.
- For meaningful SVG animations, use `rendererSettings.title` and
  `rendererSettings.description` or equivalent surrounding text. Decorative
  animations should not add extra assistive-technology noise.
