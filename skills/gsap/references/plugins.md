# GSAP Plugins

GSAP ships a deep plugin library for scroll, layout, drag, input, text, SVG, easing, and physics work. This reference covers registration discipline plus the plugins you reach for most in Next.js + React + TypeScript apps, tiered by how often you'll use them.

## GSAP is 100% free (as of 2025)

Since [Webflow's acquisition of GSAP](https://gsap.com/blog/webflow-GSAP/), **every GSAP plugin is free, including for commercial use**. The formerly "premium"/Club plugins — **SplitText, MorphSVG, DrawSVG, ScrollSmoother, MotionPath, InertiaPlugin, Physics2D, GSDevTools** — all ship in the public `gsap` npm package.

- There is **no Club GreenSock**, **no `gsap-trial`**, **no bonus/private registry**, and **no license key or auth token**.
- Install everything with `npm install gsap`. Import any plugin from its submodule (e.g. `gsap/SplitText`, `gsap/MorphSVGPlugin`).
- Do **not** generate an `.npmrc` with a GreenSock token, point at `npm.greensock.com`, or tell users to join Club GSAP to "unlock" a plugin. That guidance is obsolete.

Use `gsap/dist/<PluginName>` only when a bundler specifically needs UMD files; ESM submodule imports are the default.

## Table of contents

**Tier 1 (common)**

- [ScrollToPlugin](#scrolltoplugin) — animate scroll position
- [ScrollSmoother](#scrollsmoother) — smooth/momentum native scroll
- [Flip](#flip) — animate between layout states (FLIP)
- [Draggable (+ InertiaPlugin)](#draggable--inertiaplugin) — drag, throw, momentum
- [Observer](#observer) — unified pointer/wheel/touch input
- [SplitText](#splittext) — split into chars/words/lines, accessible
- [DrawSVG](#drawsvg) — animate stroke reveal
- [MorphSVG](#morphsvg) — morph one shape into another
- [MotionPath](#motionpath) — animate along a path
- [CustomEase](#customease) — custom easing curves
- [ScrollTrigger](#scrolltrigger) — see `./scrolltrigger.md`

**Tier 2 / niche** — [Niche & specialized plugins](#niche--specialized-plugins): EasePack, CustomWiggle, CustomBounce, ScrambleText, Physics2D, PhysicsProps, Pixi, GSDevTools, MotionPathHelper

## Registration discipline

Register each plugin **once**, at the app/client boundary, importing from the plugin's submodule so the bundler can tree-shake everything you don't use. Registration must happen **before** the first tween or API call that uses the plugin.

```tsx
// app/gsap.ts  (or a small client-only module imported once)
import { gsap } from "gsap";
import { useGSAP } from "@gsap/react";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { ScrollToPlugin } from "gsap/ScrollToPlugin";
import { Flip } from "gsap/Flip";
import { Draggable } from "gsap/Draggable";
import { InertiaPlugin } from "gsap/InertiaPlugin";
import { Observer } from "gsap/Observer";
import { SplitText } from "gsap/SplitText";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { MorphSVGPlugin } from "gsap/MorphSVGPlugin";
import { MotionPathPlugin } from "gsap/MotionPathPlugin";
import { CustomEase } from "gsap/CustomEase";

// useGSAP is itself a plugin — register it too.
gsap.registerPlugin(
  useGSAP,
  ScrollTrigger,
  ScrollToPlugin,
  Flip,
  Draggable,
  InertiaPlugin,
  Observer,
  SplitText,
  DrawSVGPlugin,
  MorphSVGPlugin,
  MotionPathPlugin,
  CustomEase,
);
```

- Register at module top level or once in the app shell — **never inside a component that re-renders**. Re-registering on every render is wasteful and a sign of a misplaced boundary.
- In Next.js App Router, keep registration in a `"use client"` module. Plugin *instances* (Draggable, Observer, SplitText, ScrollSmoother) are DOM objects — create them after mount, inside `useGSAP()`.
- Plugin modules guard direct `window` access, so importing/registering is SSR-safe; only *creating instances* requires the client.

**Cleanup, everywhere:** stateful plugin instances must be reverted/killed on unmount. Inside `useGSAP(() => { ... }, { scope })`, GSAP-created tweens and instances are reverted automatically. For instances you store yourself (or non-GSAP listeners), return a cleanup function or call `.kill()` / `.revert()` explicitly. Each Tier-1 plugin below notes its cleanup call.

## ScrollToPlugin

Animates scroll position of the window or any scrollable element — "scroll to element" / "scroll to position" **without** ScrollTrigger.

```tsx
gsap.registerPlugin(ScrollToPlugin);

gsap.to(window, { duration: 1, scrollTo: { y: 500 } });
gsap.to(window, { duration: 1, scrollTo: { y: "#section", offsetY: 50 } });
gsap.to(scrollContainer, { duration: 1, scrollTo: { x: "max" } });
```

`scrollTo` object: `x` / `y` (number or `"max"`), `y: "#selector"` to scroll an element into view, `offsetX` / `offsetY` (pixel offset from target). Stateless — no cleanup beyond the tween itself (which `useGSAP` handles).

## ScrollSmoother

Wraps native scroll to add smooth, momentum-style scrolling. Requires **ScrollTrigger** and a specific DOM structure: a `#smooth-wrapper` containing a `#smooth-content`. It uses native page scrolling but transforms the content wrapper, so `position: fixed` elements usually belong **outside** the wrapper/content pair.

```tsx
<body>
  <div id="smooth-wrapper">
    <div id="smooth-content">{/* all scrolling content */}</div>
  </div>
  {/* fixed nav / overlays live here, outside the wrapper */}
</body>
```

```tsx
"use client";
import { useGSAP } from "@gsap/react";
import { ScrollSmoother } from "gsap/ScrollSmoother";

export function SmoothScroll({ children }: { children: React.ReactNode }) {
  useGSAP(() => {
    const smoother = ScrollSmoother.create({
      wrapper: "#smooth-wrapper",
      content: "#smooth-content",
      smooth: 1.2,
      effects: true, // enables data-speed / data-lag parallax attributes
    });
    return () => smoother.kill(); // restores inline CSS + listeners
  });

  return (
    <div id="smooth-wrapper">
      <div id="smooth-content">{children}</div>
    </div>
  );
}
```

Register **after** ScrollTrigger, and create ScrollSmoother **before** any downstream ScrollTriggers that depend on its transformed content. **Cleanup:** `ScrollSmoother.get()?.kill()` or the returned `smoother.kill()`.

## Flip

FLIP = First, Last, Invert, Play. Capture state with `Flip.getState()`, mutate the DOM (reorder, add/remove, toggle classes, change layout), then `Flip.from(state, vars)` animates from the old layout to the new one. Use it for list/grid reorders, expand/collapse, and shared-element transitions.

```tsx
"use client";
import { useGSAP } from "@gsap/react";
import { Flip } from "gsap/Flip";
import { useRef, useState } from "react";

export function ReorderGrid({ items }: { items: string[] }) {
  const scope = useRef<HTMLUListElement>(null);
  const lastState = useRef<Flip.FlipState | null>(null);
  const [order, setOrder] = useState(items);

  const shuffle = () => {
    // 1. Capture BEFORE the state change, while the DOM still has the old layout.
    lastState.current = Flip.getState(scope.current!.querySelectorAll("li"));
    // 2. Change the DOM (here, via React state -> new order).
    setOrder((o) => [...o].reverse());
  };

  // 3. After React commits the new layout, animate from the captured state.
  //    Keyed on `order` so it runs once per reorder, not on a racy single rAF.
  useGSAP(() => {
    if (lastState.current) {
      Flip.from(lastState.current, { duration: 0.5, ease: "power2.inOut", absolute: true });
    }
  }, { dependencies: [order], scope });

  return (
    <>
      <button onClick={shuffle}>Shuffle</button>
      <ul ref={scope}>{order.map((i) => <li key={i}>{i}</li>)}</ul>
    </>
  );
}
```

Key `Flip.from` vars: `absolute` (use `position: absolute` during the flip — smooths reorders), `nested` (only measure first-level children), `scale` (scale to fit instead of stretch; default `true`), `simple` (position/scale only — faster, less accurate), plus standard `duration` / `ease`. **Cleanup:** the resulting tween is reverted by `useGSAP`; Flip owns the measured before/after state, so don't mix it with a separate layout animator on the same elements.

## Draggable (+ InertiaPlugin)

Makes elements draggable, spinnable, or throwable with mouse/touch. Pair with **InertiaPlugin** and `inertia: true` for throw/momentum (glide-to-stop after release). `Draggable.create()` returns an **array** of instances.

```tsx
"use client";
import { useGSAP } from "@gsap/react";
import { Draggable } from "gsap/Draggable";
import { InertiaPlugin } from "gsap/InertiaPlugin";
import { useRef } from "react";

export function DragCard() {
  const box = useRef<HTMLDivElement>(null);
  const container = useRef<HTMLDivElement>(null);

  useGSAP(() => {
    const [draggable] = Draggable.create(box.current, {
      type: "x,y",
      bounds: container.current,
      inertia: true, // requires InertiaPlugin registered
      edgeResistance: 0.65,
    });
    return () => draggable.kill(); // removes listeners, disables instance
  }, { scope: container });

  return (
    <div ref={container} className="relative h-80">
      <div ref={box} className="absolute h-20 w-20 cursor-grab bg-sky-500" />
    </div>
  );
}
```

Key options: `type` (`"x"`, `"y"`, `"x,y"`, `"rotation"`, `"scroll"`), `bounds` (element/selector or `{ minX, maxX, minY, maxY }`), `inertia` (`true` for throw/momentum), `edgeResistance` (0–1), `cursor`, plus `onDragStart` / `onDrag` / `onDragEnd` and `onThrowUpdate` / `onThrowComplete` callbacks. **InertiaPlugin** also tracks velocity of any property — `InertiaPlugin.track(node, "x")`, then `gsap.to(obj, { inertia: { x: "auto" } })` continues current velocity to a stop. **Cleanup:** `draggable.kill()`. Provide a keyboard/pointer fallback for accessibility when drag is the only way to perform an action.

## Observer

Normalizes pointer, wheel, and touch input across devices into directional callbacks. Use it for swipe detection, scroll-direction logic, and custom gestures **without** tying animation progress to scroll position (use ScrollTrigger for that).

```tsx
"use client";
import { useGSAP } from "@gsap/react";
import { Observer } from "gsap/Observer";
import { useRef } from "react";

export function SwipeArea() {
  const area = useRef<HTMLDivElement>(null);

  useGSAP(() => {
    const observer = Observer.create({
      target: area.current,
      type: "wheel,touch,pointer",
      onUp: () => goTo("prev"),
      onDown: () => goTo("next"),
      onLeft: () => goTo("next"),
      onRight: () => goTo("prev"),
      tolerance: 10,
      preventDefault: true,
    });
    return () => observer.kill();
  }, { scope: area });

  return <div ref={area} className="h-screen" />;
}
```

Key options: `target`, `type` (`"wheel"`, `"touch"`, `"pointer"` — combine with commas; default `"wheel,touch,pointer"`), `onUp` / `onDown` / `onLeft` / `onRight` (fire once tolerance is passed), `tolerance` (pixels before a direction registers; default 10), `preventDefault`. **Cleanup:** `observer.kill()`.

## SplitText

Splits an element's text into **chars**, **words**, and/or **lines**, each wrapped in its own element, for per-unit staggered animation. `SplitText.create(target, vars)` returns an instance exposing `chars`, `words`, `lines` arrays (and `masks` when `mask` is set). It integrates with `gsap.context()`, `gsap.matchMedia()`, and `useGSAP()`.

```tsx
"use client";
import { useGSAP } from "@gsap/react";
import { SplitText } from "gsap/SplitText";
import { useRef } from "react";

export function RevealHeading({ text }: { text: string }) {
  const scope = useRef<HTMLHeadingElement>(null);

  useGSAP(() => {
    const split = SplitText.create(scope.current, {
      type: "lines, words",
      mask: "lines", // wrap each line for overflow:clip reveal
      autoSplit: true, // re-split when fonts load / width changes
      onSplit(self) {
        // Return the tween so SplitText reverts + time-syncs it on re-split.
        return gsap.from(self.lines, {
          yPercent: 100,
          opacity: 0,
          stagger: 0.06,
          duration: 0.6,
          ease: "power3.out",
        });
      },
    });
    return () => split.revert(); // restores original markup
  }, { scope });

  return <h1 ref={scope}>{text}</h1>;
}
```

Key vars: **`type`** (comma list of `"chars"`/`"words"`/`"lines"` — split only what you animate for performance), **`autoSplit`** (revert + re-split when fonts finish loading or width changes; create animations **inside `onSplit()`** and **return** them for auto cleanup/time-sync), **`onSplit(self)`** (runs on each split/re-split), **`mask`** (`"lines"`/`"words"`/`"chars"` — wraps each unit in an `overflow: clip` element for reveal effects; wrappers on `self.masks`), **`aria`** (accessibility — `"auto"` (default) keeps an `aria-label` on the element and `aria-hidden` on the split pieces so readers read the whole label; `"hidden"` hides all; `"none"` leaves aria untouched — use `"none"` plus a screen-reader-only duplicate when nested link/semantic content must stay exposed), plus `charsClass`/`wordsClass`/`linesClass`, `smartWrap`, `wordDelimiter`, `propIndex`.

**Accessibility & reduced motion:** keep `aria: "auto"` so screen readers still read the heading; gate the animation behind `prefers-reduced-motion` via `gsap.matchMedia()` and render the final state immediately for users who opt out. **Tips:** split after custom fonts load (`document.fonts.ready`) or rely on `autoSplit`; avoid `text-wrap: balance` (interferes with line splitting); SplitText does not support SVG `<text>`. **Cleanup:** `split.revert()` restores markup; `split.kill()` also stops `autoSplit` listeners.

## DrawSVG

Animates the **stroke** of SVG elements (via `stroke-dasharray` / `stroke-dashoffset`) to "draw" or "erase" them. Works on `<path>`, `<line>`, `<polyline>`, `<polygon>`, `<rect>`, `<ellipse>`. The element **must have a visible stroke** (`stroke` + `stroke-width`) or nothing renders. Stroke only — it never affects fill.

The `drawSVG` value describes the **visible segment** (`"start end"` in % or length), not a time range. `"0% 100%"` = full stroke; `"20% 80%"` = visible only between 20% and 80%; a single value like `0` or `"100%"` implies a start of 0.

```tsx
gsap.registerPlugin(DrawSVGPlugin);

// Draw from nothing to a full stroke.
gsap.from("#path", { duration: 1, drawSVG: 0 });
// Explicit segment animation.
gsap.fromTo("#path", { drawSVG: "0% 0%" }, { drawSVG: "0% 100%", duration: 1 });
// Visible only in the middle (gaps at both ends).
gsap.to("#path", { duration: 1, drawSVG: "20% 80%" });
```

Prefer single-segment `<path>` elements (multi-segment paths can render oddly in some browsers). `DrawSVGPlugin.getLength(el)` / `.getPosition(el)` return stroke length and current position. **Cleanup:** the tween is reverted by `useGSAP`; no stateful instance.

## MorphSVG

Morphs one SVG shape into another by animating the `d` attribute. Start and end shapes need not share point counts — MorphSVG converts to cubic beziers and adds points as needed. Works on `<path>`, `<polyline>`, `<polygon>`; convert `<circle>`/`<rect>`/`<ellipse>`/`<line>` first with `MorphSVGPlugin.convertToPath(...)`.

```tsx
gsap.registerPlugin(MorphSVGPlugin);

MorphSVGPlugin.convertToPath("circle, rect, ellipse, line"); // if needed

gsap.to("#diamond", { duration: 1, morphSVG: "#lightning", ease: "power2.inOut" });
// Object form for full control:
gsap.to("#diamond", {
  duration: 1,
  morphSVG: { shape: "#lightning", type: "rotational", shapeIndex: 2 },
});
```

Key `morphSVG` object options: **`shape`** (required — selector, element, or raw path/points string), **`type`** (`"linear"` default or `"rotational"` — try rotational when linear kinks mid-morph), **`map`** (segment matching: `"size"` default, `"position"`, `"complexity"`), **`shapeIndex`** (offset of which start point maps to the first end point — fixes "crossing over"/inversion; number for single-segment, array for multi-segment, negative reverses; use `shapeIndex: "log"` once to print the auto value), plus `smooth`, `curveMode`, `origin`, `precision`, `precompile`. **Tips:** for twisted morphs set `shapeIndex` (use `"log"`); `precompile` only fixes slow first-frame startup, not per-frame jank — simplify the SVG for jank. **Cleanup:** tween reverted by `useGSAP`; the plugin stores the original `d` so you can morph back to the same element/id.

## MotionPath

Animates an element along an SVG path (or raw path data). Optionally aligns the element to the path and rotates it to follow the tangent.

```tsx
gsap.registerPlugin(MotionPathPlugin);

gsap.to(".dot", {
  duration: 2,
  ease: "none",
  motionPath: {
    path: "#route",
    align: "#route",
    alignOrigin: [0.5, 0.5],
    autoRotate: true,
  },
});
```

Key `motionPath` options: `path` (path element/selector or path-data string), `align` (element/selector to align the target to), `alignOrigin` (`[x, y]` 0–1; default `[0.5, 0.5]`), `autoRotate` (rotate to follow the path tangent), `curviness` (0–2 path smoothing). **Cleanup:** tween reverted by `useGSAP`. (For tuning a path interactively, see `MotionPathHelper` under niche plugins — dev only.)

## CustomEase

Creates arbitrary easing curves from a cubic-bezier string or SVG path data when a built-in ease isn't enough. Register once, then reference the created ease by name or by the returned ease.

```tsx
gsap.registerPlugin(CustomEase);

CustomEase.create("hop", "M0,0 C0.1,0.8 0.2,1 0.5,1 0.8,1 0.9,0.2 1,0");
gsap.to(".el", { x: 100, duration: 1, ease: "hop" });
```

Stateless — no cleanup beyond the tween. (Core easing basics live in `./core.md`.)

## ScrollTrigger

ScrollTrigger (scroll-linked animation, pinning, scrub, trigger configuration) has its own dedicated reference. See **[`./scrolltrigger.md`](./scrolltrigger.md)**. Remember to register it before ScrollSmoother, and create ScrollSmoother before dependent ScrollTriggers.

## Niche & specialized plugins

Register and import these exactly like the Tier-1 plugins (all free, all in the `gsap` package). Reach for them only when a specific effect calls for them.

- **EasePack** (`gsap/EasePack`) — extra named eases: `SlowMo` (slow-motion start/end with a configurable linear middle), `RoughEase` (jagged/randomized motion), and `ExpoScaleEase` (corrects perceived speed when animating `scale`). Configure via the ease string, e.g. `ease: "slow(0.7, 0.7, false)"`.
- **CustomWiggle** (`gsap/CustomWiggle`, needs `CustomEase`) — wiggle/shake easing with multiple oscillations: `CustomWiggle.create("wig", { wiggles: 6, type: "easeOut" })`, then `ease: "wig"`.
- **CustomBounce** (`gsap/CustomBounce`, needs `CustomEase`) — configurable bounce easing (and a matching squash ease): `CustomBounce.create("bnc", { strength: 0.6, squash: 2 })`.
- **ScrambleText** (`gsap/ScrambleTextPlugin`) — reveals/transitions text with a scramble/glitch effect: `gsap.to(".text", { duration: 1, scrambleText: { text: "New message", chars: "01", revealDelay: 0.5 } })`. Keep the final text accessible (the visible text resolves to the real string).
- **Physics2D** (`gsap/Physics2DPlugin`) — simple 2D physics (velocity, angle, gravity, friction) for projectiles/particles: `gsap.to(".ball", { duration: 2, physics2D: { velocity: 250, angle: 80, gravity: 500 } })`.
- **PhysicsProps** (`gsap/PhysicsPropsPlugin`) — applies independent velocity/acceleration to arbitrary properties: `gsap.to(".obj", { duration: 2, physicsProps: { x: { velocity: 100, end: 300 }, y: { velocity: -50, acceleration: 200 } } })`.
- **Pixi** (`gsap/PixiPlugin`) — animates PixiJS display objects through GSAP: `gsap.to(sprite, { pixi: { x: 200, y: 100, scale: 1.5 }, duration: 1 })`. Pass the Pixi import to the plugin (`PixiPlugin.registerPIXI(PIXI)`) when required by your Pixi version.
- **GSDevTools** (`gsap/GSDevTools`) — **dev-only** UI for scrubbing/toggling timelines: `GSDevTools.create({ animation: tl })`. **Never ship to production** — strip it or guard it behind a dev flag.
- **MotionPathHelper** (`gsap/MotionPathHelper`) — **dev-only** visual editor for tuning MotionPath alignment/offset: `const helper = MotionPathHelper.create(".dot", { path: "#path" })`. Note the API takes the path inside the vars object; do not register a `MotionPathHelperPlugin`. **Cleanup:** `helper.kill()` removes the editing controls. Use it to author a path, then hard-code the result and remove the helper.

## Pitfalls / Do-not

- **Don't use a plugin without registering it.** `gsap.registerPlugin(Plugin)` must run before the first tween/API call, or the plugin's properties (`scrollTo`, `morphSVG`, `drawSVG`, etc.) are silently ignored.
- **Don't register inside re-rendering components.** Register once at the app/client boundary; importing from submodules keeps unused plugins out of the bundle (tree-shaking).
- **Don't ship dev tools to production.** `GSDevTools` and `MotionPathHelper` are authoring aids — remove or dev-gate them.
- **Don't skip cleanup.** Revert/kill stateful instances on unmount: `split.revert()`, `draggable.kill()`, `observer.kill()`, `ScrollSmoother.get()?.kill()`, `helper.kill()`. `useGSAP()` reverts GSAP-created work in its scope automatically — lean on it.
- **Don't reach for outdated licensing.** No Club GSAP, no `gsap-trial`, no auth tokens, no private registry. Everything installs from `gsap`.
- **Don't animate without a fallback.** Provide `prefers-reduced-motion` alternatives (via `gsap.matchMedia()`) and keyboard/pointer fallbacks for drag/gesture interactions; keep `aria` intact for SplitText.

## Related references

- [Core API](./core.md)
- [React & Next.js](./react-nextjs.md)
- [ScrollTrigger](./scrolltrigger.md)
- [Timeline](./timeline.md)
- [Performance](./performance.md)
- [Recipes](./recipes.md)
