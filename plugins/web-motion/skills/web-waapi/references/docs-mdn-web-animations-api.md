# Web Animations API Notes

Use this as a compact, source-backed reference when `SKILL.md` is not enough.
Check `references/source-ledger.md` for upstream URLs and freshness notes.

## Core Model

- The Web Animations API describes DOM animation through a timing model and an
  animation model. It gives script access to the browser animation engine that
  also underlies CSS animations and transitions.
- `Element.animate(keyframes, options)` is the ergonomic entry point. It creates
  an `Animation`, applies it to the element, starts playback, and returns the
  `Animation` instance.
- An element can have multiple animations. Use `element.getAnimations()` or
  `document.getAnimations()` when overlapping effects, cleanup, or global
  playback control matters.
- `KeyframeEffect` stores animatable properties, keyframes, target/pseudo
  target, timing, and composition behavior. Create one directly when an effect
  must be reused, cloned, retargeted, or mutated.
- `Document.timeline` is the default timeline. Newer scroll/view timelines can
  be passed through `Element.animate()` options, but they are support-policy
  features rather than default WAAPI assumptions.

## Timing And Playback

- `Animation.currentTime` gets or sets the animation time in milliseconds. It
  can be `null` before the animation is active, after cancellation, or when no
  timeline is attached.
- `play()`, `pause()`, `reverse()`, `finish()`, `cancel()`, and
  `updatePlaybackRate()` are the normal imperative controls.
- `finish()` seeks to the end that matches playback direction.
- `finish()` is not a generic cleanup method. It can fail for a zero playback
  rate or an infinite effect end; reserve it for deliberate end-state jumps.
- `cancel()` clears the animation's effects, aborts playback, and resets
  `startTime` and `currentTime` to `null`.
- `ready` is a promise for the ready state after pending play/pause work.
  Await it before deterministic seek/sync code when the browser has not yet
  committed playback.
- `animation.finished` is a promise for the current run. A new promise is
  created whenever the animation leaves the finished state and starts again.
- Cancelling a non-idle animation rejects the current `finished` promise with an
  `AbortError`; catch that expected interruption in async flows.
- Setting `playbackRate` is synchronous and can cause a running animation to
  jump. Use `updatePlaybackRate()` when rate changes should be coordinated with
  playback.

## Persistence And Cleanup

- Avoid using `fill: 'forwards'` or `'both'` as the long-term persistence
  mechanism. Filled animations can retain animation state and have higher
  cascade precedence than normal specified styles.
- Prefer CSS/state-owned final styles when possible. If script must persist the
  current animated values, wait for the intended point, call `commitStyles()`,
  then `cancel()` to remove the animation effect.
- `commitStyles()` writes the current animation styling to the target element's
  inline `style` attribute. This is useful for deliberate handoff, but it can
  pollute inline styles if used casually.
- `commitStyles()` stores computed values at commit time. It does not preserve
  the live relationship to CSS variables, inherited font-size changes, or other
  context-sensitive computed values.
- `commitStyles()` can throw if the target cannot receive inline styles or is
  not rendered. Treat disconnected, `display: none`, and pseudo-element flows as
  error-aware handoff code.
- Browsers can automatically remove replaced filling animations. Use
  `persist()` only when that automatic removal would be wrong.
- `persist()` keeps the animation around after replacement and can consume
  resources. It is not a substitute for cleanup.
- Cleanup must run on unmount/navigation and on repeated triggers. Re-triggered
  animations should cancel or finish the old handle before starting a new one
  unless overlap is intentional.

## Keyframes And Effects

- Keyframes can be an array of keyframe objects or a property-indexed object.
  Prefer explicit from/to values for every animated property.
- `effect.getTiming()`, `effect.getComputedTiming()`, `effect.updateTiming()`,
  `effect.getKeyframes()`, and `effect.setKeyframes()` help inspect or mutate
  effects without reconstructing every animation object.
- `KeyframeEffect.composite` controls how an effect combines with underlying
  values: `replace`, `add`, or `accumulate`.
- `iterationComposite` controls how values combine across iterations, but MDN
  marks it as limited availability. Treat it as support-policy gated.
- Additive/accumulating composition is useful for advanced transforms and
  filters, but it is harder to reason about than replacement. Prefer
  replacement unless additive composition is the actual requirement.

## Newer Options And Support Cautions

- Core `Element.animate()`, `Animation`, `KeyframeEffect`, `currentTime`,
  `cancel()`, and `prefers-reduced-motion` are MDN Baseline widely available.
- MDN marks `Element.animate()` and `KeyframeEffect` as broadly available since
  March 2020, while also warning that some parts can vary by browser.
- `KeyframeEffect.composite` is broadly available in modern browsers, but still
  review target support when additive behavior is product-critical.
- `KeyframeEffect.iterationComposite` is not Baseline because it does not work
  in some widely used browsers.
- `Element.animate()` supports newer `timeline`, `rangeStart`, and `rangeEnd`
  options that map to CSS scroll/view timeline concepts. Guard or verify these
  against the target browser policy before shipping.
- Scroll/view timeline code should include a feature guard, a CSS `@supports`
  fallback, or an explicit browser-support note in the implementation review.

## Accessibility And Performance

- Use `matchMedia('(prefers-reduced-motion: reduce)')`, framework hooks, or the
  repo's motion abstraction to reduce, replace, or skip nonessential motion.
- Reduced motion should change the effect semantics, not just shorten a large
  spatial movement to an abrupt motion.
- Prefer `transform` and `opacity` for hot paths. Layout properties, filters,
  shadows, and large paint areas need measurement and device/browser proof.
- Avoid read/write loops: collect measurements first, then run `animate()` in a
  write phase such as the next animation frame or framework effect.

## Review Heuristics

- Flag `element.animate(...)` calls whose returned `Animation` is ignored.
- Flag `fill: 'forwards'` or `'both'` without an explicit persistence decision:
  CSS final state, `commitStyles()` plus `cancel()`, or accepted temporary fill.
- Flag `await animation.finished` without `AbortError` handling when code can
  cancel or interrupt the animation.
- Flag `commitStyles()` without evidence that inline-style persistence is
  intended and the target can accept committed styles.
- Flag `persist()` unless replacement-state fidelity is the explicit reason.
- Flag `timeline`, `rangeStart`, `rangeEnd`, `ScrollTimeline`,
  `ViewTimeline`, `composite: 'add'`, `composite: 'accumulate'`, or
  `iterationComposite` without support evidence.
- Flag layout-affecting keyframes in scroll, list, drag, resize, or repeated
  hover paths unless measurements prove the cost is acceptable.
- Verify null refs and client-only execution in SSR/hydration frameworks.
