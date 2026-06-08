# Web CSS Animations Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Identify the state driver and animated properties.
- [ ] Use explicit transition-property lists and product motion tokens.
- [ ] Add reduced-motion behavior beside the motion.
- [ ] Guard new CSS with @supports or local browser policy.
- [ ] Check gotcha: transition: all hides expensive accidental properties.
- [ ] Check gotcha: Unregistered custom properties animate discretely.
- [ ] Check gotcha: animation shorthand resets animation-timeline, so set timeline after shorthand.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
