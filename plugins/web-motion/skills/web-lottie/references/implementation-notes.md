# web-lottie Implementation Notes

Use this reference when the task needs more than the quick guidance in
`SKILL.md`: player selection, wrapper implementation, asset review, security,
performance, or accessibility.

## Player Selection

| Need | Prefer | Notes |
| --- | --- | --- |
| Existing Bodymovin JSON and SVG output | `lottie-web` | Supports `svg`, `canvas`, and `html` renderers. SVG renderer supports `title` and `description` renderer settings. |
| `.lottie` package, themes, state machines, or packaged assets | `@lottiefiles/dotlottie-web` | Core package renders Lottie JSON and dotLottie through canvas/WASM. |
| Heavy dotLottie render workload | `DotLottieWorker` | Uses worker rendering when browser support and bundler behavior are acceptable. |
| Hardware acceleration experiments | dotLottie `/webgl` or `/webgpu` entrypoints | Treat as an explicit product/perf decision and test feature support plus fallback. |
| Ordinary UI transitions | CSS, Motion, GSAP, or WAAPI | Do not use Lottie for hover, accordion, route, or list choreography. |

## Version And Package Truth

- Current package pins from the 2026-06-04 audit are recorded in
  `references/provenance.json`. At that check, npm `latest` was
  `lottie-web@5.13.0` and `@lottiefiles/dotlottie-web@0.74.0`.
- `lottie-web@5.13.0` publishes `index.d.ts` and `build/player/lottie.js`.
  Prefer installed package types over README prose for exact event names and
  method signatures.
- `@lottiefiles/dotlottie-web@0.74.0` publishes ESM/CJS exports for `.`,
  `./webgl`, `./webgpu`, and WASM files. Treat `/webgl` and `/webgpu` as
  explicit compatibility decisions, not drop-in defaults.
- The dotLottie npm package did not expose a `gitHead` in the npm metadata at
  the audit time. Use the npm version and tarball integrity as package truth;
  use the GitHub repo HEAD only for source orientation unless a matching tag is
  verified.
- If installed versions differ from bundled pins, implement against installed
  types/source and mention the drift in the closeout.

## Lifecycle Wrapper Checklist

- Instantiate only after the DOM element or canvas exists.
- Keep SSR/server components free of player instantiation. Dynamic import or
  isolate player code in a client-only component when needed.
- Reserve dimensions before the animation loads to avoid layout shift.
- Set `autoplay` and `loop` explicitly.
- Add load/error event handling when the asset is remote, large, or
  user-visible.
- Destroy the instance on unmount. Remove custom event listeners if the wrapper
  attached any outside the player cleanup path.
- Avoid global controls such as `lottie.destroy()` without a `name` unless the
  surface intentionally owns every animation on the page.
- Store all asset-contract strings in one local mapping when using markers,
  segments, animation IDs, theme IDs, or state-machine IDs.

## `lottie-web` Notes

- `loadAnimation` needs a `container` and either `path` or `animationData`.
- `renderer` can be `svg`, `canvas`, or `html`; default is `svg`.
- `loop` accepts `boolean | number`; `autoplay` defaults to immediate playback
  unless set false.
- Instance controls include `play`, `pause`, `stop`, `setSpeed`,
  `goToAndStop`, `goToAndPlay`, `playSegments`, `setSubframe`, `getDuration`,
  and `destroy`.
- Events include `complete`, `loopComplete`, `enterFrame`, `segmentStart`,
  `data_ready`, `data_failed`, `loaded_images`, `DOMLoaded`, `error`, and
  `destroy`.
- SVG renderer settings include `title`, `description`,
  `preserveAspectRatio`, `progressiveLoad`, `hideOnTransparent`, `className`,
  `id`, `focusable`, and filter sizing.
- Canvas renderer settings include `context`, `clearCanvas`,
  `preserveAspectRatio`, `progressiveLoad`, and `dpr`.
- Deep clone reused `animationData` when repeaters or runtime mutation cause
  repeated-instance bugs.

## dotLottie Notes

- `DotLottie` takes a config object with `canvas`, `src` or `data`, `autoplay`,
  `loop`, `loopCount`, `marker`, `segment`, `speed`, `mode`, `layout`,
  `themeId`, `stateMachineId`, and `renderConfig`.
- `renderConfig` includes `autoResize`, `devicePixelRatio`,
  `freezeOnOffscreen`, and `quality`. Keep `freezeOnOffscreen` enabled unless
  the product requires continuous offscreen rendering.
- `layout.fit` behaves like object-fit for the canvas: `contain`, `cover`,
  `fill`, `none`, `fit-width`, or `fit-height`; `layout.align` is `[x, y]`
  from `0` to `1`.
- Default package exports the core player and WASM. `/webgl` and `/webgpu`
  entrypoints use hardware-specific WASM subpaths.
- `DotLottieWorker` methods are async because control crosses a worker
  boundary. Await destructive or sequencing-sensitive calls.
- Useful events include `ready`, `load`, `loadError`, `renderError`, `play`,
  `pause`, `stop`, `complete`, `loop`, `frame`, `render`, `freeze`,
  `unfreeze`, `destroy`, and state-machine events. Use `loadError` and
  `renderError` for user-visible fallbacks.
- State machines can open URLs. Require `stateMachineConfig.openUrlPolicy` with
  `requireUserInteraction: true` and a narrow `whitelist`, or block URL opens
  with an empty whitelist.

## Asset Review

Inspect the asset before changing runtime code:

- Size: large JSON or `.lottie` files need lazy loading, a poster, or route-level
  code splitting.
- External dependencies: JSON exports can reference images or fonts. Prefer
  local versioned assets and predictable public paths.
- Renderer support: test masks, mattes, effects, text, images, and expressions
  in the chosen renderer and browsers.
- Loop intent: decorative infinite loops need reduced-motion behavior and should
  not consume CPU offscreen.
- Multi-animation assets: document animation IDs, marker names, theme IDs, and
  state-machine IDs as contracts.
- Failure states: missing asset, failed fetch, unsupported browser feature, and
  slow network should leave usable UI.

## Accessibility And Reduced Motion

- If the animation is meaningful, provide equivalent text in visible copy,
  `aria-label`, surrounding status text, or SVG renderer metadata.
- If the animation is decorative, hide it from assistive technology and keep
  surrounding UI semantics intact.
- For `lottie-web` SVG, use `rendererSettings.title` and `description` when the
  animation itself needs an accessible name/description.
- For dotLottie canvas, put semantic text outside the canvas; canvas pixels are
  not a reliable accessibility tree.
- Under `prefers-reduced-motion`, stop autoplay/loops, render a poster frame, or
  switch to an instant/subtle state change. Do not rely on offscreen freezing as
  a reduced-motion substitute.

## Security And Operations

- Do not treat third-party animation URLs as harmless. They affect availability,
  privacy, CSP, cache behavior, and reviewability.
- Prefer committed assets or controlled CDN paths with cache/version strategy.
- Reject or sanitize untrusted animation uploads before playback; animation
  files may contain external resource references and interactive URL behavior.
- For dotLottie state machines, default URL-opening policy should be blocked or
  allowlisted plus user-initiated.
- Confirm the app's CSP permits the required `fetch`, image/font, worker, WASM,
  canvas, WebGL, or WebGPU behavior before shipping.

## Focused Validation

Use the skill audit script first, then app-local checks:

```bash
node scripts/audit.mjs doctor --root . --format json
node scripts/audit.mjs scan --root . --format markdown
```

Manual validation should cover reduced motion, route unmount, repeated mount,
network failure, resize, slow devices, and every marker/state-machine path used
by product code.
