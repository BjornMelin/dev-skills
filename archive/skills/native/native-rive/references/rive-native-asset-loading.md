# Rive native asset loading and lifecycle

Skill: native-rive
Checked at: 2026-06-04

## When To Load

- Read for .riv bundling, runtime setup, and cleanup.


## Operating Guidance

Rive React Native/Nitro runtime, .riv assets, state machines, inputs, asset loading, platform compatibility, accessibility, and iOS/Android proof.

### Decision Boundaries

- Use web-rive for browser runtime.
- Use native-lottie for Lottie/dotLottie assets.
- Use native-motion-core for code-driven Reanimated motion.

### Workflow Details

1. Check package/runtime compatibility and native build requirements.
2. Verify asset path, state machine name, input names, and autoplay.
3. Map app state to inputs with cleanup on unmount.
4. Validate iOS/Android build/rendering and accessibility fallback.

### Gotchas

- State machine names and inputs are asset contracts.
- Native Rive runtime may require development build proof, not only Expo Go.
- Canvas-like output needs surrounding accessible semantics.

## Validation Notes

- Inspect installed package versions and local architecture before applying examples.
- Prefer the bundled `scripts/audit.mjs doctor --root <repo> --format json` command when setup is unclear.
- Use `scripts/audit.mjs scan --root <repo> --format markdown` for repeatable static findings, then manually verify every finding against current code.
- Close with repo-specific checks and user-visible runtime proof when this skill affects a rendered surface.
