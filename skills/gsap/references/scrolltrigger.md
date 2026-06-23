# GSAP ScrollTrigger

ScrollTrigger ties GSAP tweens and timelines to scroll position for pinning, scrubbing, parallax, reveals, and snapping. It is part of the public `gsap` package and is fully free (including ScrollSmoother) as of v3.13+ under Webflow's 2025 release, so no license gate, trial, or premium plugin install is required.

## Table of contents

- [Registering the plugin](#registering-the-plugin)
- [Basic scrollTrigger config](#basic-scrolltrigger-config)
- [Full config reference](#full-config-reference)
- [Start and end geometry](#start-and-end-geometry)
- [Scrub: true vs numeric smoothing](#scrub-true-vs-numeric-smoothing)
- [toggleActions](#toggleactions)
- [Pinning and pinSpacing](#pinning-and-pinspacing)
- [Snap](#snap)
- [Driving a timeline with one ScrollTrigger](#driving-a-timeline-with-one-scrolltrigger)
- [ScrollTrigger.batch() for staggered reveals](#scrolltriggerbatch-for-staggered-reveals)
- [Horizontal scroll with containerAnimation](#horizontal-scroll-with-containeranimation)
- [Smooth scroll: scrollerProxy and Lenis/Locomotive](#smooth-scroll-scrollerproxy-and-lenislocomotive)
- [refresh() and refreshPriority ordering](#refresh-and-refreshpriority-ordering)
- [Markers (development only)](#markers-development-only)
- [React and Next.js cleanup](#react-and-nextjs-cleanup)
- [prefers-reduced-motion](#prefers-reduced-motion)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## Registering the plugin

ScrollTrigger is a plugin. Register it once, before any usage, in client-side code only:

```typescript
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);
```

In Next.js App Router, registration and all ScrollTrigger creation must run on the client — keep them inside a `"use client"` component, an effect, or `useGSAP()`, never at the module top level of a server component.

## Basic scrollTrigger config

Add a `scrollTrigger` object to any tween (or timeline) to bind it to scroll. Shorthand `scrollTrigger: ".selector"` sets only `trigger` and uses defaults for the rest.

```typescript
gsap.to(".box", {
  x: 500,
  duration: 1,
  scrollTrigger: {
    trigger: ".box",
    start: "top center",   // top of trigger hits center of viewport
    end: "bottom center",  // bottom of trigger hits center of viewport
    toggleActions: "play reverse play reverse",
  },
});
```

For scroll behavior with no linked tween, use `ScrollTrigger.create()` with the same config and drive custom logic from callbacks:

```typescript
ScrollTrigger.create({
  trigger: "#section",
  start: "top top",
  end: "bottom 50%+=100px",
  onUpdate: (self) => console.log(self.progress.toFixed(3), self.direction),
});
```

## Full config reference

| Property | Type | Description |
|----------|------|-------------|
| `trigger` | String \| Element | Element whose position defines where the ScrollTrigger starts. Required (or use the shorthand). |
| `start` | String \| Number \| Function | When the trigger becomes active. Default `"top bottom"` (or `"top top"` when `pin: true`). |
| `end` | String \| Number \| Function | When the trigger deactivates. Default `"bottom top"`. |
| `endTrigger` | String \| Element | Element used to compute `end` when a different element should define it. |
| `scrub` | Boolean \| Number | Links animation progress to scroll. `true` = direct; a number = seconds for the playhead to catch up. |
| `toggleActions` | String | Four discrete actions in order: onEnter, onLeave, onEnterBack, onLeaveBack. Default `"play none none none"`. |
| `pin` | Boolean \| String \| Element | Pin an element while active. `true` pins the trigger. Animate children, not the pinned element itself. |
| `pinSpacing` | Boolean \| String | Default `true` (adds a spacer so layout does not collapse). `false` or `"margin"`. |
| `horizontal` | Boolean | `true` for horizontal scrolling scrollers. |
| `scroller` | String \| Element | Scroll container (default: viewport). Use a selector/element for a scrollable div. |
| `markers` | Boolean \| Object | `true` for dev markers, or `{ startColor, endColor, fontSize, ... }`. Remove for production. |
| `once` | Boolean | If `true`, kills the ScrollTrigger after `end` is reached once (the animation keeps its end state). |
| `id` | String | Unique id for `ScrollTrigger.getById(id)`. |
| `refreshPriority` | Number | Higher = refreshed earlier. Set when triggers are created out of page order. |
| `toggleClass` | String \| Object | Toggle a class while active. String = on trigger; or `{ targets: ".x", className: "active" }`. |
| `snap` | Number \| Array \| Function \| "labels" \| Object | Snap to progress values. See [Snap](#snap). |
| `containerAnimation` | Tween \| Timeline | Tie this trigger to the progress of a horizontal animation instead of native scroll. See [containerAnimation](#horizontal-scroll-with-containeranimation). |
| `invalidateOnRefresh` | Boolean | Re-read function-based and from values on each refresh. Pair with function `end`/`x` for responsive scenes. |
| `anticipatePin` | Number | Pre-applies the pin slightly early to reduce a jump on fast scroll. |
| `pinnedContainer` | Element | Declares an ancestor that is itself pinned, so measurements stay correct. |
| `fastScrollEnd` / `preventOverlaps` | Boolean / String | Mitigate half-played or overlapping animations on fast scroll. |
| `onEnter` / `onLeave` / `onEnterBack` / `onLeaveBack` | Function | Fire when crossing start/end forward or backward. Receive the ScrollTrigger instance. |
| `onUpdate` / `onToggle` / `onRefresh` / `onScrubComplete` | Function | `onUpdate` on progress change; `onToggle` when active flips; `onRefresh` after recalc; `onScrubComplete` when a numeric scrub settles. |

The instance passed to callbacks exposes `progress`, `direction`, `isActive`, `start`, `end`, and `getVelocity()`.

## Start and end geometry

`start` and `end` describe a trigger position relative to a scroller position, written as `"triggerPosition scrollerPosition"`:

- `"top bottom"` — top of the trigger reaches the bottom of the viewport.
- `"top center"`, `"center center"`, `"bottom 80%"`, `"top bottom-=100px"` — keywords, percentages, and pixel offsets all compose.
- A single relative value such as `"+=300"` or `"+=100%"` is relative to the calculated start (`100%` = one scroller height).
- A plain number is an absolute scroll position in pixels; `"max"` is the maximum scroll position.
- A function runs on every refresh and receives the ScrollTrigger instance; use it for responsive measurements and pair with `invalidateOnRefresh: true`.
- `clamp(...)` (v3.12+) keeps values within page scroll bounds, which fixes above-the-fold and near-page-end triggers: `start: "clamp(top bottom)"`, `end: "clamp(bottom top)"`.

Defaults: without pinning, `start: "top bottom"` and `end: "bottom top"`. With `pin: true`, the effective default start is `"top top"`. Use `endTrigger` when a different element should define the end.

## Scrub: true vs numeric smoothing

Scrub links animation progress directly to scroll position so the user "scrubs" the playhead by scrolling:

```typescript
gsap.to(".box", {
  x: 500,
  scrollTrigger: {
    trigger: ".box",
    start: "top center",
    end: "bottom center",
    scrub: true, // direct 1:1 mapping to scroll
  },
});
```

- `scrub: true` maps progress directly to scroll, with no lag.
- `scrub: <number>` (for example `scrub: 1`) gives the playhead that many seconds to "catch up" to the scroll position, producing a smooth, weighted feel. `scrub: 0.5` catches up in half a second.

Scrub and `toggleActions` are mutually exclusive on the same trigger; if both are present, scrub wins. Choose scroll-linked progress (scrub) or discrete play/reverse (toggleActions), not both.

## toggleActions

`toggleActions` is a space-separated string of four actions applied at the four boundary crossings, in this order: onEnter, onLeave, onEnterBack, onLeaveBack. Each slot accepts `play`, `pause`, `resume`, `reset`, `restart`, `complete`, `reverse`, or `none`.

```typescript
gsap.from(".card", {
  autoAlpha: 0,
  y: 40,
  scrollTrigger: {
    trigger: ".card",
    start: "top 80%",
    toggleActions: "play none none reverse",
  },
});
```

Default is `"play none none none"` — play on enter, do nothing on the other three crossings. Use toggleActions for discrete reveals; use [scrub](#scrub-true-vs-numeric-smoothing) for scroll-linked progress.

## Pinning and pinSpacing

Pin holds an element in place while its scroll range is active:

```typescript
gsap.to(".panel", {
  scrollTrigger: {
    trigger: ".section",
    start: "top top",
    end: "+=1000",  // stay pinned for 1000px of scroll
    pin: true,
    scrub: 1,
  },
});
```

- `pinSpacing` defaults to `true`, inserting a spacer so the surrounding layout does not collapse while the pinned element is set to `position: fixed`. Use `pinSpacing: false` only when the layout already reserves that space.
- Do not animate the pinned element itself; pin a wrapper and animate its children.
- Pin order matters: create pinned triggers top-to-bottom or set `refreshPriority` (see below). Pinned scenes mutate page geometry, so they need refresh proof after any layout change.
- `pinnedContainer` declares an ancestor that is also pinned; `pinReparent` can fix transformed-ancestor problems but moves the element under `body` while active and may break CSS selectors, so use it sparingly.

## Snap

`snap` settles scroll to specific progress points after the user stops scrolling. It belongs on scrubbed triggers (standard or timeline-driven), not on containerAnimation-based triggers.

```typescript
const tl = gsap.timeline({
  scrollTrigger: {
    trigger: ".steps",
    start: "top top",
    end: "+=3000",
    pin: true,
    scrub: 1,
    snap: {
      snapTo: "labels",        // also: number increments, [0, 0.5, 1], or a function
      duration: 0.3,
      delay: 0.1,
      ease: "power1.inOut",
    },
  },
});
```

Forms: a number (`0.25` = quarter increments), an array of explicit values, a function, `"labels"`/`"labelsDirectional"` (timeline labels), or an object for full control. `ScrollTrigger.snapDirectional()` is available for custom snapping logic.

## Driving a timeline with one ScrollTrigger

Attach the ScrollTrigger to the timeline, then add child tweens. The timeline's progress maps onto the scroll range:

```typescript
const tl = gsap.timeline({
  scrollTrigger: {
    trigger: ".container",
    start: "top top",
    end: "+=2000",
    scrub: 1,
    pin: true,
  },
});

tl.to(".a", { x: 100 })
  .to(".b", { y: 50 })
  .to(".c", { autoAlpha: 0 });
```

Put the ScrollTrigger only on the top-level timeline (or a top-level tween). Never put a `scrollTrigger` on a child tween inside a timeline.

## ScrollTrigger.batch() for staggered reveals

`ScrollTrigger.batch(targets, vars)` creates one ScrollTrigger per target and groups their callbacks within a short interval, so many elements that enter the viewport together animate as one staggered batch. It is a strong alternative to IntersectionObserver and returns an array of ScrollTrigger instances.

Batched callbacks receive two array arguments — `(elements, scrollTriggers)` — unlike normal callbacks, which receive a single instance.

```typescript
ScrollTrigger.batch(".card", {
  start: "top 80%",
  end: "bottom 20%",
  onEnter: (elements) =>
    gsap.to(elements, { autoAlpha: 1, y: 0, stagger: 0.15, overwrite: true }),
  onLeaveBack: (elements) =>
    gsap.set(elements, { autoAlpha: 0, y: 50, overwrite: true }),
});
```

Tuning options:

- `interval` (Number) — max seconds to collect each batch (defaults to roughly one animation frame). The timer starts when the first callback of a type fires.
- `batchMax` (Number | Function) — max elements per batch; when reached, the callback fires and a new batch begins. A function (evaluated on refresh) keeps it responsive.

Do not pass `trigger` (the targets are the triggers) or animation-coupled options: `animation`, `scrub`, `snap`, `toggleActions`, `invalidateOnRefresh`, `onScrubComplete`, or `onSnapComplete`. (`onRefreshInit` is not batched and uses the standard single-instance signature.)

## Horizontal scroll with containerAnimation

A common pattern pins a section and, as the user scrolls vertically, moves inner content horizontally ("fake" horizontal scroll). Pin the panel, animate the `x`/`xPercent` of a wrapper inside it, and bind that tween to vertical scroll. Then `containerAnimation` lets other triggers measure against the horizontal animation's progress instead of native scroll.

**Critical:** the horizontal tween/timeline must use `ease: "none"`. Any other ease breaks the 1:1 scroll-to-position mapping — the single most common mistake here.

```typescript
const scrollingEl = document.querySelector<HTMLElement>(".horizontal-el")!;
const panel = scrollingEl.parentElement!;
const maxX = () => Math.max(0, scrollingEl.scrollWidth - panel.clientWidth);

const scrollTween = gsap.to(scrollingEl, {
  x: () => -maxX(),
  ease: "none", // required
  scrollTrigger: {
    trigger: panel,
    pin: true,   // pin the wrapper, not the element being animated
    scrub: true,
    start: "top top",
    end: () => `+=${maxX()}`,
    invalidateOnRefresh: true,
  },
});

// Triggers driven by horizontal movement reference the containerAnimation:
gsap.to(".nested-el-1", {
  y: 100,
  scrollTrigger: {
    containerAnimation: scrollTween, // important
    trigger: ".nested-wrapper-1",
    start: "left center", // measured along horizontal movement
    toggleActions: "play none none reset",
  },
});
```

Caveats: pinning and snapping are not available on containerAnimation-based triggers. Do not animate the trigger element horizontally — animate a child; if the trigger itself moves, offset `start`/`end` accordingly.

## Smooth scroll: scrollerProxy and Lenis/Locomotive

GSAP's own **ScrollSmoother** is the built-in smooth-scroll option (now free) and needs no proxy. Third-party libraries split into two cases:

- **Window-scrolling libraries (e.g. Lenis):** these still scroll the real `window`, so ScrollTrigger's native `scrollTop`/`scrollLeft` reads stay correct — **no `scrollerProxy` is needed**. You only have to keep ScrollTrigger in sync: forward scroll events with `lenis.on('scroll', ScrollTrigger.update)` and drive the library's RAF from GSAP's ticker.
- **Transformed / custom-container scrollers (e.g. Locomotive, or any non-window scroller):** here scroll position lives on a transformed element rather than the window, so `ScrollTrigger.scrollerProxy(scroller, vars)` is required to override how ScrollTrigger reads and writes scroll position, using the library's getters/setters instead of native `scrollTop`/`scrollLeft`.

In `vars`, each of `scrollTop`/`scrollLeft` is both getter and setter: called with an argument it sets, called with none it returns the current value; at least one is required. Optional keys: `getBoundingClientRect()` (when the scroller's rect is not the viewport default), `scrollWidth`/`scrollHeight`, `fixedMarkers: true` (markers drift under transformed scrollers), and `pinType: "fixed" | "transform"` (use `"fixed"` if pins jitter, `"transform"` if pins do not stick).

**Critical:** the smooth scroller must notify ScrollTrigger on every update — register `ScrollTrigger.update` as a listener, then call `ScrollTrigger.refresh()` after setup.

```typescript
// Lenis: drive its RAF from GSAP's ticker and forward scroll events.
import Lenis from "lenis";

const lenis = new Lenis();
lenis.on("scroll", ScrollTrigger.update);
gsap.ticker.add((time) => lenis.raf(time * 1000));
gsap.ticker.lagSmoothing(0);

// Transformed custom scroller (e.g. Locomotive): proxy read/write + pinType.
ScrollTrigger.scrollerProxy(".smooth-wrapper", {
  scrollTop(value) {
    if (arguments.length) scroller.scrollTop = value as number;
    return scroller.scrollTop;
  },
  getBoundingClientRect() {
    return { top: 0, left: 0, width: window.innerWidth, height: window.innerHeight };
  },
  pinType: "transform", // transformed scroller; use "fixed" for native overflow
});
scroller.addListener?.(ScrollTrigger.update);
ScrollTrigger.refresh();
```

Centralize smooth-scroll integration at one app-level owner; do not configure proxy behavior ad hoc per component, and do not present third-party smooth scrolling as required — native scroll or ScrollSmoother is usually a smaller accessibility and maintenance surface.

## refresh() and refreshPriority ordering

`ScrollTrigger.refresh()` recalculates every trigger's positions. It runs automatically on viewport resize (debounced ~200ms), but dynamic layout changes are not auto-detected. Call `refresh()` after:

- Images and fonts finish loading (use `imagesLoaded`, `img.decode()`, or `document.fonts.ready`).
- Async or CMS content renders, accordions/tabs expand, or virtualized lists change height.
- Route transitions that alter trigger positions.

Avoid calling `refresh()` in scroll, pointer, ticker, or animation-frame hot paths; refresh on the discrete events that actually change layout, not on a timer.

Refresh runs in creation order, sorted by `refreshPriority` (higher refreshes earlier). When triggers are created top-to-bottom in page order, the default ordering is correct and you do not need `refreshPriority`. Only set it when async or out-of-order creation makes ordering unavoidable — give earlier-on-page sections a higher number than later ones so pin spacing is computed in document order.

```typescript
await document.fonts.ready;
ScrollTrigger.refresh();
```

For route transitions where prior scroll restoration causes bad fresh calculations, `ScrollTrigger.clearScrollMemory()` resets stored scroll positions.

## Markers (development only)

Set `markers: true` to visualize start/end positions during development:

```typescript
scrollTrigger: {
  trigger: ".box",
  start: "top center",
  end: "bottom center",
  markers: true, // remove before production
}
```

Always remove markers (or set `markers: false`) before shipping. Use `fixedMarkers: true` in a scrollerProxy if markers drift under a transformed smooth scroller.

## React and Next.js cleanup

Use the `useGSAP()` hook from `@gsap/react`. It scopes selectors and automatically reverts every tween, timeline, and ScrollTrigger created inside it when the component unmounts, which is essential for App Router route changes and Fast Refresh.

```tsx
"use client";

import { useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { useGSAP } from "@gsap/react";

gsap.registerPlugin(ScrollTrigger, useGSAP);

export function ParallaxHero() {
  const root = useRef<HTMLDivElement>(null);

  useGSAP(
    () => {
      gsap.to(".hero-bg", {
        yPercent: -30,
        ease: "none",
        scrollTrigger: {
          trigger: root.current,
          start: "top top",
          end: "bottom top",
          scrub: true,
        },
      });
    },
    { scope: root },
  );

  return (
    <div ref={root}>
      <div className="hero-bg" />
    </div>
  );
}
```

`useGSAP` reverts on unmount and route change automatically. Pass `{ scope: root, dependencies: [pathname] }` to re-run per route, and return a cleanup function for manual escape hatches (`ScrollTrigger.getById("hero")?.kill()`). Use the page-wide sledgehammer `ScrollTrigger.getAll().forEach((t) => t.kill())` only when disposing a whole route scope.

Never create ScrollTriggers during SSR — guard everything behind `"use client"` and effects/`useGSAP`, since ScrollTrigger needs `window` and a measured DOM.

## prefers-reduced-motion

Use `gsap.matchMedia()` for responsive and reduced-motion scenes. It creates a context that automatically reverts the GSAP animations and ScrollTriggers it owns when conditions stop matching, so you do not hand-roll listener cleanup.

> Note: `ScrollTrigger.matchMedia()` is deprecated. Use `gsap.matchMedia()` instead.

```typescript
const mm = gsap.matchMedia();

mm.add(
  {
    reduceMotion: "(prefers-reduced-motion: reduce)",
    fullMotion: "(prefers-reduced-motion: no-preference)",
  },
  (context) => {
    const { reduceMotion } = context.conditions as { reduceMotion: boolean };

    if (reduceMotion) {
      gsap.set(".hero", { autoAlpha: 1, y: 0 }); // static, accessible state
      return;
    }

    gsap.to(".hero", {
      y: -80,
      scrollTrigger: { trigger: ".hero", start: "top top", end: "bottom top", scrub: true },
    });
  },
);

// On teardown (or via useGSAP, which reverts automatically):
mm.revert();
```

Pair every nonessential pin, scrub, parallax, or scroll-linked reveal with a static or short opacity-only variant so reduced-motion users get a usable, non-disorienting layout. The returned cleanup function from a matchMedia handler is only for custom non-GSAP work (event listeners, observers); GSAP-owned animations are reverted for you.

## Pitfalls / Do-not

- **Do not** put `scrollTrigger` on a child tween of a timeline. Attach it to the timeline or a top-level tween only. Wrong: `gsap.timeline().to(".a", { scrollTrigger: {...} })`. Right: `gsap.timeline({ scrollTrigger: {...} }).to(".a", { x: 100 })`.
- **Do not** mix `scrub` and `toggleActions` on the same trigger. Pick one; if both are set, scrub wins.
- **Do not** forget `gsap.registerPlugin(ScrollTrigger)` before any ScrollTrigger usage.
- **Do not** use an ease other than `"none"` on the horizontal animation behind `containerAnimation`; it breaks the 1:1 scroll mapping.
- **Do not** create triggers in random or async order without `refreshPriority`. Refresh runs in creation order, and wrong order corrupts pin spacing.
- **Do not** rely on auto-refresh for content changes. Resize is auto-handled; images, fonts, async content, and layout shifts are not — call `ScrollTrigger.refresh()` explicitly after they settle.
- **Do not** call `refresh()` in scroll/pointer/ticker hot paths; refresh on discrete layout events only.
- **Do not** animate the pinned element itself; pin a wrapper and animate its children.
- **Do not** leave `markers: true` in production.
- **Do not** create ScrollTriggers during SSR. Guard with `"use client"` and effects/`useGSAP`; ScrollTrigger needs `window` and a measured DOM.
- **Do not** let multiple components each create their own smooth-scroll proxy. Centralize integration at one owner.
- **Do not** forget to register `ScrollTrigger.update` with a third-party smooth scroller, or its positions go stale.
- **Do not** skip teardown on route change. Use `useGSAP`/`gsap.matchMedia().revert()`/`gsap.context().revert()`, or `kill()` triggers for the disposed scope.

## Related references

- [Core API](./core.md)
- [React & Next.js](./react-nextjs.md)
- [Timeline](./timeline.md)
- [Plugins](./plugins.md)
- [Performance](./performance.md)
- [Recipes](./recipes.md)
