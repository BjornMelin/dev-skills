# MDN CSS Animations Notes

Skill: web-css-animations
Checked at: 2026-06-04

## Source

- MDN CSS animations guide: https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Animations/Using

This file is a concise routing note, not a copied MDN page. Use the official
MDN page for exact wording, compatibility tables, examples, contributors, and
license details.

## Use When

- Authoring `@keyframes` or the `animation-*` longhands.
- Reviewing iteration count, fill mode, direction, delay, or timing functions.
- Checking whether an animation should be disabled or simplified for reduced
  motion.

## Review Notes

- Prefer `transform` and `opacity` for high-frequency animation. Treat layout,
  text-flow, paint-heavy, and filter-heavy keyframes as risk until measured.
- Keep keyframe names stable and avoid relying on source order when multiple
  animations affect the same property.
- Infinite or long-running animations need a reduced-motion branch unless the
  motion is essential and justified.
- For newer timeline features, verify current browser support before shipping.
