# GSAP Plugins Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Check installed gsap version and plugin availability.
- [ ] Register plugins exactly once in the runtime boundary.
- [ ] Keep plugin setup out of hot render paths.
- [ ] Verify lifecycle cleanup and license/package constraints.
- [ ] Check gotcha: Plugin imports differ by plugin and package availability; verify before generating code.
- [ ] Check gotcha: SplitText-style text effects need accessibility and reduced-motion fallbacks.
- [ ] Check gotcha: Flip needs measured before/after state ownership; do not mix with separate layout animators.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
