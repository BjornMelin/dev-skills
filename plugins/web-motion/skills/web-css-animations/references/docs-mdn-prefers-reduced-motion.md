# MDN Prefers Reduced Motion Notes

Skill: web-css-animations
Checked at: 2026-06-04

## Source

- MDN `prefers-reduced-motion`: https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@media/prefers-reduced-motion

This file is a concise routing note, not a copied MDN page. Use the official
MDN page for exact wording, compatibility tables, examples, contributors, and
license details.

## Use When

- Adding or reviewing `@media (prefers-reduced-motion: reduce)`.
- Replacing spatial, parallax, scroll-linked, infinite, or long-running motion.
- Coordinating CSS motion with JS libraries that also expose reduced-motion
  controls.

## Review Notes

- Reduced motion should usually remove nonessential movement, not simply speed
  it up.
- Keep opacity, color, and instant state changes available when they preserve
  context without spatial movement.
- Verify that CSS and JavaScript motion layers agree on the same user
  preference.
