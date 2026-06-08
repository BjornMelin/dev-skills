# Web Tailwind Motion Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect Tailwind version, CSS entrypoint, theme tokens, and class-generation policy.
- [ ] Prefer explicit transition properties and tokenized durations/eases.
- [ ] Use motion-safe/motion-reduce variants for user preference.
- [ ] Validate generated classes are statically discoverable.
- [ ] Check gotcha: transition-all can hide expensive properties.
- [ ] Check gotcha: Tailwind v4 tokens usually live in CSS @theme, not only JS config.
- [ ] Check gotcha: Dynamic class concatenation can be purged or unsupported by local policy.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
