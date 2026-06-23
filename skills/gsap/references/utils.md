# GSAP Utilities (gsap.utils)

`gsap.utils` is a set of pure helper functions for math, value mapping, randomness, snapping, unit parsing, and collection handling. They ship with the core `gsap` package (v3.13+, fully free and open) and need **no plugin registration** — call any helper as `gsap.utils.clamp()`, etc.

## Contents

- [Two patterns to internalize](#two-patterns-to-internalize)
- [Clamping and ranges](#clamping-and-ranges) — `clamp`, `mapRange`, `normalize`, `interpolate`
- [Random and snap](#random-and-snap) — `random`, `snap`, `shuffle`, `distribute`
- [Units and parsing](#units-and-parsing) — `getUnit`, `unitize`, `splitColor`
- [Collections and composition](#collections-and-composition) — `selector`, `toArray`, `pipe`
- [Cyclic values](#cyclic-values) — `wrap`, `wrapYoyo`
- [React / ScrollTrigger uses](#react--scrolltrigger-uses)
- [Pitfalls / Do-not](#pitfalls--do-not)
- [Related references](#related-references)

## Two patterns to internalize

**1. Omit the final value to get a reusable function.** Most helpers take the value to transform as the **last** argument. Omit it and you get back a function that accepts the value later. Use this whenever the same config maps many values (pointer/scroll handlers, tween callbacks, render loops) — build the function once, call it many times.

```ts
// With value → returns the result
gsap.utils.clamp(0, 100, 150); // 100

// Without value → returns a reusable function
const clamp = gsap.utils.clamp(0, 100);
clamp(150); // 100
clamp(-10); // 0
```

> **`random()` is the exception.** Do not omit the value — pass `true` as the last argument to get the reusable function form.

**2. Scope `selector` / `toArray` inside components.** An unscoped `".box"` matches every `.box` in the document and leaks into siblings. Pass the component's root element or ref so selectors resolve only within that subtree: `gsap.utils.selector(containerRef)` and `gsap.utils.toArray(".box", containerRef)`.

## Clamping and ranges

> These helpers are **numeric** — they operate on plain numbers, not unit strings like `"100px"`. For units, parse with [`getUnit`](#getunitvalue) and re-attach with [`unitize`](#unitizefn-unit).

### clamp(min, max, value?)

Constrains `value` to the `[min, max]` range.

```ts
gsap.utils.clamp(0, 100, 150); // 100
gsap.utils.clamp(0, 100, -10); // 0
const clamp = gsap.utils.clamp(0, 100); // function form
clamp(150); // 100
```

### mapRange(inMin, inMax, outMin, outMax, value?)

Remaps `value` from one range to another. The workhorse for converting scroll position, `progress` (0–1), or any input range to an animation range.

```ts
gsap.utils.mapRange(0, 100, 0, 500, 50); // 250
gsap.utils.mapRange(0, 1, 0, 360, 0.5); // 180 (progress → degrees)
const toDegrees = gsap.utils.mapRange(0, 1, 0, 360); // function form
toDegrees(0.5); // 180
```

### normalize(min, max, value?)

Returns `value` normalized to `0–1` for the range (equivalent to `mapRange(min, max, 0, 1)`).

```ts
gsap.utils.normalize(0, 100, 50); // 0.5
gsap.utils.normalize(100, 300, 200); // 0.5
const norm = gsap.utils.normalize(0, 100); // function form
norm(50); // 0.5
```

### interpolate(start, end, progress?)

Interpolates between two values at `progress` (0–1). Handles **numbers**, **color strings**, **objects** (matching keys), and **arrays** (element-wise). Omit `progress` for the function form.

```ts
gsap.utils.interpolate(0, 100, 0.5); // 50
gsap.utils.interpolate("#ff0000", "#0000ff", 0.5); // mid color
gsap.utils.interpolate({ x: 0, y: 0 }, { x: 100, y: 50 }, 0.5); // { x: 50, y: 25 }
gsap.utils.interpolate([0, 10], [100, 20], 0.5); // [50, 15]
const lerp = gsap.utils.interpolate(0, 100); // function form
lerp(0.25); // 25
```

Numbers/objects/arrays are interpolated numerically; only **color** inputs parse their own string format.

## Random and snap

### random(minimum, maximum[, snapIncrement, returnFunction]) / random(array[, returnFunction])

Returns a random number in `[minimum, maximum]`, or a random element from an `array`. Optional `snapIncrement` snaps to the nearest multiple. For the **reusable function** form, pass `true` as the last argument (`returnFunction`) — it takes no args and yields a new random value each call.

```ts
gsap.utils.random(-100, 100); // e.g. 42.7
gsap.utils.random(0, 500, 5); // 0–500, snapped to nearest 5
gsap.utils.random(["red", "blue", "green"]); // one element at random
const pick = gsap.utils.random(-200, 500, 10, true); // function form
pick(); // random in range, snapped to 10 (new value each call)
```

**String form in tween vars** — GSAP evaluates it per target:

```ts
gsap.to(".box", { x: "random(-100, 100, 5)", duration: 1 });
gsap.to(".item", { backgroundColor: "random([red, blue, green])" });
```

### snap(snapTo, value?)

Snaps `value` to the nearest multiple of `snapTo`, the nearest entry in an **array** of allowed values, or — with an **object config** — the nearest value within a `radius` (else returns the original).

```ts
gsap.utils.snap(10, 23); // 20
gsap.utils.snap(0.25, 0.7); // 0.75
gsap.utils.snap([0, 100, 200], 150); // 200 (nearest in array)

// Object form: snap only within radius, otherwise pass through
const snap = gsap.utils.snap({ values: [0, 100, 200], radius: 20 });
snap(95); // 100 (within 20)
snap(150); // 150 (nothing within 20 → unchanged)
```

Use the `snap` tween property for grid/step animation: `gsap.to(".x", { x: 200, snap: { x: 20 } })`.

### shuffle(array)

**Mutates** and returns the same array in random order. Shuffle a **copy** when the source is state, props, or shared data.

```ts
const shuffled = gsap.utils.shuffle([...items]); // copy first
```

### distribute(config)

**Returns a function** that assigns a value to each target by its position in the array or grid. Pass the result straight into a tween (GSAP calls it per target with `(index, target, targets)`) or call it manually. Config (all optional):

- `base` (number) — starting value, default `0`.
- `amount` (number) — total spread across all targets (added to base); `amount: 1` over 100 → `0.01` step.
- `each` (number) — fixed step between targets (added to base); `each: 1` over 4 → `0, 1, 2, 3`. Use instead of `amount`.
- `from` (number | string | number[]) — origin: index, `"start"`, `"center"`, `"edges"`, `"random"`, `"end"`, or ratios like `[0.25, 0.75]`. Default `0`.
- `grid` (string | number[]) — grid position instead of flat index: `[rows, cols]` or `"auto"`. `axis` (`"x"` | `"y"`) restricts to one grid axis.
- `ease` (Ease) — distribute along an ease curve, e.g. `"power1.inOut"`. Default `"none"`.

```ts
// Scale: edges ~3, center 0.5 (2.5 distributed from center)
gsap.to(".class", { scale: gsap.utils.distribute({ base: 0.5, amount: 2.5, from: "center" }) });

// Manual use
const dist = gsap.utils.distribute({ base: 50, amount: 100, from: "center", ease: "power1.inOut" });
const targets = gsap.utils.toArray<HTMLElement>(".box");
const v = dist(2, targets[2], targets);
```

## Units and parsing

### getUnit(value)

Returns the unit string of a value (`"px"`, `"%"`, `"deg"`, …), or `""` for unitless.

```ts
gsap.utils.getUnit("100px"); // "px"
gsap.utils.getUnit(42); // "" (unitless)
```

### unitize(fn, unit)

Wraps a numeric function so its **return value** gets `unit` appended — the bridge between numeric helpers and CSS/tween end values. Keep unit conversion at the edge.

```ts
const wrapX = gsap.utils.unitize(gsap.utils.wrap(0, 300), "px");
wrapX(450); // "150px"
wrapX(-50); // "250px"
```

### splitColor(color, returnHSL?)

Converts any color string into `[r, g, b]` (0–255), or `[r, g, b, a]` when alpha is present. Pass `true` for `returnHSL` to get `[h, s, l]`/`[h, s, l, a]`. Accepts `rgb()`/`rgba()`/`hsl()`/`hsla()`, hex, and named colors.

```ts
gsap.utils.splitColor("#6fb936"); // [111, 185, 54]
gsap.utils.splitColor("rgba(204, 153, 51, 0.5)"); // [204, 153, 51, 0.5]
gsap.utils.splitColor("#6fb936", true); // [94, 55, 47] (HSL)
```

## Collections and composition

### selector(scope)

Returns a scoped query function that finds elements **only within** the given element or ref (it unwraps a React ref's `.current`). Use it in components so `".box"` never escapes the component root.

```ts
const q = gsap.utils.selector(containerRef);
gsap.to(q(".circle"), { x: 100 }); // only .circle inside containerRef
```

### toArray(value, scope?)

Coerces a selector string, NodeList, HTMLCollection, single element, or array into a real array. Pass `scope` to restrict a selector string to a root element.

```ts
gsap.utils.toArray(".item"); // Element[]
gsap.utils.toArray(".item", container); // scoped to container
```

### pipe(...functions)

Left-to-right function composition: `pipe(f1, f2, f3)(v)` === `f3(f2(f1(v)))`. Compose stable transforms (e.g. normalize → snap → unitize) once, reuse everywhere.

```ts
const transform = gsap.utils.pipe(
  gsap.utils.clamp(0, 100),
  gsap.utils.normalize(0, 100),
  gsap.utils.snap(0.1),
);
transform(73); // clamp → normalize → snap
```

Build composed functions **once** (outside frame loops / render bodies), not on every frame or render.

## Cyclic values

### wrap(min, max, value?)

Wraps `value` into `[min, max)` (inclusive min, exclusive max). For infinite scroll, carousels, and cyclic indexes.

```ts
gsap.utils.wrap(0, 360, 370); // 10
gsap.utils.wrap(0, 360, -10); // 350
const wrapHue = gsap.utils.wrap(0, 360); // function form
wrapHue(370); // 10
gsap.utils.wrap(["red", "green", "blue"], 4); // "green" (array form, cycle by index)
```

### wrapYoyo(min, max, value?)

Like `wrap`, but bounces at the ends instead of jumping — for back-and-forth motion.

```ts
gsap.utils.wrapYoyo(0, 100, 150); // 50 (bounced back)
const ping = gsap.utils.wrapYoyo(0, 100); // function form
ping(150); // 50
```

Test cyclic helpers at first, last, overflow, and **negative** inputs — they are often only correct because their input domain is constrained upstream.

## React / ScrollTrigger uses

**Map scroll progress with `pipe(clamp, mapRange)`** — build the functions once, scope selectors to the component root:

```tsx
"use client";
import { useRef } from "react";
import gsap from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { useGSAP } from "@gsap/react";

gsap.registerPlugin(ScrollTrigger, useGSAP);

export function ParallaxBadge() {
  const root = useRef<HTMLDivElement>(null);

  useGSAP(() => {
    const q = gsap.utils.selector(root); // scoped to this component
    const toRotation = gsap.utils.pipe(
      gsap.utils.clamp(0, 1),
      gsap.utils.mapRange(0, 1, 0, 90), // progress 0–1 → 0–90deg
    );
    ScrollTrigger.create({
      trigger: root.current,
      scrub: true,
      onUpdate: (self) => gsap.set(q(".badge"), { rotation: toRotation(self.progress) }),
    });
  }, { scope: root });

  return <div ref={root}><span className="badge">★</span></div>;
}
```

**Snap a dragged/scrolled value to a grid**, returning a CSS string via `unitize`:

```ts
const snapToGrid = gsap.utils.unitize(gsap.utils.snap(20), "px");
snapToGrid(53); // "60px"
```


## Pitfalls / Do-not

- **Do not** assume numeric helpers handle units. `mapRange`, `normalize`, and `interpolate` (for numbers/objects/arrays) work on plain numbers — feeding `"100px"` will not parse the unit. Use [`getUnit`](#getunitvalue) / [`unitize`](#unitizefn-unit) to manage units at the edge.
- **Do not** use unscoped `selector` / `toArray` selectors inside components — `".box"` matches the whole document and leaks across components. Always pass the component root or ref.
- **Do not** create helper functions inside frame loops or render bodies. Build the reusable function once (omit the final value) and call it per frame.
- **Do not** call `shuffle` on state, props, or shared arrays — it mutates in place; shuffle a copy.
- **Do not** put `random` output into SSR markup or hydration-sensitive initial styles (it differs between server and client). Use deterministic values for SSR and for snapshot/visual tests.
- **Do not** rely on undocumented behavior or invented options — stick to the documented API.

## Related references

- [Core API](./core.md)
- [React & Next.js](./react-nextjs.md)
- [ScrollTrigger](./scrolltrigger.md)
- [Performance](./performance.md)
- [Plugins](./plugins.md)
