# Web Lottie Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect asset format, player package, renderer, autoplay/loop, and hosting path.
- [ ] Create and destroy player instances at the owner boundary.
- [ ] Respect reduced motion and provide non-canvas semantics.
- [ ] Validate asset size, remote URLs, and event listeners.
- [ ] Check gotcha: Canvas-rendered animation needs external accessible text or labels.
- [ ] Check gotcha: Remote animation URLs need CSP/cache/security review.
- [ ] Check gotcha: Looping/autoplay assets require reduced-motion and pause behavior.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
