# Rive React/runtime lifecycle

Skill: web-rive
Checked at: 2026-06-04

## When To Load

- Read when using @rive-app/react-webgl2, @rive-app/webgl2, @rive-app/react-canvas, or @rive-app/canvas.


## Operating Guidance

Rive web and React runtime integration, .riv assets, state machines, inputs, lifecycle cleanup, accessibility, remote asset security, and fallback behavior.

### Decision Boundaries

- Use web-lottie for Lottie/dotLottie assets.
- Use native-rive for React Native.
- Use Motion/GSAP/CSS when no .riv asset or state machine is involved.

### Workflow Details

1. Inspect asset ownership, state machine names, inputs, autoplay, and runtime package.
2. Bind inputs through stable component state and cleanup runtime instances.
3. Add fallback and accessible semantics outside canvas.
4. Review URL actions and remote asset policy.

### Gotchas

- Canvas output is not self-describing to assistive tech.
- State machine input names are asset contracts; verify against the asset.
- Remote .riv files and URL actions need explicit allowlisting.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
