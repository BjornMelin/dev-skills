# Native Lottie playback refs and lifecycle

Skill: native-lottie
Checked at: 2026-06-04

## When To Load

- Read when code controls LottieView refs, imperative playback, progress, segments, or component unmount.

## Source Anchors

- https://docs.expo.dev/versions/latest/sdk/lottie/
- https://reactnative.dev/docs/accessibility

## Reference Notes

- The owning component should control playback refs and stop/reset behavior. Avoid globally reachable animation handles.
- Progress-driven Lottie should have a stable source of truth and avoid per-frame React re-renders where native animation values can own the update.
- Autoplay/loop should pause or simplify under reduced motion and when screens lose focus.

## Focused Checks

- Test screen blur/focus, unmount, app background/foreground, and asset replacement.
- Check accessible label/state outside the animation view.

## Failure Modes

- Leaving looping animations active behind a hidden route.
- Using decorative animation progress as the only indication of completion.


## Operating Guidance

lottie-react-native and dotLottie native assets, Expo compatibility, asset bundling, refs, playback control, accessibility, reduced motion, and platform validation.

### Decision Boundaries

- Use web-lottie for browser Lottie.
- Use native-rive for interactive Rive state machines.
- Use native-motion-core for code-driven Reanimated motion.

### Workflow Details

1. Check Expo SDK package compatibility and asset format.
2. Bundle assets through the app asset pipeline.
3. Own playback refs, pause/stop behavior, and unmount cleanup.
4. Validate Android/iOS rendering, reduced motion, and accessibility labels.

### Gotchas

- Large JSON animations can hurt startup and memory.
- Autoplay loops need pause/reduced-motion behavior.
- Native asset paths differ from web URLs and need bundler-safe imports.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
