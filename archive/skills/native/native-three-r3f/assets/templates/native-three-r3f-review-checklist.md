# Native Three R3F Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Inspect Expo SDK, GL/WebGPU package, R3F/Three versions, and asset pipeline.
- [ ] Choose native R3F only when a 3D scene is the product surface.
- [ ] Own canvas dimensions, DPR/quality, loaders, and cleanup.
- [ ] Validate on device/development build for native GPU risk.
- [ ] Check gotcha: Browser R3F examples often assume DOM/WebGL APIs absent on native.
- [ ] Check gotcha: Native asset loading and decoder paths differ from web.
- [ ] Check gotcha: GPU runtime changes need device proof, not just TypeScript.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
