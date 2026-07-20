# pm-linker-and-streaming-install

## Why

Bun 1.3.x improved install performance and supply-chain controls. Knowing the linker
modes and install knobs avoids slow monorepo installs and reduces fresh-publish risk.

## Do

- Choose a linker deliberately:
  - `hoisted` (default for single packages) - traditional flat `node_modules`.
  - `isolated` - strict, pnpm-style isolation that prevents phantom (undeclared)
    dependencies and is materially faster in peer-heavy monorepos.
- Set it durably in `bunfig.toml` (see `tooling-bunfig`):
  ```toml
  [install]
  linker = "isolated"
  ```
  or per command: `bun install --linker=isolated`.
- Speed up warm monorepo installs with the experimental **Global Virtual Store**
  (`install.globalStore = true` in `bunfig.toml`, or `BUN_INSTALL_GLOBAL_STORE=1`): with
  the isolated linker, packages materialize once into a global store and each project
  symlinks in (Bun 1.3.14+, off by default).
- Gate freshly published packages with `--minimum-release-age=<seconds>` so you never
  install a version published seconds ago.
- Rely on streaming extraction (default since 1.3.13): `bun install` streams tarballs to
  disk instead of buffering full archives in memory, lowering peak memory on large
  installs.

## Don't

- Don't force `isolated` on a repo that imports phantom dependencies until those imports
  are declared; installs fail fast by design.
- Don't disable streaming extraction unless you hit a verified regression.

## Escape hatch

If a verified streaming-install regression appears:

```bash
BUN_FEATURE_FLAG_DISABLE_STREAMING_INSTALL=1 bun install
```
