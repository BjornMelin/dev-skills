# Web Three R3F Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect Three/R3F/Drei/React versions and asset pipeline.
- [ ] Choose Canvas/R3F or plain Three renderer based on local ownership.
- [ ] Reserve stable layout, DPR, fallback, and reduced-motion behavior.
- [ ] Verify nonblank pixels, resize, interaction, asset errors, and cleanup.
- [ ] Check gotcha: Undefined parent height causes blank canvases.
- [ ] Check gotcha: React state inside useFrame causes frame-loop reconciliation.
- [ ] Check gotcha: R3F auto-disposal does not cover every primitive or externally created object.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
