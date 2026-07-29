# Native Motion Core Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect Expo SDK, RN, Reanimated, Worklets, Gesture Handler, Babel config, and New Architecture mode.
- [ ] Pick the smallest primitive: RN state/style, Reanimated CSS, shared values, gestures, scroll, or layout animation.
- [ ] Keep product state in React/store and transient motion in shared values.
- [ ] Validate interruption, unmount, reduced motion, and iOS/Android behavior.
- [ ] Check gotcha: Reanimated 4 requires compatible Worklets and New Architecture; Reanimated 3 advice differs.
- [ ] Check gotcha: Functions passed to scheduleOnRN must be defined on the RN runtime, not inside a worklet.
- [ ] Check gotcha: Reading sharedValue.value on JS can block; derive/consume on the UI thread.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
