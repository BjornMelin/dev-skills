# Bun CLI Cheatsheet (Bun-First Repos)

This is a quick reference for day-to-day Bun development. Prefer `rules/` for “what to do” and “what not to do”.

## Install / Upgrade Bun

```bash
# Upgrade Bun in-place
bun upgrade

# Check version
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
# Install deps from package.json
bun install

# Deterministic install (CI)
bun install --frozen-lockfile

# Add / remove
bun add <pkg>
bun add -d <pkg>
bun remove <pkg>

# Update / outdated
bun update
bun update <pkg>
bun outdated

# Tool runner (npx equivalent)
bunx <bin> [...args]
```

## Run Code

```bash
# Run a package.json script
bun run dev

# Run an entrypoint directly (TS/JS)
bun run src/index.ts

# Watch / hot reload
bun --watch run src/server.ts
bun --hot run src/server.ts

# Explicit env file
bun --env-file=.env.production run src/server.ts
```

## Monorepos (Workspaces)

```bash
# Run in all workspace packages
bun run --workspaces test

# Filter packages
bun run --filter \"packages/*\" build

# Parallel / sequential
bun run --parallel --workspaces lint typecheck
bun run --sequential --workspaces build

# Keep going if one fails
bun run --parallel --no-exit-on-error --workspaces test
```

## Testing

```bash
bun test
bun test --watch
bun test --coverage
bun test --grep \"pattern\"
```

## Bundling / Build

```bash
bun build ./src/index.ts --outdir ./dist --target=bun --minify --sourcemap

# Compile to an executable (CLI/service)
bun build ./src/cli.ts --compile --outfile mycli
```

## Vercel Bun Runtime (Functions)

Enable Bun runtime:

```json
{
  \"$schema\": \"https://openapi.vercel.sh/vercel.json\",
  \"bunVersion\": \"1.x\"
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

Next.js with Bun runtime (notably ISR):

```json
{
  \"scripts\": {
    \"dev\": \"bun run --bun next dev\",
    \"build\": \"bun run --bun next build\"
  }
}
```

