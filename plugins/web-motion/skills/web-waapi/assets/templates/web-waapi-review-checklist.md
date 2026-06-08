# Web WAAPI Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Check browser support and local fallback policy.
- [ ] Create keyframes/options with explicit duration, fill, easing, and composite behavior.
- [ ] Own animation cancellation and finish behavior.
- [ ] Verify rapid interruptions, route unmount, reduced motion, and commitStyles usage.
- [ ] Check gotcha: commitStyles persists computed styles and should be followed by cancel when appropriate.
- [ ] Check gotcha: fill: forwards can retain stacking/style side effects.
- [ ] Check gotcha: Multiple animations on the same property need composite/replace intent.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
