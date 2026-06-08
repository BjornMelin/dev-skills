# dotLottie-web Compact Source Notes

Sources:

- https://developers.lottiefiles.com/docs/dotlottie-player/dotlottie-web/
- https://developers.lottiefiles.com/docs/dotlottie-player/dotlottie-web/api/config/
- https://developers.lottiefiles.com/docs/dotlottie-player/dotlottie-web/api/render-config/
- https://developers.lottiefiles.com/docs/dotlottie-player/dotlottie-web/events/
- https://github.com/LottieFiles/dotlottie-web
- https://registry.npmjs.org/@lottiefiles%2fdotlottie-web/latest

Checked at: 2026-06-04. These notes paraphrase official LottieFiles docs,
source, and npm package metadata. Use installed package types/source first when
exact runtime behavior matters.

## Package Pin

- npm package: `@lottiefiles/dotlottie-web@0.74.0`.
- npm package metadata at check time did not expose `gitHead`; no matching
  `v0.74.0` or `0.74.0` tag was returned by the source repo probe.
- Use the npm package version and tarball integrity as package truth. Use
  `LottieFiles/dotlottie-web@6795b9017ef09620c51c1a86a2900e84aad7ad61` only
  as source orientation unless a matching package source pin is later verified.
- Published exports include `.`, `./webgl`, `./webgpu`, and WASM subpaths.
  Advanced entrypoints need bundler and browser proof.

## Runtime Surface

- `DotLottie` is the core web player. It renders through canvas/WASM and can
  load `.lottie` bundles or JSON.
- The player requires WebAssembly, Canvas 2D, and Fetch API for normal browser
  use. Workers and OffscreenCanvas are optional performance features.
- Constructor config centers on `canvas`, `src` or `data`, `autoplay`, `loop`,
  `loopCount`, `speed`, `mode`, `segment`, `marker`, `animationId`, `themeId`,
  `stateMachineId`, `stateMachineConfig`, `layout`, and `renderConfig`.
- `layout.fit` behaves like object fit with choices such as `contain`, `cover`,
  `fill`, `none`, `fit-width`, and `fit-height`. `layout.align` uses `[x, y]`
  values from `0` to `1`.

## Render Config

- `autoResize`: automatically resizes rendering with the canvas/container.
- `devicePixelRatio`: controls render resolution. Higher values can improve
  sharpness but cost memory and render work.
- `freezeOnOffscreen`: pauses rendering when offscreen to conserve resources.
  Keep enabled unless product behavior requires continuous offscreen rendering.
- `quality`: source types expose a numeric quality knob; test visual output
  before using lower values.

## Events And Fallbacks

- Important lifecycle and playback events include `ready`, `load`, `loadError`,
  `renderError`, `play`, `pause`, `stop`, `complete`, `loop`, `frame`,
  `render`, `freeze`, `unfreeze`, and `destroy`.
- State-machine events include start/stop, transitions, state enter/exit,
  custom/error events, and input changes.
- User-visible or remote assets should handle `loadError` and `renderError`
  with a static fallback or usable empty state.
- If the wrapper attaches event listeners, remove them on disposal unless
  `destroy()` is proven to own every listener in the installed version.

## State Machines And URL Policy

- `stateMachineConfig.openUrlPolicy` controls URL-opening behavior.
- Require `requireUserInteraction: true` and a narrow `whitelist` for any
  allowed URL action. Use an empty whitelist or avoid URL actions when product
  requirements do not explicitly need them.
- Treat state-machine IDs, animation IDs, theme IDs, markers, and segment names
  as product contracts. Keep them centralized and test every path used by code.

## Compatibility Checks

- Test worker rendering where OffscreenCanvas support and bundler behavior are
  uncertain.
- Gate `/webgl` and `/webgpu` entrypoints behind feature support and fallback
  behavior. WebGPU in particular should not be used without browser-policy proof.
- Confirm CSP permits required fetch, WASM, worker, canvas, WebGL, or WebGPU
  behavior before shipping.
