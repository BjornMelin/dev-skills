# Browser Motion Performance Notes

Use this reference when reviewing or implementing CSS motion that may affect
rendering performance, low-end devices, or visual stability.

## Sources Checked

- web.dev high-performance CSS animations:
  https://web.dev/articles/animations-guide
- web.dev animations and performance:
  https://web.dev/articles/animations-and-performance
- web.dev transitions:
  https://web.dev/learn/css/transitions
- MDN `will-change`:
  https://developer.mozilla.org/en-US/docs/Web/CSS/will-change
- MDN animatable CSS properties:
  https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Animations/Animatable_properties

## Property Cost

- Prefer `transform` and `opacity` in hot paths. Browsers can usually optimize
  these more cheaply than layout or paint properties.
- Layout-affecting properties such as `width`, `height`, `top`, `left`,
  `right`, `bottom`, margin, padding, and font sizing can be valid for small
  one-off effects, but they need target limits and profiling proof.
- Paint-heavy properties such as large shadows, filters, backdrop filters, and
  gradients can be expensive even when layout is stable.
- Text animation can harm readability and layout stability. Preserve line
  height, wrapping, focus visibility, and hit target geometry.
- Long spatial transitions, parallax, and scroll-linked effects need stronger
  accessibility review than small hover/focus fades.

## `will-change`

- Treat `will-change` as a last-resort or just-in-time hint for real animation
  pressure, not a default optimization.
- Scope it to the exact property that is about to change, usually `transform`
  or `opacity`.
- Avoid `will-change: all`, `contents`, or `scroll-position` for ordinary UI
  motion.
- Remove temporary `will-change` after the animation where practical. Permanent
  hints are only reasonable for frequently animated controls with measured need.

## Reduced Motion

- Respect `prefers-reduced-motion: reduce` for nonessential movement,
  parallax, scroll-linked effects, long travel distances, autoplay loops, and
  decorative infinite animations.
- Prefer instant state changes, opacity-only fades, color changes, or shorter
  durations when motion is reduced.
- Do not remove functional feedback entirely. Keep focus, busy, pressed, and
  error states perceivable.
- Validate the reduced-motion branch with the browser or OS setting when
  possible; reading CSS alone does not prove the active preference path.

## Verification

- Test rapid interruption: hover in/out, focus/blur, open/close, route changes,
  resize, and repeated triggers.
- Test reduced-motion mode with browser/emulator settings, not only by reading
  CSS.
- For performance-sensitive work, profile on the relevant route or device with
  paint flashing, FPS/rendering tools, CPU throttling, or the repo's browser
  evidence lane.
