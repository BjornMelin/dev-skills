# Tailwind Theme Variable Notes

Source: https://tailwindcss.com/docs/theme

Use this when a task needs Tailwind v4 CSS-first token behavior for motion.

## Core Model

- Tailwind v4 theme variables are CSS variables defined with `@theme`.
- Theme variables do more than ordinary custom properties: they determine which
  utilities and variants exist.
- `@theme` declarations must be top-level, not nested under selectors or media
  queries.
- Use `:root` for ordinary CSS variables that should not create utilities.
- Tailwind's package source defines default motion tokens such as
  `--ease-in`, `--ease-out`, `--animate-spin`, `--animate-ping`,
  `--animate-pulse`, `--animate-bounce`,
  `--default-transition-duration`, and
  `--default-transition-timing-function`.

## Motion-Relevant Namespaces

- `--animate-*` creates `animate-*` utilities.
- `--ease-*` creates easing utilities.
- `--perspective-*` can support 3D transform utilities.
- `--shadow-*`, `--drop-shadow-*`, and `--blur-*` can affect animated visual
  polish but can be paint-heavy in hot paths.
- `--default-transition-duration` and
  `--default-transition-timing-function` affect default transition output when
  the project customizes them.

## Token Decisions

Use `@theme` when:

- the value represents a reusable product motion pattern;
- a class should be generated from the token;
- another package or app needs the same utility vocabulary.

Use component CSS when:

- the keyframe is one-off or component-local;
- browser support guards such as `@supports` dominate the implementation;
- the motion is easier to reason about outside the markup.
- the variable needs to be changed under a selector or media query without
  creating a new Tailwind utility.

Use existing tokens when possible. Adding `--animate-*`, `--ease-*`, or default
transition variables should be a design-system change, not a shortcut for a
single component.

## Runtime Variable Boundary

Use ordinary CSS variables for values consumed by runtime animation engines:

```css
:root {
  --motion-dialog-y: 0.5rem;
}
```

Then reference the variable from Tailwind-compatible CSS, Motion, GSAP, or
WAAPI without minting a Tailwind utility. Promote it to `@theme` only when a
stable utility API is needed.

## Example

```css
@theme {
  --animate-dialog-enter: dialog-enter 180ms ease-out both;

  @keyframes dialog-enter {
    from {
      opacity: 0;
      transform: translateY(0.5rem) scale(0.98);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
}
```

```tsx
<div className="motion-safe:animate-dialog-enter motion-reduce:opacity-100" />
```
