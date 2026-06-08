# Web Rive Implementation Notes

Load this file when choosing a Rive web runtime, integrating a `.riv` asset, or
reviewing an existing Rive implementation.

## Runtime Selection

- `@rive-app/react-webgl2` / `@rive-app/webgl2`: default for new Rive work. Use
  it when visual quality, Rive Renderer features, Data Binding, text, vector
  feathering, or heavier interactive scenes matter. Keep
  `useOffscreenRenderer: true` unless measured evidence says otherwise; it is
  especially important for multiple WebGL2 instances because browsers limit
  concurrent WebGL contexts.
- `@rive-app/react-canvas` / `@rive-app/canvas`: smaller renderer for simpler
  assets where Canvas 2D is enough and Rive Renderer-only features are not
  needed.
- `@rive-app/react-canvas-lite` / `@rive-app/canvas-lite`: smallest option, but
  not for assets using text, layout, audio, scripting, or other advanced
  features.
- `@rive-app/webgl`, `@rive-app/react-webgl`, and `rive-react` are legacy
  package paths. Do not add new usage; migrate to the specific WebGL2 or Canvas
  package that matches the renderer boundary.
- Use the target repo's installed versions as code truth. This skill's current
  provenance pins are React `@rive-app/*@4.28.6` and Web
  `@rive-app/*@2.37.8`.

## React And SSR Boundaries

- Rive draws into a browser canvas. In SSR frameworks, make the Rive component a
  client-only leaf and keep useful surrounding content on the server.
- Use `useRive` when app code needs the raw `rive` instance for playback,
  events, Data Binding, or state-machine controls. Use the default `<Rive />`
  only for simple playback.
- The Rive React runtime creates the instance after the canvas is mounted. A
  container that starts at `0x0` can render blank or blurry, so reserve stable
  dimensions with CSS before mounting.
- Isolate `useRive` in a wrapper component when conditionally rendering. This
  lets the hook clean up and recreate the rendering context around mount cycles.
- Prefer `useEffect` keyed on `rive` for operations that need the ready
  instance instead of assuming a consumer `onLoad` callback has current React
  state.
- In Next.js, Remix, Astro islands, or other SSR frameworks, keep Rive imports
  behind a browser/client boundary. Server components should own semantic
  content, not the canvas runtime.

## Asset Contracts

- Treat `.riv` files like API contracts. Record artboard, state machine, view
  model, property, event, input, text, font, audio, and out-of-band asset names.
- Keep names in one typed map or wrapper. Avoid scattered string literals.
- Explicitly pass `stateMachines` or `animations` for product behavior. If a
  wrapper relies on the default artboard/state machine exported from the asset,
  document that default next to the contract map.
- Prefer local/public assets or a controlled CDN. Remote examples are demos; a
  production remote asset needs an allowlisted origin, cache/version strategy,
  fallback UI, and failure test.
- For shared assets used by many instances, consider runtime-supported file
  caching such as `useRiveFile` in React after checking the installed package.
- If the asset uses referenced fonts, images, audio, or other out-of-band
  assets, verify the runtime asset loader path and CORS/cache behavior rather
  than only checking that the `.riv` URL loads.

## Data Binding Versus Inputs

- Prefer Data Binding View Models for new interactive assets. View model
  properties cover numbers, booleans, triggers, strings, enums, colors, nested
  view models, lists, images, and artboards.
- Legacy state-machine inputs still work and `useStateMachineInput` is the
  React hook for existing input-driven files. Use it when the exported `.riv`
  already exposes Inputs or when the work is a prototype.
- Do not migrate Inputs to Data Binding in code only. The `.riv` asset and the
  runtime wrapper must change together.
- `useStateMachineInput` may return `null` until the asset loads. Guard access,
  and use `.value` for number/boolean inputs and `.fire()` for triggers.
- For Data Binding, decide whether the runtime should use auto-binding, a named
  instance, the default instance, or a freshly created instance. That choice is
  part of the asset contract and should be testable.

## Events, Accessibility, And Security

- Rive canvas content is not a substitute for semantic HTML. Put meaningful
  labels, status text, and controls outside the canvas unless a specific runtime
  feature covers the requirement.
- Decorative loops should be hidden from assistive technology and disabled,
  paused, or replaced under reduced motion.
- For meaningful animation, provide the accessible state in nearby text or
  controls and verify keyboard/screen-reader behavior.
- Rive events can drive product actions. Avoid automatic URL/event handling
  unless the behavior is product-required, user-initiated, and origin-checked.

## Performance And Cleanup

- React hooks clean up their owned Rive instance, but app-added event listeners,
  timers, observers, and property subscriptions still need cleanup in effects.
- Plain Web runtime integrations must call `cleanup()` when the instance is no
  longer needed. Use `deleteRiveRenderer()` only after `cleanup()` for full
  WebGL renderer teardown when the canvas is removed entirely.
- Use `stopRendering()` / `startRendering()` or runtime visibility controls for
  hidden canvases when appropriate.
- If code disables `useOffscreenRenderer`, require a measured reason and a
  browser/device test that covers many-instance and navigation cleanup.
- Test many instances, mobile GPUs, high-DPI sharpness, route unmount, and
  resized containers when the Rive surface is important.
