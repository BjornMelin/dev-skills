# Native Controls Transitions Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Identify whether navigation, native control, or app state owns the transition.
- [ ] Prefer platform-native transition knobs before custom overlays.
- [ ] Keep Expo UI controls as leaf native controls.
- [ ] Validate iOS and Android behavior when native controls or navigation config change.
- [ ] Check gotcha: Navigation transitions can fight screen-level Reanimated transitions.
- [ ] Check gotcha: Expo UI control props are not arbitrary React Native View animation surfaces.
- [ ] Check gotcha: Route params and unmount timing affect transition cleanup.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
