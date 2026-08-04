# Performance

Transition specificity and GPU compositing hints.

## Transition Only What Changes

Never use `transition: all` or Tailwind's `transition-all`. Always specify the exact properties that change.

Tailwind's bare `transition` is **not** `all` — it maps to a curated list. In v4 that list is
`color`, `background-color`, `border-color`, `outline-color`, `text-decoration-color`, `fill`,
`stroke`, the gradient custom properties, `opacity`, `box-shadow`, `transform`, `translate`,
`scale`, `rotate`, `filter`, `backdrop-filter`, `display`, `content-visibility`, `overlay`, and
`pointer-events`. Note that `filter` **is** included, so bare `transition` already covers a
`blur()` change; and `display`/`content-visibility`/`overlay` are the entries that make
discrete transitions work with `@starting-style`. Still prefer naming exactly what changes —
the curated list is broad enough to animate properties you did not intend.

### Why

- `transition: all` forces the browser to watch every property for changes
- Causes unexpected transitions on properties you didn't intend to animate (colors, padding, shadows)
- Prevents browser optimizations

### CSS Example

```css
/* Good: only transition what changes */
.button {
  transition-property: scale, background-color;
  transition-duration: 150ms;
  transition-timing-function: ease-out;
}

/* Bad: transition everything */
.button {
  transition: all 150ms ease-out;
}
```

### Tailwind

```tsx
// Good: explicit properties
<button className="transition-[scale,background-color] duration-150 ease-out">

// Bad: transition all
<button className="transition-all duration-150 ease-out">
```

### Tailwind `transition-transform` Note

`transition-transform` in Tailwind maps to `transition-property: transform, translate, scale, rotate`, so it covers all transform-related properties, not just `transform`. Use this when you're only animating transforms. For multiple non-transform properties, use the bracket syntax: `transition-[scale,opacity,filter]`.

## Use `will-change` Sparingly

`will-change` hints the browser to pre-promote an element to its own GPU compositing layer. Without it, the browser promotes the element only when the animation starts; that one-time layer promotion can cause a micro-stutter on the first frame.

This particularly helps when an element is changing `scale`, `rotation`, or moving around with `transform`. For other properties, it doesn't help much: the browser can't composite them on the GPU anyway.

### Rules

```css
/* Good: specific property that benefits from GPU compositing */
.animated-card {
  will-change: transform;
}

/* Good: multiple compositor-friendly properties */
.animated-card {
  will-change: transform, opacity;
}

/* Bad: never use will-change: all */
.animated-card {
  will-change: all;
}

/* Bad: properties that can't be GPU-composited anyway */
.animated-card {
  will-change: background-color, padding;
}
```

### Useful Properties

| Property | GPU-compositable | Worth using `will-change` |
| --- | --- | --- |
| `transform` | Yes | Yes |
| `opacity` | Yes | Yes |
| `filter` (blur, brightness) | Yes | Yes |
| `clip-path` | Engine-, value- and version-dependent — not a cross-browser guarantee | Only after profiling shows a benefit |
| `top`, `left`, `width`, `height` | No | No |
| `background`, `border`, `color` | No | No |

`clip-path` needs care because animation support and compositor promotion are different
questions. Animating it works cross-browser for interpolable `<basic-shape>` values (Chrome 55,
Firefox 49, Safari 12.1) — non-interpolable value combinations still will not animate. Whether
that animation runs off the main thread is another matter: Chromium enabled composited
`clip-path` animations in 2026, while the equivalent Gecko and WebKit bugs are still open. So
treat a clip-path animation as paint-bound unless you have profiled the specific browser,
shape, and version.

`will-change` is a hint, not a promotion switch. Browsers already treat a currently-animating
property as if it were declared in `will-change`, so adding it rarely changes anything; overuse
costs memory. Add it only when profiling shows a measurable win, and remove it after.

### When to Skip

Modern browsers are already good at optimizing on their own. Only add `will-change` when you notice first-frame stutter; Safari in particular benefits from it. Don't add it preemptively to every animated element; each extra compositing layer costs memory.
