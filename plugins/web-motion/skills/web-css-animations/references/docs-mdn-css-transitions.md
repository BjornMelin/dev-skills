# MDN CSS Transitions Notes

Skill: web-css-animations
Checked at: 2026-06-04

## Source

- MDN CSS transitions guide: https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Transitions/Using

This file is a concise routing note, not a copied MDN page. Use the official
MDN page for exact wording, compatibility tables, examples, contributors, and
license details.

## Use When

- Implementing ordinary two-state UI motion with `transition`.
- Reviewing transition-property, duration, delay, easing, and cancellation.
- Deciding whether a CSS transition is enough or WAAPI/GSAP should own the
  interaction.

## Review Notes

- Prefer explicit `transition-property` values over `transition: all`.
- Avoid transitioning layout-affecting properties in hot paths. Use transforms
  when the visual result can be preserved.
- Long transitions need a reduced-motion branch or an explicit product reason.
- Discrete transitions such as `display` and `content-visibility` require
  newer CSS features and browser-support checks.
