# Native Lottie Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Check Expo SDK package compatibility and asset format.
- [ ] Bundle assets through the app asset pipeline.
- [ ] Own playback refs, pause/stop behavior, and unmount cleanup.
- [ ] Validate Android/iOS rendering, reduced motion, and accessibility labels.
- [ ] Check gotcha: Large JSON animations can hurt startup and memory.
- [ ] Check gotcha: Autoplay loops need pause/reduced-motion behavior.
- [ ] Check gotcha: Native asset paths differ from web URLs and need bundler-safe imports.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
