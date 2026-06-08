# CSS Modern Motion Notes

Use this reference when a CSS animation task involves newer browser features or
you need source-backed boundaries without loading full upstream pages.

## Sources Checked

- MDN CSS animations guide:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Animations/Using
- MDN CSS transitions guide:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Transitions/Using
- MDN `animation-timeline`:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/animation-timeline
- MDN scroll-driven animation timelines:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Scroll-driven_animations/Timelines
- MDN animatable CSS properties:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Animations/Animatable_properties
- MDN `@property`:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@property
- MDN `transition-behavior`:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/transition-behavior
- MDN `@starting-style`:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/At-rules/@starting-style
- MDN CSS view transitions:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/View_transitions
- CSS Animations Level 2 draft:
  https://drafts.csswg.org/css-animations-2/
- CSS Transitions Level 2 draft:
  https://drafts.csswg.org/css-transitions-2/
- Scroll-driven Animations Level 1 draft:
  https://drafts.csswg.org/scroll-animations-1/
- CSS Properties and Values API Level 1 draft:
  https://drafts.css-houdini.org/css-properties-values-api-1/

## Transitions

- Transitions interpolate between two style states after a property change.
  Prefer explicit `transition-property` lists over `all`.
- Discrete properties usually switch at the midpoint of an animation. For
  `display` and `content-visibility`, browsers keep the content visible for the
  full transition when `transition-behavior: allow-discrete` is used.
- For entry transitions from an initial render or from `display: none`, pair
  `@starting-style` with the target visible selector.
- `@starting-style` applies to CSS transitions only. It is not part of a
  keyframe animation implementation.
- For older browser support, declare a non-discrete transition first, then a
  second transition declaration with `allow-discrete`.
- Keep discrete transitions targeted. Prefer explicit `display`,
  `content-visibility`, or `overlay` entries over `transition: all`.

## Keyframe Animations

- CSS keyframes define rendered states; `animation-*` properties define timing,
  repetition, fill, direction, play state, and timeline.
- Missing `from`/`0%` or `to`/`100%` keyframes fall back to the element's
  computed styles.
- `animation-composition` controls how multiple animations affecting the same
  property combine. Use it deliberately; do not assume additive behavior.
- `animation-fill-mode: forwards` or `both` can retain side effects from
  animated properties, including stacking context effects.
- Use explicit `animation-duration`, `animation-delay`,
  `animation-iteration-count`, and `animation-fill-mode` when deterministic
  visual capture or tests matter.
- Avoid long-running or infinite keyframes unless they communicate active
  progress and have a reduced-motion branch.

## Scroll-Driven Animations

- `animation-timeline` links a keyframe animation to the default document
  timeline, no timeline, a named scroll/view timeline, `scroll()`, or `view()`.
- The `animation` shorthand resets `animation-timeline` to `auto`. Declare
  `animation-timeline` after `animation`.
- Scroll progress timelines track scroll position of a scroller. View progress
  timelines track a subject's visibility inside its nearest scroller.
- Use `@supports (animation-timeline: view())` or local support policy before
  depending on CSS scroll-driven animation in production.
- MDN notes that a small non-zero duration such as `1ms` can keep behavior
  consistent across engines for scroll-driven effects whose visual progress is
  controlled by the timeline rather than elapsed time.
- Verify the scroll container, axis, timeline scope, and subject visibility in
  a browser. Static CSS review cannot prove timeline progress.

## Custom Properties

- Unregistered custom properties animate discretely.
- Register a custom property with `@property` or `CSS.registerProperty()` when
  it should interpolate by a typed computed value.
- A valid `@property` rule needs `syntax` and `inherits`; `initial-value` is
  required unless the syntax accepts any token stream.
- Use registered custom properties for reusable numeric, color, length,
  percentage, or angle tokens that are animated in CSS.
- Register only stable public custom properties. Avoid sprinkling one-off
  registrations when a normal property or transform function is clearer.

## View Transitions Boundary

- The CSS view transitions module defines pseudo-elements, selectors, and
  properties for styling snapshots created by the View Transition API.
- CSS alone does not initiate a same-document View Transition; code calls the
  browser API and CSS styles the resulting transition.
- Use this skill for `view-transition-name`, `::view-transition-*`,
  `:active-view-transition`, and CSS timing. Use framework/API docs for the DOM
  update lifecycle and browser call.
- Avoid duplicate `view-transition-name` values among simultaneously visible
  elements unless the browser/framework explicitly scopes them.
