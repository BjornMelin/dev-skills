# Timeline Current Notes

Use this file when timeline work needs more detail than the default `SKILL.md`
path. Keep application code anchored to the target repo's installed `gsap`
version.

## Position Parameter

The position parameter controls where tweens, timelines, labels, callbacks, and
pauses are inserted.

- No position means append at the end.
- Number values are absolute seconds from the timeline start.
- `"+=1"` and `"-=1"` are relative to the current end of the timeline.
- Labels insert at named positions. Referencing a missing label creates it at
  the end of the timeline.
- `"<"` and `">"` point to the start or end of the most recently inserted
  child, not necessarily the end of the whole timeline.
- `"<0.2"` is shorthand for `"<+=0.2"`. `">-0.2"` starts before the recent
  child's end.
- Percent forms are supported in GSAP 3.7+: `"-=25%"`, `"+=50%"`,
  `"<25%"`, `"<+=25%"`, and `"label+=30%"`.

## Timeline Vars

Common timeline vars:

- `defaults`: values inherited by child tweens, such as `duration`, `ease`,
  `overwrite`, and `stagger`.
- `paused`, `reversed`, `delay`, `repeat`, `repeatDelay`, `repeatRefresh`,
  `yoyo`.
- `onStart`, `onUpdate`, `onRepeat`, `onComplete`, `onReverseComplete`, plus
  matching `*Params` arrays and `callbackScope`.
- `smoothChildTiming`: advanced behavior that repositions children to keep
  playback visually smooth when timing changes while running.
- `autoRemoveChildren`: can improve memory for fire-and-forget roots, but it
  removes completed children and limits reverse/scrub control.
- `scrollTrigger`: valid for a top-level scroll-controlled timeline.

Do not use timeline `duration` as a substitute for child tween duration. Put
child duration in `defaults` or in each tween's vars.

## Methods Worth Checking

- `add(child, position)`: accepts Tween, Timeline, label string, callback, or
  an array of those.
- `addLabel(label, position)` and `removeLabel(label)`.
- `call(callback, params, position)` for in-sequence callbacks.
- `addPause(position, callback, params)` and `removePause(position)`.
- `set(targets, vars, position)` for zero-duration state changes.
- `getChildren(nested, tweens, timelines, ignoreBeforeTime)` and
  `getTweensOf(targets, onlyActive)` for inspection.
- `currentLabel()`, `nextLabel(time)`, and `previousLabel(time)` for label-based
  controls.
- `seek(position, suppressEvents)`, `time()`, `totalTime()`, `progress()`, and
  `totalProgress()` for playhead control.
- `tweenTo(position, vars)` and `tweenFromTo(from, to, vars)` for animated
  playhead scrubbing.
- `kill()` removes the timeline from its parent. `revert()` kills and restores
  inline styles created by the animation.

## Repeats And Yoyo

- `repeat` counts extra plays after the first play. `repeat: 2` plays three
  iterations total.
- `repeatDelay` adds a gap between repeat cycles.
- `yoyo: true` alternates direction on odd repeat cycles. It does not mean the
  `reversed()` state is true.
- Use `totalTime()` and `totalProgress()` when controls must include repeats
  and repeat delays; use `time()` and `progress()` for the current iteration.
- `onRepeat` fires as iterations change. Keep callbacks idempotent because
  seeking, reversing, and suppressed events can change callback timing.

## Cleanup And Rebuilds

- In mounted UI, prefer `gsap.context()`, `gsap.matchMedia()`, or `useGSAP()`
  so timelines are recorded and reverted with the component/page lifecycle.
- Use `tl.revert()` when inline styles should be restored. Use `tl.kill()` when
  the owner will restore state separately.
- Rebuild timelines only after killing/reverting the previous instance.
- Kill active `tweenTo()`/`tweenFromTo()` control tweens if a new interaction
  supersedes them.
- Do not create timelines inside render functions, pointer handlers, ticker
  callbacks, `requestAnimationFrame`, or `useFrame()` loops. Create once and
  adjust `progress()`, `time()`, or vars.

## Common Gotchas

- `"<"` and `">"` anchor to the most recently inserted child, which can be a
  callback, label, pause, or nested timeline.
- A missing label in a position string is created at the end of the timeline.
  This is useful intentionally but can hide typos.
- `from()` and `fromTo()` tweens can immediately render start values. When
  stacking from-type tweens on the same target/property, consider
  `immediateRender: false` on later tweens.
- `tweenFromTo()` may render its from position immediately; pass
  `{ immediateRender: false }` for user-triggered scrub tweens created in
  advance.
- `clear()` removes children and callbacks, and can optionally clear labels.
  It does not remove timeline event callbacks; use `eventCallback(type, null)`
  for that.
- ScrollTrigger belongs on the top-level timeline. Do not nest
  ScrollTriggered children inside another timeline.

