# TypeGPU Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Check installed typegpu, unplugin-typegpu, @webgpu/types, tsover, and browser/runtime support.
- [ ] Define schemas before resources and shader signatures.
- [ ] Keep root/device/resource ownership explicit.
- [ ] Validate unsupported-browser fallback, reduced-motion/static quality, and GPU cleanup.
- [ ] Check gotcha: A d.* schema is the CPU layout, GPU layout, and TypeScript type source of truth.
- [ ] Check gotcha: TypeScript shader functions require unplugin-typegpu; WGSL-only usage may not.
- [ ] Check gotcha: Do not allocate buffers, textures, bind groups, or pipelines per frame unless measured and cached.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
