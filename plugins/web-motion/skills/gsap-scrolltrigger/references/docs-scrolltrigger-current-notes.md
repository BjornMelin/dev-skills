# ScrollTrigger Current Notes

Use this file when a task needs detail beyond the default `SKILL.md` path.

## Decision Lanes

- Use native CSS scroll timelines only when the user requests CSS-only behavior,
  the repo already owns that pattern, or the animation is simple enough to stay
  declarative without GSAP.
- Use IntersectionObserver for lightweight visibility state when no scroll
  progress, pinning, scrub, or timeline choreography is needed.
- Use ScrollTrigger when animation progress, pinning, snapping, scroll velocity,
  refresh lifecycle, or GSAP timeline integration is central.
- Use `Observer` for wheel/touch/pointer gesture direction without tying
  animation progress to scroll position.

## Current API Surface To Check

- `ScrollTrigger.version` is exposed by the package. Inspect the target repo's
  installed `gsap` version before relying on new options.
- `ScrollTrigger.Vars` includes `anticipatePin`, `containerAnimation`,
  `endTrigger`, `fastScrollEnd`, `id`, `invalidateOnRefresh`, `once`, `pin`,
  `pinnedContainer`, `pinReparent`, `pinSpacing`, `preventOverlaps`,
  `refreshPriority`, `scrub`, `snap`, `start`, `end`, `toggleActions`, and
  `toggleClass`.
- Static helpers worth checking before custom code:
  `ScrollTrigger.batch()`, `clearScrollMemory()`, `config()`, `create()`,
  `defaults()`, `getAll()`, `getById()`, `killAll()`, `maxScroll()`,
  `normalizeScroll()`, `observe()`, `refresh()`, `saveStyles()`,
  `scrollerProxy()`, `snapDirectional()`, `sort()`, and `update()`.
- `ScrollTrigger.matchMedia()` still exists for compatibility, but
  `gsap.matchMedia()` is the modern default because it creates a context and
  automatically reverts animations and ScrollTriggers when conditions stop
  matching.

## Start And End

- String values describe a trigger position and scroller position:
  `"top center"`, `"bottom 80%"`, `"center 100px"`, or compound offsets such as
  `"top bottom-=100px"`.
- Single relative values such as `"+=300"` or `"+=100%"` are relative to the
  calculated start. `"max"` means maximum scroll position.
- Numeric values are absolute scroll positions.
- Function values run on refresh and receive the ScrollTrigger instance.
- `clamp(...)` is available for string values and keeps calculations within
  page scroll bounds, which helps above-the-fold and near-page-end triggers.
- If `pin: true`, the default start is effectively `"top top"`; otherwise the
  default start is `"top bottom"` and the default end is `"bottom top"`.
- Use `endTrigger` when a different element, not the trigger, should define the
  end position.

## Pinning

- Pinning changes layout through a spacer unless `pinSpacing: false` is used.
  Keep the default unless the surrounding layout already provides the space.
- Pin order matters. Create triggers top-to-bottom or use `refreshPriority`.
- `refreshPriority` sorts higher values earlier in GSAP source. Prefer avoiding
  custom priorities unless async trigger creation makes ordering unavoidable.
- `pinnedContainer` is for triggers inside another pinned container.
- `pinReparent` can fix transformed ancestor problems, but it moves the pinned
  element under `body` while active and may break CSS selectors.

## Scrub, Snap, And Toggle Actions

- `scrub: true` maps animation progress directly to scroll progress.
- Numeric scrub smooths the playhead catch-up by that many seconds.
- `toggleActions` controls discrete animation actions in this order:
  `onEnter`, `onLeave`, `onEnterBack`, `onLeaveBack`.
- `snap` supports increments, arrays, functions, `"labels"`,
  `"labelsDirectional"`, or object config. It belongs on scrubbed timelines or
  standard scrubbed triggers, not on containerAnimation-based triggers.
- `fastScrollEnd` and `preventOverlaps` are useful when quick scrolls cause
  half-played or overlapping trigger animations.
- `once: true` is for one-way triggers that should be killed after the forward
  endpoint. Do not use it when reverse-scroll behavior is required.

## Batch

- `ScrollTrigger.batch(targets, vars)` creates one ScrollTrigger per target and
  batches callbacks within `interval` or until `batchMax` is reached.
- Batched callbacks receive `(elements, scrollTriggers)` arrays. Normal
  callbacks receive one ScrollTrigger instance.
- Batch vars can include standard trigger config such as `start`, `end`, and
  `once`, but should not include `trigger`, `animation`, `scrub`, `snap`,
  `toggleActions`, `onScrubComplete`, or `onSnapComplete`.
- `onRefreshInit` is not batched; it uses the standard callback signature.

## Horizontal Container Animation

- The container tween/timeline must be linear: `ease: "none"`.
- `containerAnimation` triggers measure against the linked animation progress,
  not native horizontal scroll.
- Pinning and snapping are unavailable on ScrollTriggers that use
  `containerAnimation`.
- Avoid animating the trigger element horizontally. Animate child panels or
  offset `start`/`end` to account for the trigger movement.

## Scroller Proxy

- Use `scrollerProxy()` only when a third-party smooth scroller owns scroll
  position. GSAP ScrollSmoother does not need it.
- Provide getter/setter functions for `scrollTop` and/or `scrollLeft`.
- Provide `getBoundingClientRect()` when the scroller's measured rect differs
  from viewport/native expectations.
- Register the smooth scroller's update event with `ScrollTrigger.update`, then
  call `ScrollTrigger.refresh()` after setup.
- Set `fixedMarkers: true` if markers visually drift due to transformed
  scrollers. Choose `pinType: "fixed"` or `"transform"` only when measured
  pinning behavior requires it.
- Do not present third-party smooth scrolling as required. Native scroll or
  ScrollSmoother is usually a smaller accessibility and maintenance surface.

## SPA And Framework Cleanup

- Prefer framework-scoped cleanup: `useGSAP()`, `gsap.context().revert()`,
  `gsap.matchMedia().revert()`, Vue `onUnmounted`, Svelte destroy callbacks,
  or equivalent lifecycle hooks.
- Use `ScrollTrigger.getAll().forEach((trigger) => trigger.kill())` only when
  the whole page/route scope is being disposed.
- Use `ScrollTrigger.clearScrollMemory()` for route transitions when prior
  scroll restoration causes bad fresh calculations.
- Call `ScrollTrigger.refresh()` after async content, image/font layout shifts,
  virtualized content changes, or route transitions that alter trigger
  positions. Avoid refresh calls in scroll, pointer, ticker, or animation-frame
  hot paths.
- `gsap.matchMedia()` automatically reverts GSAP animations and ScrollTriggers
  created in matching handlers when conditions stop matching. Returned cleanup
  functions are for custom non-GSAP work such as event listeners.
