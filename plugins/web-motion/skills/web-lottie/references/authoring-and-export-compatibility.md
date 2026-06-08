# Authoring and export compatibility

Skill: web-lottie
Checked at: 2026-06-04

## When To Load

- Read when accepting designer-authored Lottie JSON/dotLottie assets or debugging mismatch between After Effects preview and runtime output.

## Source Anchors

- https://github.com/airbnb/lottie-web/wiki/Usage
- https://github.com/LottieFiles/dotlottie-web

## Reference Notes

- Treat the animation asset as a contract: dimensions, frame rate, markers, image assets, fonts/text, expressions, and unsupported effects should be reviewed before integration.
- Prefer local bundled assets or pinned package versions for production-critical animation. Remote library/CDN paths need supply-chain, CSP, cache, and outage review.
- Large vector assets can hurt startup, parse time, and memory even when rendering is GPU/canvas-backed.

## Focused Checks

- Check asset size, renderer type, external images/fonts, and unsupported Bodymovin features.
- Verify loop/autoplay behavior under reduced motion.

## Failure Modes

- Accepting arbitrary remote Lottie URLs from users or CMS data.
- Using canvas output without surrounding accessible semantics.


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
