# Native Accessibility Performance Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect platform, Expo SDK, animation engine, and accessibility setting ownership.
- [ ] Classify UI-thread, JS-thread, layout, and GPU work.
- [ ] Add reduced-motion or static behavior without removing functional feedback.
- [ ] Validate on iOS/Android or document skipped device proof.
- [ ] Check gotcha: Reduced motion should not remove essential progress, focus, pressed, or error feedback.
- [ ] Check gotcha: Haptics are feedback, not a substitute for visible state.
- [ ] Check gotcha: Per-frame React state updates can destroy native animation performance.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
