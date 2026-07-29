# GSAP React Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Confirm React/client boundary and installed @gsap/react.
- [ ] Register useGSAP where the project pattern expects plugin registration.
- [ ] Scope selectors to refs or context.
- [ ] Revert contexts and kill manually owned animations on cleanup.
- [ ] Check gotcha: Do not run GSAP setup in a server component.
- [ ] Check gotcha: Avoid unscoped string selectors in component code.
- [ ] Check gotcha: Dependency changes should either rebuild inside useGSAP safely or use contextSafe callbacks.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
