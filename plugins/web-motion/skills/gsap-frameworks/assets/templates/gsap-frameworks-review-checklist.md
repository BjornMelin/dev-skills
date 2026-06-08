# GSAP Frameworks Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Identify framework lifecycle hooks and mount/unmount boundaries.
- [ ] Scope selectors to the component root.
- [ ] Register plugins once at module/app boundary.
- [ ] Revert contexts, kill timelines, and clean listeners on unmount.
- [ ] Check gotcha: Do not let selector text escape a component root.
- [ ] Check gotcha: Hydration/client-only boundaries matter in SSR frameworks.
- [ ] Check gotcha: Framework transitions may already own enter/exit timing; avoid duplicate animation owners.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
