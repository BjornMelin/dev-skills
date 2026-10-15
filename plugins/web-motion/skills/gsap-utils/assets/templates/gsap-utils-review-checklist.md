# GSAP Utils Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Identify whether the helper should return a reusable function or immediate value.
- [ ] Keep unit handling explicit.
- [ ] Scope selector helpers in component code.
- [ ] Test boundary inputs and cyclic values.
- [ ] Check gotcha: mapRange and normalize operate on numbers, not unit strings.
- [ ] Check gotcha: Omitting the final value returns a reusable function; this is often the intended pattern.
- [ ] Check gotcha: selector(scope) prevents cross-component targeting mistakes.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
