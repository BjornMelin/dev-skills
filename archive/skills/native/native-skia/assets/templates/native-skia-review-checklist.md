# Native Skia Review Checklist

## Before Editing

- [ ] Confirm this skill owns the task and adjacent skills do not.
- [ ] Inspect installed package/framework/runtime versions.
- [ ] Read the first matching reference from SKILL.md resource routing.
- [ ] Run `node scripts/audit.mjs doctor --root <repo> --format json` when setup is unclear.

## Manual Review

- [ ] Check Skia, Expo SDK, platform support, and web requirements.
- [ ] Keep drawing state and animation values in the appropriate runtime.
- [ ] Avoid many animated native views when one canvas is the right surface.
- [ ] Validate memory, resize, background/foreground, and platform rendering.
- [ ] Check gotcha: Skia web may require CanvasKit setup.
- [ ] Check gotcha: Canvas output needs accessible surrounding UI.
- [ ] Check gotcha: Large paths/images/shaders need memory and lifecycle ownership.
- [ ] Verify reduced-motion or accessible static fallback when motion is nonessential.
- [ ] Verify cleanup on unmount/navigation or owner teardown.

## Closeout

- [ ] Run `node scripts/audit.mjs scan --root <repo> --format markdown` when auditing.
- [ ] Run the target repo's focused validation commands.
- [ ] Capture browser/device/manual proof when the changed surface renders motion.
- [ ] Report fixed findings, skipped findings with reasons, and residual risk.
