# runtime-bun-run-bun-flag

## Why

Some package binaries have a `#!/usr/bin/env node` shebang. Bun may run these under Node by default (to match expectations). The `--bun` flag forces Bun to execute the binary with the Bun runtime instead.

This is especially relevant when you want “Bun all the way down” (for consistency or platform requirements).

## Do

- Use `bun run --bun <bin> ...` when you intentionally want Bun to execute a Node-shebang binary.
- Consider `bunfig.toml` (`run.bun = true`) or `BUN_RUN_BUN=1` when a repo consistently needs this behavior.

## Don't

- Don’t blanket-apply `--bun` without verifying the binary is compatible with Bun’s runtime behavior.

## Examples

Force Bun to execute a Node-shebang CLI:

```bash
bun run --bun next dev
bun run --bun next build
```

