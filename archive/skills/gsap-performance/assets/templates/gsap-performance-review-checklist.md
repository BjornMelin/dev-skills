# GSAP Performance Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Find hot paths and animated properties.
- [ ] Classify layout, paint, composite, and JavaScript costs.
- [ ] Run audit scan and inspect high-confidence findings.
- [ ] Recommend transform/opacity, batching, throttling, or engine changes only with evidence.
- [ ] Check gotcha: will-change is a scoped hint, not a global optimization.
- [ ] Check gotcha: ScrollTrigger refresh calls after layout changes need ordering, not random timeouts.
- [ ] Check gotcha: Infinite tweens need reduced-motion and cleanup behavior.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
