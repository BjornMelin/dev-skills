# Tailwind Motion Field Guide

Load this before the larger provenance notes when a task needs current Tailwind
v4 motion decisions, class-generation safety, or audit heuristics.

## Version And Setup Probe

- Check `package.json`, lockfiles, and CSS entrypoints before writing classes.
- Treat the installed package as the implementation truth. Latest Tailwind docs
  may describe APIs that are absent from an older local pin.
- Tailwind v4 apps usually import Tailwind from CSS with
  `@import "tailwindcss";` and define design tokens with top-level `@theme`.
- Tailwind v3 apps usually rely on `tailwind.config.*` and `content` arrays.
  Do not apply v4-only `@source inline()` or CSS-first config assumptions to a
  v3-only project.
- Prefer the repo's existing utility combiner (`cn`, `clsx`, `cva`, etc.) and
  token names instead of inventing parallel helpers.
- If both `tailwind.config.*` and v4 CSS directives exist, identify the
  ownership boundary before editing. Do not move `content` or `safelist` logic
  blindly; v4 source policy should normally live beside the Tailwind import.

## Engine Boundary

- Tailwind is best for static utility classes and CSS token APIs.
- Use Tailwind transitions for state changes that already exist in CSS:
  hover, focus, active, open, selected, disabled, validation, and data/ARIA
  variants.
- Use Tailwind animation utilities for loops or finite keyframes that are
  globally reusable.
- Use CSS when the selector, browser support guard, or keyframe definition is
  clearer outside markup.
- Use Motion React, GSAP, or WAAPI when the effect needs runtime measurement,
  timeline orchestration, interruption control, presence/layout coordination,
  scroll values, seeking, or cancellation.

## Class Generation

Tailwind scans files as plain text. It does not understand JavaScript string
interpolation, prop values, or template-literal construction.

Prefer:

```tsx
const speedClass = {
  fast: 'duration-100',
  normal: 'duration-150',
  slow: 'duration-300',
} as const;
```

Avoid:

```tsx
const speedClass = `duration-${speed}`;
```

For generated classes that cannot be literal in source, keep the set finite and
register it in the stylesheet:

```css
@import "tailwindcss";
@source inline("{motion-safe:,motion-reduce:}animate-enter-{fast,soft}");
```

Use `@source not` to exclude noisy paths, `source(none)` when a stylesheet must
register every source explicitly, and `@source not inline()` to exclude a finite
class set.

## Motion Tokens

- `@theme` variables create Tailwind utilities and CSS custom properties.
- Use `--animate-*` for reusable `animate-*` utilities.
- Use `--ease-*`, `--default-transition-duration`, and
  `--default-transition-timing-function` only when the repo already tokenizes
  motion at that level or a repeated product pattern justifies it.
- Keep `@theme` top-level. Use `:root` for ordinary CSS variables that should
  not generate utilities.
- Define keyframes inside `@theme` when they are only needed through a
  `--animate-*` utility; define them outside `@theme` when the keyframes must
  always be emitted or are component-local.
- Share Tailwind v4 motion tokens with Motion, GSAP, or WAAPI by reading CSS
  variables at the runtime boundary instead of duplicating duration/easing
  literals in JavaScript.
- Name motion tokens by product intent, not incidental values:
  `--animate-dialog-enter`, not `--animate-fade-y-8`.

## CSS Variable Boundary

- `@theme --animate-*` and `@theme --ease-*` are public utility APIs. Changing
  them can rename or alter generated classes across the app.
- `:root --motion-*`, scoped data-theme variables, and component custom
  properties are runtime CSS variables. They are suitable for JS animation
  libraries, theming, and component internals when no Tailwind utility should be
  generated.
- `@theme inline` can intentionally map an existing CSS variable into a
  Tailwind utility namespace, but use it only when the app has an explicit
  token indirection policy.
- Keep reduced-motion overrides close to the owning component or stylesheet so
  token changes cannot reintroduce movement silently.

## Transition Utilities

- Prefer specific utilities: `transition-transform`, `transition-opacity`,
  `transition-colors`, `transition-shadow`, `transition-none`, or
  `transition-[height]`.
- `transition-(--properties)` is shorthand for a CSS variable property list.
- `transition` is broader in v4 than many teams expect. It includes transform
  longhands, filters, discrete properties, and several interaction properties.
- Use `transition-all` only after checking the whole declaration surface is
  intended to animate.
- Use `transition-discrete` only when discrete property transitions are part of
  the design and browser support is acceptable.

## Animation Utilities

- Built-ins are `animate-spin`, `animate-ping`, `animate-pulse`,
  `animate-bounce`, and `animate-none`.
- `animate-[value]` accepts an arbitrary animation declaration.
- `animate-(--my-animation)` expands to `animation: var(--my-animation)`.
- Promote repeated arbitrary animations to `@theme --animate-*` variables.
- Guard decorative infinite loops with `motion-safe:` and provide
  `motion-reduce:animate-none` or an equivalent non-spatial substitute.

## Reduced Motion

- Use `motion-safe:` to opt nonessential movement in only for users who have
  not requested reduced motion.
- Use `motion-reduce:` to remove transitions, transforms, and animations or to
  substitute opacity/color/instant state changes.
- For CSS-only keyframes, add an explicit `@media (prefers-reduced-motion:
  reduce)` branch when Tailwind variants cannot reach the animated selector.
- Keep reduced-motion classes in the same component as the motion unless a
  shared component or stylesheet owns the policy.
- Verify with browser/device reduced-motion emulation, not only by reading the
  class list.

## Review Heuristics

- Every motion class should have a clear driver: pseudo-class, data/ARIA state,
  render state, route state, or loop.
- Literal class maps are safer than interpolated class names.
- Arbitrary values should be rare, repeated values should become tokens, and
  one-off product motion should usually be component CSS.
- Animated surfaces need stable layout dimensions, especially text, media,
  grids, and lists.
- The closeout should include class-generation evidence, reduced-motion proof,
  responsive checks, and the repo's focused validation commands.
