# Rive layout, fit, resize, and DPR behavior

Skill: web-rive
Checked at: 2026-06-04

## When To Load

- Read when sizing a Rive canvas/component, changing artboards, or debugging cropped/blurred assets.

## Source Anchors

- https://rive.app/docs/runtimes/web
- https://rive.app/docs/runtimes/web/state-machines

## Reference Notes

- Canvas size, CSS size, device pixel ratio, and Rive layout fit/alignment all contribute to perceived framing and sharpness.
- Use the runtime layout options and surrounding CSS together; do not rely on the canvas default size.
- Resize observers or framework layout effects should update the runtime at the owner boundary.

## Focused Checks

- Check desktop, mobile, high-DPR, and container resize.
- Verify fallback dimensions so layout does not jump before the asset loads.

## Failure Modes

- A zero-height or auto-height container around the Rive canvas.
- Cropping interactive state-machine hit areas by CSS overflow without review.


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
