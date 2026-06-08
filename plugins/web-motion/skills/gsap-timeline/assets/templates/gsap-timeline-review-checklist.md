# GSAP Timeline Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Model the sequence as labels, relative positions, and defaults.
- [ ] Use position parameters instead of delay chains.
- [ ] Store timeline handles when playback control or cleanup is needed.
- [ ] Verify interruptions and reverse/restart behavior.
- [ ] Check gotcha: Timeline constructor duration is not child tween duration.
- [ ] Check gotcha: Nested ScrollTriggers are usually wrong; attach scroll control to the top-level tween/timeline.
- [ ] Check gotcha: Labels are a maintainability tool, not just comments.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
