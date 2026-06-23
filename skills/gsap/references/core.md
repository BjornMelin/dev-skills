# GSAP Core API

The GSAP core engine (`gsap.to/from/fromTo/set`, transforms, easing, stagger, defaults, and `matchMedia`) animates anything with a numeric property — DOM, SVG, CSS variables, JS objects. GSAP is **100% free and open** since the Webflow acquisition (v3.13+): every former "Club GreenSock" plugin now ships in the public `gsap` npm package, with no membership, trial package, or auth token required.

## Contents

- [Install](#install)
- [Tween methods](#tween-methods)
- [Common vars](#common-vars)
- [Transforms and CSS properties](#transforms-and-css-properties)
- [Targets](#targets)
- [Stagger](#stagger)
- [Easing](#easing)
- [Controlling tweens](#controlling-tweens)
- [Function-based values](#function-based-values)
- [Relative values](#relative-values)
- [Defaults](#defaults)
- [Responsive and reduced motion (matchMedia)](#responsive-and-reduced-motion-matchmedia)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## Install

```bash
npm install gsap
```

All plugins are in the same package — import them from `gsap` (or `gsap/<Plugin>`) and register them. See [Plugins](./plugins.md).

```tsx
import gsap from "gsap";
```

## Tween methods

A **tween** animates one or more targets to a set of values. All four return a `Tween` instance.

- **`gsap.to(targets, vars)`** — animate from the current state to `vars`. Most common.
- **`gsap.from(targets, vars)`** — animate from `vars` to the current state (good for entrances).
- **`gsap.fromTo(targets, fromVars, toVars)`** — explicit start and end; never reads current values.
- **`gsap.set(targets, vars)`** — apply immediately (duration 0, no animation).

Always use **camelCase** property names in the vars object (`backgroundColor`, `marginTop`, `rotationX`, `scaleY`).

```tsx
gsap.to(".box", { x: 200, rotation: 45, duration: 1, ease: "power2.out" });
gsap.from(".card", { y: 40, autoAlpha: 0, duration: 0.6 }); // entrance
gsap.fromTo(".bar", { scaleX: 0 }, { scaleX: 1, transformOrigin: "left center", duration: 0.8 });
gsap.set(".panel", { xPercent: -100 }); // instant, no tween
```

## Common vars

| Var | Meaning |
|-----|---------|
| `duration` | Seconds (default `0.5`). |
| `delay` | Seconds before start. |
| `ease` | String or function. Default `"power1.out"`. Use `"none"` for linear. |
| `stagger` | Number (seconds between each) or object (see [Stagger](#stagger)). |
| `repeat` | Number of repeats, or `-1` for infinite. |
| `yoyo` | Boolean; with `repeat`, alternates direction each cycle. |
| `repeatDelay` | Seconds to wait between repeats. |
| `overwrite` | `false` (default), `true`, or `"auto"` (see below). |
| `paused` | `true` creates the tween in a paused state. |
| `immediateRender` | When to apply the start state (see below). |
| `onStart` / `onUpdate` / `onComplete` / `onRepeat` / `onReverseComplete` | Callbacks scoped to the Tween/Timeline instance. Pass args via `onCompleteParams: [...]`. |

**`overwrite`** controls collision behavior between tweens of the same targets:

- `false` (default) — no automatic killing; tweens coexist.
- `true` — immediately kill all active tweens of the same targets.
- `"auto"` — when the tween first renders, kill only the individually overlapping properties in other **active** tweens of the same targets. Useful for rapid, interruptible interactions, but not a substitute for a single clear animation owner.

**`immediateRender`** — When `true` (the default for `from()` and `fromTo()`), the start state is applied the moment the tween is created, which prevents a flash of unstyled content. **When stacking multiple `from()`/`fromTo()` tweens on the same property of the same element, set `immediateRender: false` on the later one(s)** so the first tween's end state is not overwritten before it runs.

## Transforms and CSS properties

GSAP's CSSPlugin (built into core) animates DOM elements. Prefer GSAP's **transform aliases** over the raw `transform` string: they apply in a consistent order (translation → scale → rotationX/Y → skew → rotation), are more performant, and work reliably across browsers and SVG.

| GSAP property | Equivalent / note |
|---------------|-------------------|
| `x`, `y`, `z` | `translateX/Y/Z` (default unit: px) |
| `xPercent`, `yPercent` | translate in %; use for percentage-based movement; work on SVG |
| `scale`, `scaleX`, `scaleY` | `scale`; `scale` sets both axes |
| `rotation` | `rotate` (default deg; or `"1.25rad"`); equals `rotationZ` |
| `rotationX`, `rotationY` | 3D rotate |
| `skewX`, `skewY` | `skew` (deg or rad string) |
| `transformOrigin` | `transform-origin` (e.g. `"left top"`, `"50% 50%"`) |

Default units: x/y/z in px, rotation/skew in deg. Relative values work here too: `x: "+=20"`, `rotation: "-=30"`.

### autoAlpha vs opacity

Prefer **`autoAlpha`** over `opacity` for fade in/out. When the value reaches `0`, GSAP also sets `visibility: hidden` (no rendering cost, no pointer events); when non-zero, it sets `visibility: inherit`. This avoids leaving invisible elements that still block clicks.

```tsx
gsap.to(".fade", { autoAlpha: 0, duration: 0.5 }); // fully hidden + non-interactive at 0
```

### Animating CSS variables

GSAP animates CSS custom properties in any browser that supports them.

```tsx
gsap.to(":root", { "--hue": 180, "--size": 100, duration: 1 });
```

### svgOrigin (SVG only)

Like `transformOrigin` but expressed in the SVG's **global** coordinate space (e.g. `svgOrigin: "250 100"`). Use when several SVG elements should rotate or scale around a common point. No percentage values; units optional. Only one of `svgOrigin` or `transformOrigin` applies per element.

```js
gsap.to(svgEl, { rotation: 90, svgOrigin: "100 100" });
```

### Directional rotation suffixes

Append a suffix to a rotation **string** value to control the path: `_short` (shortest path), `_cw` (clockwise), `_ccw` (counter-clockwise). Applies to `rotation`, `rotationX`, `rotationY`.

```js
gsap.to(".box", { rotation: "-170_short" }); // shortest path to the -170 orientation (170° CCW, not 190° CW)
gsap.to(".dial", { rotationX: "+=30_cw" });
gsap.to(".box", { rotation: "360_cw", duration: 1 });
```

### clearProps

A comma-separated list of property names (or `"all"` / `true`) to **remove** from the element's inline style when the tween completes — useful when a CSS class should take over afterward. Clearing any transform-related property (`x`, `scale`, `rotation`, …) clears the **entire** transform.

```tsx
gsap.to(".box", { x: 100, duration: 1, clearProps: "transform" });
```

## Targets

A target can be a CSS selector string, a DOM element or ref, an array, a NodeList, or a plain JS object (animate any numeric property). Multiple targets animate together unless you add a `stagger`.

```tsx
const obj = { count: 0 };
gsap.to(obj, { count: 100, duration: 2, onUpdate: () => render(obj.count) });
```

## Stagger

Offset the start of each target. A number is the seconds between each:

```tsx
gsap.to(".item", { y: -20, stagger: 0.1 });
```

Object syntax gives full control. `from` decides where the stagger originates: `"start"` (default), `"center"`, `"end"`, `"edges"`, `"random"`, or an index.

```tsx
gsap.to(".item", {
  opacity: 1,
  y: 0,
  stagger: {
    amount: 0.3,   // total time spread across ALL targets (use `each` for per-item time instead)
    from: "center",
    grid: "auto",  // for 2D grids; lets `from` work spatially
    ease: "power1.in",
  },
});
```

Use `each` (per-item delay) **or** `amount` (total spread), not both.

## Easing

Use string eases unless a custom curve is needed. The base name equals `.out`; suffixes are `.in`, `.out`, `.inOut`. The power number is curve strength (1 = gradual, 4 = steepest).

| Family | `.in` / `.out` / `.inOut` | Notes |
|--------|---------------------------|-------|
| `none` | — | linear |
| `power1`–`power4` | yes | general-purpose; `power1.out` is the default |
| `sine` | yes | gentle |
| `expo` | yes | sharp |
| `circ` | yes | arc-like |
| `back` | yes | overshoot, e.g. `"back.out(1.7)"` |
| `elastic` | yes | springy, e.g. `"elastic.out(1, 0.3)"` |
| `bounce` | yes | settling bounce |

```tsx
ease: "power3.inOut"
ease: "back.out(1.7)"      // overshoot
ease: "elastic.out(1, 0.3)"
ease: "none"               // linear
```

### CustomEase

For bespoke curves, register `CustomEase` (ships in the `gsap` package). See [Plugins](./plugins.md).

```tsx
import { CustomEase } from "gsap/CustomEase";
gsap.registerPlugin(CustomEase);

// cubic-bezier control points, as in CSS cubic-bezier()
const easeA = CustomEase.create("easeA", ".17,.67,.83,.67");
// or a full normalized SVG path for multi-segment curves
const hop = CustomEase.create("hop", "M0,0 C0,0 0.056,0.442 0.175,0.442 0.294,0.442 0.332,0 0.332,0 0.332,0 0.414,1 0.671,1 0.991,1 1,0 1,0");

gsap.to(".item", { x: 100, ease: easeA, duration: 1 });
```

## Controlling tweens

Store the returned `Tween` when you need playback control, inspection, or cleanup (React unmount, route change, rapid re-triggers).

```tsx
const tween = gsap.to(".box", { x: 100, duration: 1, repeat: 1, yoyo: true });
tween.pause();
tween.play();
tween.reverse();
tween.progress(0.5);   // jump to 50%
tween.time(0.2);       // jump to 0.2s
tween.timeScale(2);    // 2x speed
tween.kill();          // stop and free
```

In React, prefer `useGSAP` for automatic cleanup — see [React & Next.js](./react-nextjs.md).

## Function-based values

Pass a function for any vars value; it runs **once per target** on first render. The return value is used as that target's animation value.

```tsx
gsap.to(".item", {
  x: (i, target, targets) => i * 50, // 0, 50, 100, ...
  stagger: 0.1,
});
```

## Relative values

Prefix a value with `+=`, `-=`, `*=`, or `/=` to compute it relative to the current value at first render.

```tsx
gsap.to(".class", { x: "+=20", rotation: "-=30" }); // add 20px, subtract 30deg
// "*=2" doubles, "/=2" halves the current value
```

## Defaults

Set project-wide tween defaults once:

```tsx
gsap.defaults({ duration: 0.6, ease: "power2.out" });
```

Timelines accept per-timeline defaults too — see [Timeline](./timeline.md).

## Responsive and reduced motion (matchMedia)

`gsap.matchMedia()` runs setup only while a media query matches; when it stops matching, every animation and ScrollTrigger created in that callback is **reverted automatically**. Use it for breakpoints **and** `prefers-reduced-motion`. Treat reduced motion as a behavioral branch, not just a shorter duration: skip decorative movement entirely while keeping functional feedback visible, and ensure off-screen entrances still land on a sensible static final state.

```tsx
const mm = gsap.matchMedia();

mm.add(
  {
    isDesktop: "(min-width: 800px)",
    isMobile: "(max-width: 799px)",
    reduceMotion: "(prefers-reduced-motion: reduce)",
  },
  (ctx) => {
    const { isDesktop, reduceMotion } = ctx.conditions as {
      isDesktop: boolean;
      isMobile: boolean;
      reduceMotion: boolean;
    };

    if (reduceMotion) {
      gsap.set(".box", { autoAlpha: 1, y: 0 }); // static final state, no motion
      return;
    }

    gsap.from(".box", { y: 40, autoAlpha: 0, duration: isDesktop ? 1 : 0.5 });

    return () => { /* optional custom cleanup */ };
  },
  containerRef // optional 3rd arg: scope selector text to a root element/ref
);

// mm.revert();             // tear down all (e.g. on unmount)
// gsap.matchMediaRefresh(); // re-run matching handlers after toggling a motion control
```

Do **not** nest `gsap.context()` inside `matchMedia` — `matchMedia` creates a context internally; use `mm.revert()` for cleanup.

## Pitfalls / Do-not

- **Do not** animate layout-heavy properties (`width`, `height`, `top`, `left`) when transform aliases (`x`, `y`, `scale`, `rotation`) achieve the same effect — transforms are far cheaper. See [Performance](./performance.md).
- **Do not** animate the raw `transform` string when an alias expresses the same effect; aliases compose in a consistent, performant order.
- **Do not** rely on the default `immediateRender: true` when stacking multiple `from()`/`fromTo()` tweens on the same property of the same target — set `immediateRender: false` on the later ones.
- **Do not** set both `svgOrigin` and `transformOrigin` on the same SVG element; only one applies.
- **Do not** use invented ease names — stick to documented eases (or `CustomEase`).
- **Do not** forget that `gsap.from()` uses the element's current state as the **end** state and applies the start values immediately (unless `immediateRender: false`).
- **Do not** fire-and-forget tweens in event handlers that need interruption — store the handle and `kill()`/reverse on the next interaction or on cleanup.
- **Do not** chain animations with `delay` when a [Timeline](./timeline.md) would sequence them more reliably.

## Related references

- [React & Next.js](./react-nextjs.md)
- [Timeline](./timeline.md)
- [ScrollTrigger](./scrolltrigger.md)
- [Plugins](./plugins.md)
- [Performance](./performance.md)
- [Utilities](./utils.md)
