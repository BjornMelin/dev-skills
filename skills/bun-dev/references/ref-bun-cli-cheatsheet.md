# Bun CLI Cheatsheet (Bun-First Repos)

This is a quick reference for day-to-day Bun development. Prefer `rules/` for "what to do"
and "what not to do".

## Install / Upgrade Bun

```bash
bun upgrade         # upgrade Bun in-place
bun --version
```

## New Project

```bash
bun init

# Templates
bun create react my-app
bun create next my-app
bun create vite my-app
```

## Package Management

```bash
bun install                      # install deps from package.json (writes bun.lock)
bun ci                           # deterministic CI install (frozen lockfile)
bun install --linker=isolated    # strict, pnpm-style isolation (monorepos)

bun add <pkg>
bun add -d <pkg>
bun remove <pkg>

bun update
bun update <pkg>
bun outdated

bun audit                        # dependency security advisories
bunx <bin> [...args]             # npx equivalent
```

## Run Code

```bash
bun run dev                      # run a package.json script
bun run src/index.ts             # run a TS/JS entrypoint directly

bun --watch run src/server.ts
bun --hot run src/server.ts

bun --env-file=.env.production run src/server.ts
```

## Monorepos (Workspaces)

```bash
bun run --workspaces test        # run in all workspace packages
bun run --filter "packages/*" build

bun run --parallel --workspaces lint typecheck
bun run --sequential --workspaces build
bun run --parallel --no-exit-on-error --workspaces test
```

## Testing

```bash
bun test
bun test --watch
bun test --coverage
bun test -t "pattern"            # filter by test name (--test-name-pattern)

# Scale (Bun 1.3.x)
bun test --isolate
bun test --parallel
bun test --shard=3/4
bun test --changed
```

## Bundling / Build

```bash
bun build ./src/index.ts --outdir ./dist --target=bun --minify --sourcemap

# Compile to an executable (CLI/service)
bun build ./src/cli.ts --compile --outfile mycli
```

## Vercel Bun Runtime (Functions)

Enable the Bun runtime (Beta):

```json
{
  "$schema": "https://openapi.vercel.sh/vercel.json",
  "bunVersion": "1.x"
}
```

Write a Bun Function:

```ts
export default {
  async fetch(req: Request) {
    return Response.json({ ok: true });
  },
};
```

Next.js with the Bun runtime (notably ISR):

```json
{
  "scripts": {
    "dev": "bun run --bun next dev",
    "build": "bun run --bun next build"
  }
}
```
