# lottie-web loadAnimation And Renderer Notes

Sources:

- https://github.com/airbnb/lottie-web/wiki/loadAnimation-options
- https://github.com/airbnb/lottie-web/wiki/Renderer-Settings
- https://raw.githubusercontent.com/wiki/airbnb/lottie-web/loadAnimation-options.md
- https://raw.githubusercontent.com/wiki/airbnb/lottie-web/Renderer-Settings.md

Checked at: 2026-06-04. These notes paraphrase official wiki pages and are
optimized for implementation decisions.

## `loadAnimation` Options

- `container`: required DOM element. Do not call `loadAnimation` before the
  element exists.
- `renderer`: choose `svg`, `canvas`, or `html`. Prefer SVG for crisp vector UI
  and renderer metadata; choose canvas/html only for asset-specific reasons that
  have been tested.
- `loop`: default behavior loops; set explicit `false`, `true`, or a numeric
  loop count.
- `autoplay`: default behavior starts playback when ready; set explicit `false`
  for state-controlled, reduced-motion-aware, or user-triggered playback.
- `path` or `animationData`: provide one asset source. Use local versioned
  paths or imported JSON in production. Remote paths need operational review.
- `assetsPath`: override where exported image assets are resolved when JSON
  references a generated images folder.
- `name`: only needed when global lottie methods must target an instance by
  name; direct instance references are clearer for wrappers.
- `initialSegment`: start from a specific frame range.
- `progressiveLoad`: can defer SVG DOM element creation for long animations but
  must be tested with masks and mattes.

## Renderer Settings

- `preserveAspectRatio`: applies SVG-like sizing behavior for SVG/canvas
  renderers. Align this with the wrapper's CSS aspect-ratio box.
- `title` and `description`: SVG renderer metadata for meaningful animation.
  They do not replace visible/status text when the animation communicates state.
- `progressiveLoad`: SVG-only progressive element creation.
- `context`: provide a canvas 2D context when integrating with an existing
  canvas owner.
- `clearCanvas`: when false, the app must own canvas clearing behavior.
- `className` and `id`: add styling/query hooks to the root animation element.
- `hideOnTransparent`: hides transparent elements; test if opacity-driven
  effects are part of the asset.
- `runExpressions`: set false only when disabling expressions is intentional and
  tested against the export.
- `focusable`: SVG focusability. Keep decorative animations out of keyboard
  navigation.
- `filterSize`: expand SVG filter bounds when shadows or filters are clipped.

## Review Defaults

- Reserve width, height, and aspect ratio before loading to avoid layout shift.
- Put reduced-motion branching before autoplay/loop decisions.
- Treat every marker, segment, `name`, and external asset path as a product
  contract. Keep those strings in one local mapping.
- Verify first load, replay, route unmount, repeated mount, reduced motion,
  missing asset, and slow network.
