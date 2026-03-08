# pm-bunx-vs-npx

## Why

`bunx` is Bun’s tool runner (analogous to `npx`). Using `bunx` keeps repos Bun-first and avoids pulling in a Node/npm toolchain for ad-hoc CLIs.

## Do

- Prefer `bunx <pkg>` over `npx <pkg>`.
- Pin versions for critical tooling runs: `bunx <pkg>@<version> ...`.

## Don't

- Don’t use `npx` in Bun-first repos unless you intentionally require Node’s npx behavior.

## Examples

```bash
bunx biome check .
bunx tsc -p tsconfig.json
bunx typescript@5.7.3 tsc --version
```

