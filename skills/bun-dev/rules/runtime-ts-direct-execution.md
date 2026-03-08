# runtime-ts-direct-execution

## Why

Bun runs TypeScript directly. Lean on this in Bun-first repos to simplify dev and reduce toolchain surface area.

## Do

- Run TS/JS entrypoints directly:
  - `bun run src/index.ts`
- Pass args normally:
  - `bun run src/server.ts --port 3000`

## Don't

- Don’t add `tsx`/`ts-node` just to run TypeScript if Bun already owns the runtime.

## Examples

```bash
bun run src/server.ts --port 3000
```

