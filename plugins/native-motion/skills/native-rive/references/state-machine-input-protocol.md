# Native Rive state-machine input protocol

Skill: native-rive
Checked at: 2026-06-04

## When To Load

- Read when app state drives boolean, number, or trigger inputs in a native Rive state machine.

## Source Anchors

- https://rive.app/docs/runtimes/react-native/state-machines
- https://rive.app/docs/runtimes/react-native

## Reference Notes

- State machine names and input names are designer/runtime contracts. Verify them with the asset before wiring app logic.
- Triggers are events, not durable state. Boolean/number inputs should represent durable state and be reset intentionally.
- Keep app state to Rive input mapping small and observable for debugging.

## Focused Checks

- Exercise every state-machine input and reset/replay path.
- Check accessibility fallback for every meaningful visual state.

## Failure Modes

- Guessing input names from code comments.
- Using animation state as the only business-state source.


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
