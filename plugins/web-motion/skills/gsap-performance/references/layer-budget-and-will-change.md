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

## Operating Guidance

- Use transform and opacity as the default fast path, but do not treat GPU
  promotion as free. A composited layer can trade layout cost for memory,
  upload, blending, and text-rendering cost.
- Scope `will-change` to the shortest practical lifetime. Prefer adding it
  before the effect and clearing it in completion or cleanup callbacks.
- Treat `force3D`, `translateZ(0)`, and similar promotion hacks as measured
  exceptions. They are not substitutes for reducing animated area, paint-heavy
  effects, or JavaScript work.
- For ScrollTrigger scenes, review refresh ordering and layout dependencies
  before assuming layer promotion is the bottleneck.

## Command References

- Run the local static scan from the skill folder:
  `node scripts/audit.mjs scan --root <repo> --format markdown`
- Use JSON output when collecting repeatable evidence:
  `node scripts/audit.mjs scan --root <repo> --format json --output gsap-performance-audit.json`
- Check setup and package context before interpreting scan output:
  `node scripts/audit.mjs doctor --root <repo> --format json`

## Validation Notes

- Confirm installed `gsap` and framework versions before applying examples.
- Inspect DevTools performance/layers output for large layers, layer count,
  paint cost, and text clarity when recommendations affect rendered UI.
- Close with repo-specific checks and user-visible proof when changing real
  animation behavior.
