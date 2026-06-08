# Runtime events, markers, and playback contracts

Skill: web-lottie
Checked at: 2026-06-04

## When To Load

- Read when code controls Lottie playback, segments, markers, events, or synchronization with app state.

## Source Anchors

- https://github.com/airbnb/lottie-web/wiki/Usage
- https://github.com/LottieFiles/dotlottie-web

## Reference Notes

- Use the player instance as the playback owner. Store it, destroy it at the component boundary, and avoid duplicate instances in the same container.
- Segments and markers should come from the asset contract and be validated against actual asset metadata.
- Synchronizing Lottie with app state requires interruption behavior for pause, stop, seek, route unmount, and asset replacement.

## Focused Checks

- Verify load, error, complete, loop, and destroy behavior.
- Test replacing the `path` or `src` while the animation is playing.

## Failure Modes

- Calling global lottie commands by name when a local instance handle is available.
- Leaving event listeners attached after destroy.


## Operating Guidance

lottie-web, dotLottie web components, animation JSON/dotLottie assets, player lifecycle, cleanup, renderer choice, accessibility, and asset validation.

### Decision Boundaries

- Use native-lottie for React Native.
- Use Rive for interactive state machines.
- Use CSS/WAAPI for simple UI motion that does not need designer-authored assets.

### Workflow Details

1. Inspect asset format, player package, renderer, autoplay/loop, and hosting path.
2. Create and destroy player instances at the owner boundary.
3. Respect reduced motion and provide non-canvas semantics.
4. Validate asset size, remote URLs, and event listeners.

### Gotchas

- Canvas-rendered animation needs external accessible text or labels.
- Remote animation URLs need CSP/cache/security review.
- Looping/autoplay assets require reduced-motion and pause behavior.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
