# Web Motion React Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Confirm package import path and React/client boundary.
- [ ] Choose presence, layout, gesture, scroll, or value-based motion deliberately.
- [ ] Respect reduced motion and state ownership.
- [ ] Verify layout projection with resize, interruption, route changes, and hydration.
- [ ] Check gotcha: AnimatePresence requires stable keys and actual unmounts.
- [ ] Check gotcha: Layout animations depend on stable layout boxes and should not fight CSS transitions.
- [ ] Check gotcha: Do not push high-frequency motion values through React state.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
