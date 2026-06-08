# Support, @supports, and reduced-motion notes

Skill: web-css-animations
Checked at: 2026-06-04

## When To Load

- Read for newer CSS features or accessibility review.

## Support Checklist

1. Check `@supports` for newer features such as scroll timelines, registered properties, and discrete transitions.
2. Pair every nonessential transition or keyframe with `prefers-reduced-motion`.
3. Preserve DOM, focus, and ARIA semantics when visual motion changes visibility.
4. Verify fallback behavior in the repo browser policy before recommending a new CSS motion feature.

## Browser Feature Matrix

| Feature | Review focus | Fallback pattern |
| --- | --- | --- |
| Transitions/keyframes | Broad support, but property cost varies. | Prefer opacity/transform and explicit `transition-property`. |
| Registered custom properties | Requires support for `@property`. | Guard with `@supports` and keep discrete fallback states. |
| Scroll-driven animations | Browser support is still policy-dependent. | Use static layout or a JS/GSAP implementation when browser policy requires it. |
| Discrete transitions | Requires `transition-behavior` and `@starting-style` support. | Use opacity/transform entry/exit or no animation. |

## Reduced-Motion Patterns

- Set `transition: none` or remove nonessential keyframes under
  `@media (prefers-reduced-motion: reduce)`.
- Keep focus movement predictable; do not animate focus targets away from their
  DOM position during keyboard interaction.
- Do not rely on motion alone to announce state changes. Pair animated state
  changes with semantic state, visible text, or ARIA where appropriate.
- Test base and active/open selectors inside the reduced-motion block so active
  states do not reintroduce transforms.
