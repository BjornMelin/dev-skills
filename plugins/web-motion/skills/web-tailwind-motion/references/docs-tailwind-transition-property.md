# Tailwind Transition Notes

Sources:

- https://tailwindcss.com/docs/transition-property
- https://tailwindcss.com/docs/transition-behavior

Use this when a task needs Tailwind transition utility details without loading
the full upstream pages.

## Property Utilities

- `transition` sets a broad property list plus the default transition timing
  function and duration from Tailwind's transition defaults.
- `transition-all` sets `transition-property: all`.
- `transition-colors` targets color-related properties, including outline and
  text-decoration colors in current v4 docs.
- `transition-opacity` targets opacity.
- `transition-shadow` targets box shadow.
- `transition-transform` targets `transform`, `translate`, `scale`, and
  `rotate`.
- `transition-none` disables property transitions.
- `transition-[<value>]` sets an arbitrary transition-property list.
- `transition-(<custom-property>)` is shorthand for
  `transition-[var(<custom-property>)]`.
- Tailwind package source resolves transition defaults through
  `--default-transition-duration` and
  `--default-transition-timing-function` theme variables.

Prefer the narrowest property utility that expresses the intended effect.
`transition` and `transition-all` can animate future CSS changes accidentally.

## Discrete Transitions

- `transition-normal` sets `transition-behavior: normal`.
- `transition-discrete` sets `transition-behavior: allow-discrete`.

Use `transition-discrete` for discrete properties such as display only when the
local browser-support policy allows it. Prefer opacity/transform alternatives
when support or behavior is uncertain.

## Reduced Motion

Tailwind supports `motion-safe:` and `motion-reduce:` variants for transitions
and animations. Pair movement with classes such as:

```tsx
<button
  className="
    transition-transform hover:-translate-y-1
    motion-reduce:transition-none motion-reduce:hover:transform-none
  "
/>
```

## Review Notes

- Do not use `transition-all` as a default component style.
- Avoid animating layout and paint-heavy properties in repeated lists or hot
  paths unless measurement proves it is acceptable.
- When using arbitrary transition properties, check generated CSS and promote
  repeated values to a token or component class.
