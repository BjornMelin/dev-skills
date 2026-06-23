# GSAP Performance

Smooth 60fps animation is mostly about animating cheap properties, promoting layers sparingly, and never doing layout work on the hot path. GSAP is fast by default — the wins below come from feeding it the right properties and cleaning up after yourself, not from licensing tiers (GSAP is 100% free and open, v3.13+).

## Contents

- [The property-cost mental model](#the-property-cost-mental-model)
- [Prefer transform and opacity](#prefer-transform-and-opacity)
- [will-change discipline](#will-change-discipline)
- [Batch reads and writes (avoid layout thrash)](#batch-reads-and-writes-avoid-layout-thrash)
- [Stagger vs manual delay chains](#stagger-vs-manual-delay-chains)
- [High-frequency updates: quickTo / quickSetter](#high-frequency-updates-quickto--quicksetter)
- [ScrollTrigger performance](#scrolltrigger-performance)
- [Kill or pause off-screen animations](#kill-or-pause-off-screen-animations)
- [Reduced motion as a perf + a11y lever](#reduced-motion-as-a-perf--a11y-lever)
- [Profiling playbook](#profiling-playbook)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## The property-cost mental model

Every animated property falls into one of four buckets. The browser runs the pipeline top-to-bottom; the lower a property triggers, the more work per frame:

| Cost tier | What runs each frame | GSAP properties | Verdict |
|-----------|----------------------|-----------------|---------|
| **Composite** (cheapest) | GPU layer transform/blend only | `x`, `y`, `z`, `xPercent`, `yPercent`, `scale`, `scaleX/Y`, `rotation`, `rotationX/Y`, `skewX/Y`, `opacity`/`autoAlpha` | Animate freely — the 60fps fast path |
| **Paint** | Re-rasterize pixels (no reflow) | `backgroundColor`, `color`, `boxShadow`, `filter`, `borderRadius`, `clipPath` | OK in moderation; paint-heavy on large areas |
| **Layout** (most expensive) | Reflow the geometry of the page | `width`, `height`, `top`, `left`, `right`, `bottom`, `margin`, `padding`, `fontSize` | Avoid on the hot path — causes jank |
| **JavaScript** | Your `onUpdate` / callbacks | function-based values, plugins, scroll callbacks | Keep callbacks tiny; never read layout inside them |

Rule of thumb: if a transform alias can produce the same visual result as a layout property, use the transform. Promoting an element to a composited layer is not free either — a large layer trades layout cost for memory, texture upload, blending, and (for text) re-rasterization cost. Measure paint-heavy effects (`filter`, `boxShadow`, large blurs) rather than assuming they composite cheaply.

## Prefer transform and opacity

Animating **transform** (`x`, `y`, `scale`, `rotation`, `skew`) and **opacity** keeps work on the compositor and skips layout and most paint. GSAP's `x`/`y` use `translate()` under the hood, so reach for them instead of `left`/`top` for movement.

```tsx
// Good — composited, stays on the GPU
gsap.to(".card", { x: 200, scale: 1.1, autoAlpha: 1, duration: 0.6 });

// Avoid — every frame reflows the page
gsap.to(".card", { left: 200, width: 320, marginTop: 40, duration: 0.6 });
```

Prefer `autoAlpha` over raw `opacity` for fades: at `0` it also sets `visibility: hidden`, so the element costs nothing to render and stops intercepting clicks. See [Core API](./core.md).

## will-change discipline

`will-change: transform` hints the browser to promote an element to its own layer ahead of time. It is a **scoped** hint, not a global optimization — set it on too many elements and you blow the layer budget, spike GPU memory, and make things slower.

- Promote **only** elements that are actively animating, and **only** for the lifetime of the effect.
- Add it just before the animation and remove it on completion/cleanup.
- Never ship a global rule (`* { will-change: transform }` or a shared component class) — that is the most common will-change mistake.
- Treat `force3D`, `translateZ(0)`, and similar promotion hacks as measured exceptions, not defaults. They do not substitute for reducing animated area or paint cost.

```tsx
const tween = gsap.to(el, {
  x: 300,
  duration: 0.8,
  onStart: () => { el.style.willChange = "transform"; },
  onComplete: () => { el.style.willChange = "auto"; }, // release the layer
});
```

For repeating/idle UI, leave `will-change` off until the interaction begins.

## Batch reads and writes (avoid layout thrash)

Layout thrash happens when you interleave DOM **reads** (anything that forces the browser to recompute geometry — `offsetWidth`, `getBoundingClientRect`, `scrollTop`, computed styles) with **writes** (style changes) in a loop, forcing a synchronous reflow each iteration.

GSAP already batches its own writes across all tweens into one flush per frame. The danger is **your** code around it. Do **all reads first, then all writes**:

```tsx
// Bad — read, write, read, write... forces reflow each loop
items.forEach((el) => {
  const w = el.offsetWidth;          // read (reflow)
  gsap.set(el, { x: w / 2 });        // write
});

// Good — batch reads, then batch writes
const widths = items.map((el) => el.offsetWidth); // all reads
items.forEach((el, i) => gsap.set(el, { x: widths[i] / 2 })); // all writes
```

Inside `onUpdate` and scroll callbacks this matters most — never call `getBoundingClientRect()` per frame. Cache measured values and recompute only on resize/refresh.

## Stagger vs manual delay chains

When many elements share the same animation, use **`stagger`** instead of N separate tweens with incrementing `delay`. One tween with a stagger is one instance for GSAP to track, schedule, and clean up — cheaper to create and far easier to kill or reverse as a unit.

```tsx
// Good — one tween, GSAP spreads the start times
gsap.from(".item", { y: 24, autoAlpha: 0, stagger: 0.06, duration: 0.4 });

// Avoid — N tweens, N delays, N handles to manage
items.forEach((el, i) => gsap.from(el, { y: 24, autoAlpha: 0, delay: i * 0.06 }));
```

For long lists, animate only visible items (virtualization or an IntersectionObserver) rather than instantiating hundreds of tweens at once. Use a [Timeline](./timeline.md) when steps must overlap with precise offsets, and reuse it — don't build new timelines every frame.

## High-frequency updates: quickTo / quickSetter

For values updated many times per second (mouse followers, scroll-linked position, pointer parallax), creating a fresh tween on every event is wasteful. Reuse **one** tween:

- **`gsap.quickTo(target, property, vars)`** — returns a function that animates `property` toward the latest value with easing/inertia. Ideal for smooth, lagging followers.
- **`gsap.quickSetter(target, property, unit?)`** — returns a function that writes the value **instantly** (no tween). Use when you don't want easing, just the cheapest possible per-frame write.

```tsx
"use client";
import { useRef } from "react";
import gsap from "gsap";
import { useGSAP } from "@gsap/react";

export function MouseFollower() {
  const dot = useRef<HTMLDivElement>(null);

  useGSAP(() => {
    const el = dot.current!;
    // Build the setters ONCE, reuse on every move — no per-event tween churn
    const xTo = gsap.quickTo(el, "x", { duration: 0.4, ease: "power3" });
    const yTo = gsap.quickTo(el, "y", { duration: 0.4, ease: "power3" });

    const onMove = (e: PointerEvent) => {
      xTo(e.clientX);
      yTo(e.clientY);
    };
    window.addEventListener("pointermove", onMove);
    return () => window.removeEventListener("pointermove", onMove);
  }); // useGSAP reverts the tweens on unmount

  return <div ref={dot} className="pointer-events-none fixed left-0 top-0 size-4 rounded-full bg-black" />;
}
```

For an instant (un-eased) follower, swap in `quickSetter`:

```tsx
const setX = gsap.quickSetter(el, "x", "px");
const setY = gsap.quickSetter(el, "y", "px");
const onMove = (e: PointerEvent) => { setX(e.clientX); setY(e.clientY); };
```

See [React & Next.js](./react-nextjs.md) for the `useGSAP` cleanup pattern.

## ScrollTrigger performance

Scroll runs on the main thread, so scroll-linked work is the most jank-prone surface. Keep it lean:

- **Pin only what's needed.** `pin: true` promotes the pinned element and adds spacing/measurement overhead. Pin the smallest necessary subtree, not whole sections.
- **Smooth scrub on low-end.** Use a numeric `scrub` (e.g. `scrub: 1`) so the timeline catches up over ~1s instead of recomputing on every scroll tick. Test the value on a representative device — too small reintroduces jank.
- **Refresh only on real layout change.** Call `ScrollTrigger.refresh()` after content loads or the DOM reflows (images, fonts, async data) — not on every `resize`. Debounce resize-driven refreshes and order them after the layout settles, not on a random timeout.
- **Order with `refreshPriority`.** When triggers depend on each other's positions (e.g. a pinned scene above content that also has triggers), set `refreshPriority` so they recalculate in the right order; higher priority refreshes first.
- **Keep callbacks cheap.** `onUpdate`/`onToggle` run during scroll — no layout reads, no allocations, no heavy work per frame.

```tsx
ScrollTrigger.create({
  trigger: ".panel",
  start: "top top",
  end: "+=1200",
  pin: true,            // promote only this element
  scrub: 1,             // smoothing — gentler on low-end devices
  refreshPriority: 1,   // refresh before lower-priority triggers
});

// After async content changes the layout — debounced, not on raw resize
ScrollTrigger.refresh();
```

See [ScrollTrigger](./scrolltrigger.md) for scene semantics.

## Kill or pause off-screen animations

Animations that aren't visible still consume frame budget. Stop them:

- **Pause** off-screen or background animations (tab hidden, element scrolled away) with an IntersectionObserver or `ScrollTrigger`'s `onToggle`, and resume when visible.
- **Kill** tweens/ScrollTriggers on route change and component unmount so stray instances don't keep ticking. In React, `useGSAP` reverts everything created in its scope automatically.
- Infinite/looping tweens (`repeat: -1`) are the worst offenders when forgotten — they run forever. Always pair them with visibility-gating and cleanup.

```tsx
const io = new IntersectionObserver(([entry]) => {
  entry.isIntersecting ? loopTween.play() : loopTween.pause();
});
io.observe(el);
```

## Reduced motion as a perf + a11y lever

`prefers-reduced-motion` is both an accessibility requirement and a free performance win — honoring it removes the most expensive looping/parallax work for the users who opt out. Treat it as a behavioral branch, not just a shorter duration: skip decorative motion entirely, keep functional feedback, and land elements on a sensible static final state.

```tsx
const mm = gsap.matchMedia();
mm.add("(prefers-reduced-motion: reduce)", () => {
  gsap.set(".hero", { autoAlpha: 1, y: 0 }); // final state, no movement, no loops
});
mm.add("(prefers-reduced-motion: no-preference)", () => {
  gsap.from(".hero", { y: 60, autoAlpha: 0, duration: 1 });
  // pinned / parallax / infinite tweens go here so they never run for reduced-motion users
});
```

`matchMedia` auto-reverts everything created in a branch when its query stops matching. See [Core API](./core.md).

## Profiling playbook

Diagnose before you optimize — replacing code without finding the bottleneck wastes effort.

1. **Reproduce with evidence.** Note the affected route, device/browser, and the exact interaction that janks.
2. **Record in DevTools Performance.** Open the Performance panel, throttle CPU (4–6× slowdown) to mimic a mid-range device, record while triggering the animation, and stop.
3. **Read the flame chart.** Look for **long tasks** (>50ms, red-flagged), and recurring **Layout** / **Recalculate Style** entries during the animation — those signal layout thrash from animating layout props or reading geometry mid-frame.
4. **Watch the FPS / frame meter.** Enable the FPS meter (Rendering tab → Frame Rendering Stats). Dropped frames or a frame chart that can't hold the green line means you're over budget.
5. **Classify the cost** as composite, paint, layout, JavaScript, image/decode, or scroll workload (see the [property-cost table](#the-property-cost-mental-model)) before deciding the fix.
6. **Check layers.** Use the Layers panel / "Paint flashing" and "Layer borders" in the Rendering tab to confirm `will-change` promoted the right elements and didn't create an oversized layer.
7. **Capture before/after** for the hot interaction so the fix is provably better, ideally under throttled CPU.

Common culprits and fixes: layout props → swap to transforms; per-frame `getBoundingClientRect` → batch/cache reads; blanket `will-change` → scope and remove it; un-debounced `ScrollTrigger.refresh()` → call only on real layout change.

## Pitfalls / Do-not

- **Do not** animate layout props (`width`, `height`, `top`, `left`, `margin`, `padding`) when transform aliases (`x`, `y`, `scale`) achieve the same look — they force reflow every frame.
- **Do not** put `will-change` (or `force3D`/`translateZ(0)`) on everything "just in case" — scope it to actively animating elements and remove it after; global will-change hurts more than it helps.
- **Do not** read layout (`offsetWidth`, `getBoundingClientRect`) inside `onUpdate` or scroll callbacks — that thrashes layout on the hot path. Read once, cache, recompute on refresh.
- **Do not** create hundreds of overlapping tweens or ScrollTriggers — use `stagger`, a single timeline, or virtualization, and test on low-end devices.
- **Do not** create a new tween on every mouse/scroll event — reuse one `gsap.quickTo()` / `gsap.quickSetter()`.
- **Do not** forget cleanup — stray tweens, infinite loops, and ScrollTriggers keep running after unmount/navigation, costing frames and causing bugs.
- **Do not** ship infinite tweens (`repeat: -1`) without a reduced-motion branch and visibility-gated pause/kill.

## Related references

- [Core API](./core.md)
- [React & Next.js](./react-nextjs.md)
- [ScrollTrigger](./scrolltrigger.md)
- [Utilities](./utils.md)
- [Recipes](./recipes.md)
