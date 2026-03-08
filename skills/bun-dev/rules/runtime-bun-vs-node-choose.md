# runtime-bun-vs-node-choose

## Why

“Using Bun” can mean:
- Bun as a **package manager** (install/run tools), while runtime remains Node (common for many ecosystems).
- Bun as the **runtime** (server, scripts, tests, bundling).

Conflating the two leads to subtle breakages and duplicated runtime requirements.

## Do

- Decide explicitly:
  - **Bun-first runtime**: run servers/tests/scripts with Bun; avoid Node-only assumptions.
  - **Node runtime + Bun package manager**: keep Node where required, but standardize on Bun for installs and scripts.
- Document the choice in `package.json` (`packageManager`) and CI config.

## Don't

- Don’t keep multiple runtimes “just in case” without a reason.
- Don’t mix package managers even if you keep Node as a runtime.

## Examples

Bun package manager, Node runtime (typical):

```bash
bun install
bunx eslint .
node dist/server.js
```

Bun runtime:

```bash
bun run src/server.ts
bun test
```

