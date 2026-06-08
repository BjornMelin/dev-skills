# Web Rive Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect asset ownership, state machine names, inputs, autoplay, and runtime package.
- [ ] Bind inputs through stable component state and cleanup runtime instances.
- [ ] Add fallback and accessible semantics outside canvas.
- [ ] Review URL actions and remote asset policy.
- [ ] Check gotcha: Canvas output is not self-describing to assistive tech.
- [ ] Check gotcha: State machine input names are asset contracts; verify against the asset.
- [ ] Check gotcha: Remote .riv files and URL actions need explicit allowlisting.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
