# Tailwind Animation Notes

Source: https://tailwindcss.com/docs/animation

Use this when a task needs Tailwind's animation utility surface without loading
the full upstream page.

## Utility Surface

- `animate-spin` maps to the default spin keyframe and is appropriate for
  progress indicators.
- `animate-ping` scales and fades, usually for notification/ripple indicators.
- `animate-pulse` changes opacity and is common for skeleton loading.
- `animate-bounce` loops vertical movement and should be rare in product UI.
- `animate-none` removes animation.
- `animate-[<value>]` sets the `animation` declaration to the arbitrary value.
- `animate-(<custom-property>)` is shorthand for
  `animate-[var(<custom-property>)]`.

## Reduced Motion

Tailwind's docs demonstrate `motion-safe:animate-*` for users who have not
requested reduced motion. For decorative or spatial loops, pair it with a
reduced-motion branch such as `motion-reduce:animate-none`,
`motion-reduce:opacity-100`, or an instant state.

## Custom Animations

In Tailwind v4, define reusable animation utilities with `@theme`:

```css
@theme {
  --animate-wiggle: wiggle 1s ease-in-out infinite;

  @keyframes wiggle {
    0%,
    100% {
      transform: rotate(-3deg);
    }
    50% {
      transform: rotate(3deg);
    }
  }
}
```

This creates the `animate-wiggle` utility. Use this for repeated product motion
tokens; prefer component CSS for one-off keyframes.

## Review Notes

- Infinite animations need a clear user value such as progress or active state.
- Arbitrary animations are acceptable for isolated experiments but should not
  become the dominant style vocabulary.
- Check that custom `animate-*` classes are statically discoverable or
  registered with `@source inline()`.
