# Native Rive Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Check package/runtime compatibility and native build requirements.
- [ ] Verify asset path, state machine name, input names, and autoplay.
- [ ] Map app state to inputs with cleanup on unmount.
- [ ] Validate iOS/Android build/rendering and accessibility fallback.
- [ ] Check gotcha: State machine names and inputs are asset contracts.
- [ ] Check gotcha: Native Rive runtime may require development build proof, not only Expo Go.
- [ ] Check gotcha: Canvas-like output needs surrounding accessible semantics.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
