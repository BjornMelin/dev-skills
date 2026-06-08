# Layer budget and will-change discipline

Skill: gsap-performance
Checked at: 2026-06-04

## When To Load

- Read before adding transform hacks, force3D, will-change, or GPU promotion advice.

## Source Anchors

- https://gsap.com/docs/v3/GSAP/gsap.to()
- https://developer.mozilla.org/docs/Web/CSS/CSS_Transitions/Using_CSS_transitions

## Reference Notes

- Promote only hot animated elements and only for the lifetime of the effect when possible.
- Large layers, filters, shadows, video, and text rendering can shift cost from layout to memory/compositing instead of making it free.
- Use transform/opacity as the default fast path, but measure paint-heavy visual effects.

## Focused Checks

- Search for persistent `will-change` on many elements.
- Review memory, layer count, text clarity, and paint cost when adding GPU promotion.

## Failure Modes

- Global CSS that sets `will-change` across component classes.
- Animating filter or shadow while assuming transform-like compositing cost.

## Review Steps

1. Identify the elements promoted by `will-change`, transforms, or `force3D`.
2. Confirm promotion is temporary, targeted, and limited to actively animated elements.
3. Check memory and paint costs for text, filters, shadows, video, and large layers.
4. Remove persistent global `will-change` rules unless measurement proves they are needed.
