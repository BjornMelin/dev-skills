# GSAP ScrollTrigger Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Identify scroll container, trigger, start/end, pin, scrub, and responsive ownership.
- [ ] Attach ScrollTrigger to a top-level tween or timeline.
- [ ] Plan refresh ordering after fonts/images/layout changes.
- [ ] Verify resize, route unmount, reduced motion, and mobile scroll.
- [ ] Check gotcha: Do not put ScrollTriggers inside child tweens of a nested timeline.
- [ ] Check gotcha: Pinned scenes affect layout and need refresh proof.
- [ ] Check gotcha: Route transitions must kill/revert triggers or scope them to a context.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
