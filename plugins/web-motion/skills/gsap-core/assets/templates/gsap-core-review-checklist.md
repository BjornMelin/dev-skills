# GSAP Core Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect installed GSAP version and framework ownership.
- [ ] Prefer official transform aliases and explicit eases/durations.
- [ ] Add matchMedia or reduced-motion handling for nonessential motion.
- [ ] Run the audit CLI and verify findings manually.
- [ ] Check gotcha: Do not animate raw transform strings when GSAP aliases express the same effect.
- [ ] Check gotcha: Set immediateRender intentionally when stacking from/fromTo tweens.
- [ ] Check gotcha: Store returned tween handles when playback control or cleanup is needed.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
