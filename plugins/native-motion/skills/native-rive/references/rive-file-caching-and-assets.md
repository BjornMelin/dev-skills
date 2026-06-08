# Native Rive file caching and asset loading

Skill: native-rive
Checked at: 2026-06-04

## When To Load

- Read when `.riv` files are bundled, cached, loaded remotely, reused across views, or include out-of-band assets.

## Source Anchors

- https://rive.app/docs/runtimes/react-native
- https://rive.app/docs/runtimes/react-native/state-machines

## Reference Notes

- A `.riv` file can be expensive to load and parse. Reuse/caching can be useful when the same file appears in multiple places, but ownership and invalidation must be explicit.
- Bundled native assets, remote assets, images, fonts, and audio have different build and runtime requirements.
- Remote Rive assets need allowlisting, cache policy, and failure fallback.

## Focused Checks

- Test missing asset, slow load, route unmount, and repeated mount behavior.
- Verify development-build/native runtime requirements before closing.

## Failure Modes

- Hard-coded asset names not verified against native bundling output.
- Remote `.riv` files without fallback or security review.


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
