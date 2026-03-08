# runtime-env-files

## Why

Environment configuration is a frequent source of “works locally, fails in CI”. Bun supports `.env` loading and an explicit `--env-file` flag.

## Do

- Use `.env` for local development defaults.
- Use `bun --env-file=<path> run ...` for explicit environment selection.
- Prefer `Bun.env` for Bun-first code (and `process.env` for Node compatibility).

## Don't

- Don’t rely on ambient shell state when you need a repeatable run.

## Examples

```bash
bun --env-file=.env.production run src/server.ts
```

```ts
const port = Bun.env.PORT ?? "3000";
```

