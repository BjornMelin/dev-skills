# Tier Reference

Detailed classification guide for the motion-audit skill. Read this when classifying animations.

## Render Pipeline

Every frame, the browser may run up to three sequential steps — each triggers all subsequent steps:

1. **Layout** — calculate geometry (size, position, flex/grid)
2. **Paint** — draw pixels into layers (colors, shadows, gradients)
3. **Composite** — merge layers, apply transforms/opacity

Animating a layout property triggers all three. A paint property triggers paint + composite. A compositor property triggers only composite.

Before any rendering: **Style Recalculation** runs on the main thread. Cost scales with DOM complexity and selector specificity.

---

## S-Tier: Compositor Thread

Animations that run entirely on the compositor thread. The main thread is not involved per-frame, so these remain smooth (60-120fps) even under heavy JS load.

### Qualifying Properties

| Property    | Notes                                                    |
| ----------- | -------------------------------------------------------- |
| `transform` | translate, rotate, scale, skew, matrix, perspective      |
| `opacity`   |                                                          |
| `filter`    | blur, brightness, contrast, drop-shadow, grayscale, etc. |
| `clip-path` |                                                          |

### Qualifying Methods

The property alone isn't enough — the animation method must also support compositor offloading:

-   **CSS `transition`** — S-tier
-   **CSS `@keyframes` / `animation`** — S-tier
-   **Web Animations API** (`element.animate()`) — S-tier
-   **Motion `animate()` and `<motion.div />`** with compositor props — S-tier (uses WAAPI internally)
-   **`scroll-timeline` / `view-timeline`** with compositor props — S-tier
-   **Motion `scroll() and `useScroll` (also via`useTransform`)** with compositor targets — S-tier

### Caveats

-   **Layer size matters**: Each composited element gets its own GPU layer. Large elements (full-viewport, tickers with cloned children) consume significant GPU memory, especially on mobile.
-   **`filter: blur()` cost escalates**: Blur radius >10px on large layers can cause visible jank despite being compositor-only. The GPU work is proportional to blur radius × layer area.
-   **Safari deoptimizations**: Safari sometimes falls back to main-thread rendering for `clip-path` animations and complex `filter` chains. Test on WebKit.
-   **Layer promotion**: The element must be promoted to its own layer. CSS/WAAPI animations auto-promote. JS-driven animations may not — see A-Tier.

### IntersectionObserver-Triggered Animations

Using IntersectionObserver (or Motion's `inView`/`useInView`) to trigger an S/A-tier animation. The observer callback is a one-time setup cost:

```javascript
// Triggers S-tier animation on intersection
inView(element, () => {
    animate(element, { x: -100, opacity: 1 })
})
```

```jsx
// Motion whileInView (auto-deactivates off-screen)
<motion.div whileInView={{ opacity: 1 }} />
```

### Detection Patterns

```css
/* S-tier — CSS transition on compositor property */
.box {
    transition: transform 0.3s ease, opacity 0.3s ease;
}

/* S-tier — CSS keyframes */
@keyframes fadeIn {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

/* S-tier — scroll-driven */
.box {
    animation-timeline: scroll();
}
```

```javascript
// S-tier — WAAPI
element.animate(
    { transform: ["translateX(0)", "translateX(100px)"] },
    { duration: 300 }
)

// S-tier — Motion
animate(".box", { opacity: 1 })
animate(element, { transform: "translateX(100px)" })
scroll(animate(element, { opacity: [0, 1] }))
```

---

## A-Tier: Main Thread → Compositor

JavaScript sets compositor values (`transform`, `opacity`) via the main thread. The actual visual change is composited (cheap), but the JS execution is interruptible — if the main thread is blocked, the animation janks.

### When It Applies

-   `element.style.transform = ...` inside `requestAnimationFrame`
-   GSAP `.to()` / `.from()` animating compositor properties
-   Any JS library using rAF (not WAAPI) to drive compositor values
-   react-spring, anime.js driving `transform`/`opacity`

### Layer Promotion

For A-tier to work, the element must already be on its own compositor layer. Auto-promotion triggers:

-   Active CSS/WAAPI animation
-   `transform: translateZ(0)` or any 3D transform
-   `position: fixed` or `position: sticky`
-   `backdrop-filter`
-   Overlapping another promoted layer

If none apply, use `will-change` to hint promotion:

```css
.dialog {
    will-change: transform, opacity;
}
```

**Use `will-change` sparingly.** Each promoted layer consumes GPU memory. Don't apply to more than ~10 elements simultaneously. Remove after animation completes if possible.

### Detection Patterns

```javascript
// A-tier — rAF loop setting compositor value
function tick() {
    element.style.transform = `translateX(${x}px)`
    requestAnimationFrame(tick)
}

// A-tier - Motion
animate(element, { x: 100 }) // Uses independent transform value instead of transform

// A-tier — GSAP
gsap.to(".box", { x: 100, opacity: 1, duration: 0.5 })

// A-tier — react-spring
const styles = useSpring({ transform: "translateX(100px)" })

// A-tier — anime.js
anime({ targets: ".box", translateX: 100 })
```

---

## B-Tier: DOM Setup + S/A-Tier Animation

A one-time DOM measurement phase followed by an S or A-tier animation. The upfront layout read is expensive but happens only once — the per-frame animation cost is S or A-tier.

### FLIP Technique

1. **First**: Record element's initial position/size
2. **Last**: Apply final state, record new position/size
3. **Invert**: Apply `transform` to make element appear at original position
4. **Play**: Animate `transform` to `none` (S-tier)

```javascript
// B-tier — manual FLIP
const first = element.getBoundingClientRect()
element.classList.add("expanded")
const last = element.getBoundingClientRect()
const dx = first.left - last.left
const dy = first.top - last.top
element.style.transform = `translate(${dx}px, ${dy}px)`
requestAnimationFrame(() => {
    element.style.transition = "transform 0.3s ease"
    element.style.transform = "none"
})
```

### Motion Layout Animations

Motion's `layout` and `layoutId` props implement FLIP automatically:

```jsx
// B-tier — Motion layout
<motion.div layout />
<motion.div layoutId="shared-element" />
```

Per-frame, Motion calculates inverse transforms and corrective `border-radius` — this is main-thread work but only involves compositor property writes.

---

## C-Tier: Paint Trigger

Animations that force the browser to repaint affected layers every frame. No layout recalculation, but pixel rendering is repeated.

### Paint Properties

| Property                | Notes                                   |
| ----------------------- | --------------------------------------- |
| `background-color`      |                                         |
| `color`                 |                                         |
| `border-color`          |                                         |
| `border-radius`         | Repaint to redraw corners               |
| `box-shadow`            | Cost scales with blur radius and spread |
| `text-shadow`           |                                         |
| `outline-color`         |                                         |
| `mask-image`            |                                         |
| `background-image`      | When animating gradients                |
| `text-decoration-color` |                                         |
| `caret-color`           |                                         |
| `column-rule-color`     |                                         |

### CSS Variable Animations — Always C-Tier Minimum

CSS variables **always trigger paint**, even when used in compositor values:

```css
/* C-tier despite opacity being a compositor property */
div {
    --progress: 0;
    opacity: var(--progress);
}
```

The browser cannot statically determine what the variable affects, so it conservatively triggers paint.

**Fix with `@property` registration:**

```css
@property --progress {
    syntax: "<number>";
    inherits: false; /* Prevents inheritance cascade */
    initial-value: 0;
}
```

With `inherits: false`, the browser knows the variable doesn't cascade and can optimize the number of elements that must recalculate style on change. However, the animation itself is still paint-tier unless the browser can prove it only affects compositor properties.

### SVG Attribute Animations

Animating SVG shape attributes triggers repaint every frame:

| Attribute              | Element                     |
| ---------------------- | --------------------------- |
| `d`                    | `<path>`                    |
| `cx`, `cy`             | `<circle>`, `<ellipse>`     |
| `r`                    | `<circle>`                  |
| `rx`, `ry`             | `<ellipse>`, `<rect>`       |
| `x`, `y`               | `<rect>`, `<text>`, `<use>` |
| `x1`, `y1`, `x2`, `y2` | `<line>`                    |
| `points`               | `<polygon>`, `<polyline>`   |
| `viewBox`              | `<svg>`                     |
| `stroke-dashoffset`    | Any                         |
| `stroke-dasharray`     | Any                         |

**Upgrade**: For position/scale, use `transform` on the SVG element instead (S-tier).

### Detection Patterns

```css
/* C-tier — paint property transition */
.button {
    transition: background-color 0.2s ease;
}
.card:hover {
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}
```

```javascript
// C-tier — JS-driven paint property
element.style.backgroundColor = newColor

// C-tier — SVG path morphing
animate("path", { d: newPathData })
```

---

## D-Tier: Layout Trigger

Animations that force layout recalculation every frame. The browser must recalculate geometry for the animated element and potentially its siblings/ancestors, then repaint, then composite.

### Layout Properties

| Property                                             | Notes                                 |
| ---------------------------------------------------- | ------------------------------------- |
| `width`, `height`                                    |                                       |
| `min-width`, `max-width`, `min-height`, `max-height` |                                       |
| `margin`, `margin-*`                                 |                                       |
| `padding`, `padding-*`                               |                                       |
| `border-width`, `border-*-width`                     |                                       |
| `top`, `right`, `bottom`, `left`                     | On positioned elements                |
| `font-size`                                          | Affects text layout of entire subtree |
| `line-height`                                        |                                       |
| `display`                                            | Toggling triggers layout              |
| `flex`, `flex-grow`, `flex-shrink`, `flex-basis`     |                                       |
| `grid-template-columns`, `grid-template-rows`        |                                       |
| `grid-gap`, `gap`                                    |                                       |
| `justify-content`, `align-items`, `align-self`       |                                       |
| `position`                                           | Changing triggers layout              |
| `float`, `clear`                                     |                                       |
| `text-align`                                         |                                       |
| `vertical-align`                                     |                                       |
| `white-space`                                        |                                       |
| `word-spacing`, `letter-spacing`                     |                                       |
| `overflow`                                           |                                       |

### scrollTop/scrollLeft as Animation Driver

Reading `scrollTop` or `scrollLeft` in an animation loop is D-tier — it forces layout to get the current scroll position:

```javascript
// D-tier — scrollTop polling
window.addEventListener("scroll", () => {
    const progress = window.scrollY / maxScroll
    element.style.opacity = progress // Would be S-tier, but scrollTop read forces layout
})
```

**Upgrade**: Use `position: sticky`, `position: fixed`, CSS `scroll-timeline`, or Motion's `scroll()` which uses native ScrollTimeline (S-tier).

### Layout Containment

`contain: layout` (or `contain: strict`) limits the blast radius of layout recalculation:

```css
.animated-card {
    contain: layout; /* Layout changes don't affect siblings */
}
```

`position: absolute` or `position: fixed` also isolates layout — the element is removed from normal flow.

### View Transitions

View Transitions are mixed-tier:

-   **Size/position changes** (width, height, position) → D-tier for the measurement and animation phase
-   **Opacity** on `::view-transition-*` pseudo-elements → S-tier
-   **Transform** on `::view-transition-*` pseudo-elements → S-tier
-   Overall classification depends on what's changing

### Detection Patterns

```css
/* D-tier — layout property transition */
.card {
    transition: width 0.3s ease, height 0.3s ease;
}
.sidebar {
    transition: margin-left 0.3s ease;
}
```

```javascript
// D-tier — animating layout properties
gsap.to(".panel", { width: "100%", duration: 0.5 })
element.style.height = newHeight + "px" // in rAF loop
```

---

## F-Tier: Layout Thrashing

The worst pattern: interleaved DOM reads and writes that force **synchronous layout** multiple times per frame. Each read after a write forces the browser to recalculate layout immediately to return an accurate value.

### The Pattern

```javascript
// F-tier — read/write/read/write cycle
element.style.width = "100px" // Write (invalidates layout)
const width = element.offsetWidth // Read (forces synchronous layout!)
element.style.width = width * 2 + "px" // Write (invalidates again)
const height = element.offsetHeight // Read (forces layout AGAIN!)
```

### Layout-Triggering Reads

These DOM APIs force synchronous layout when called after a pending style change:

| API                                                      | Object  |
| -------------------------------------------------------- | ------- |
| `offsetWidth`, `offsetHeight`, `offsetTop`, `offsetLeft` | Element |
| `clientWidth`, `clientHeight`, `clientTop`, `clientLeft` | Element |
| `scrollWidth`, `scrollHeight`, `scrollTop`, `scrollLeft` | Element |
| `getBoundingClientRect()`                                | Element |
| `getComputedStyle()`                                     | Window  |
| `innerWidth`, `innerHeight`                              | Window  |
| `scrollX`, `scrollY`                                     | Window  |

### CSS Variable Inheritance Bombs

Animating a CSS variable on `:root`, `html`, or `body` forces style recalculation on the **entire DOM tree**, even if the variable is only used by a few elements:

```javascript
// F-tier — inheritance bomb
document.documentElement.style.setProperty("--progress", value)
// Forces style recalc on ALL elements every frame
```

Observed cost: **8ms per frame** on a page with ~1300 elements — consuming the entire 120fps frame budget.

**Fix:**

1. Scope the variable to the smallest possible subtree
2. Use `@property` with `inherits: false`
3. Animate the target property directly instead of via a variable

### React useLayoutEffect Thrashing

A common React pattern that causes thrashing:

```javascript
// F-tier — read in useLayoutEffect triggers synchronous layout
useLayoutEffect(() => {
    if (element.scrollWidth > element.clientWidth) {
        // Read
        element.dataset.overflowing = "yes" // Write
    }
}, [text])
```

**Fix**: Use Motion's `frame` API to batch reads and writes:

```javascript
frame.read(() => {
    const overflowing = element.scrollWidth > element.clientWidth
    frame.update(() => {
        element.dataset.overflowing = overflowing ? "yes" : ""
    })
})
```

### Detection Patterns

Look for:

1. Any loop or rAF callback that **reads** layout properties and **writes** style properties
2. `element.style.*` writes followed by `offset*`, `client*`, `scroll*`, `getBoundingClientRect()`, or `getComputedStyle()` reads
3. `document.documentElement.style.setProperty("--` or `document.body.style.setProperty("--` inside animation loops
4. Multiple components in React each doing independent `useLayoutEffect` DOM reads/writes

---

## Quick Lookup Table

### S-Tier (Compositor)

`transform` · `opacity` · `filter` · `clip-path`

### C-Tier (Paint)

`background-color` · `color` · `border-color` · `border-radius` · `box-shadow` · `text-shadow` · `outline-color` · `mask-image` · `background-image` · `text-decoration-color` · `caret-color` · `column-rule-color`

### D-Tier (Layout)

`width` · `height` · `min-width` · `max-width` · `min-height` · `max-height` · `margin` · `padding` · `border-width` · `top` · `right` · `bottom` · `left` · `font-size` · `line-height` · `display` · `flex` · `grid-template-*` · `gap` · `justify-content` · `align-items` · `position` · `float` · `text-align` · `vertical-align` · `white-space` · `word-spacing` · `letter-spacing` · `overflow`

### SVG (C-Tier Paint)

`d` · `cx` · `cy` · `r` · `rx` · `ry` · `x` · `y` · `x1` · `y1` · `x2` · `y2` · `points` · `viewBox` · `stroke-dashoffset` · `stroke-dasharray`
