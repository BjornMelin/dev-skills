# Asset accessibility and security review

Skill: web-lottie
Checked at: 2026-06-04

## When To Load

- Read before accepting remote assets, canvas-only output, autoplay loops, or URL actions.


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
