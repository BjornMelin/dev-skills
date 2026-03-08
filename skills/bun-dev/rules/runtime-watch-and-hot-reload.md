# runtime-watch-and-hot-reload

## Why

Fast feedback loops are one of Bun’s biggest advantages. Use built-in watch/hot modes instead of external file watchers when possible.

## Do

- Auto-restart on file changes:
  - `bun --watch run src/server.ts`
- Use hot reloading when appropriate:
  - `bun --hot run src/server.ts`

## Don't

- Don’t add `nodemon` by default in Bun-first repos.

## Examples

```bash
bun --watch run src/index.ts
```

