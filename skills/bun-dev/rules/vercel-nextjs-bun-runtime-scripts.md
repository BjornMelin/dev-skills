# vercel-nextjs-bun-runtime-scripts

## Why

When using the Bun runtime on Vercel with Next.js (notably with ISR), Vercel recommends running `next` commands in a way that forces Bun to execute the CLI (since `next` is typically a Node-shebang binary).

## Do

- Update `package.json` scripts to run Next with `bun run --bun ...` when you enable Bun runtime and rely on Next.js features that need it (e.g., ISR).

## Don't

- Don’t assume `next dev`/`next build` will automatically execute under Bun when a binary has a Node shebang.

## Examples

Before:

```json
{
  "scripts": {
    "dev": "next dev",
    "build": "next build"
  }
}
```

After:

```json
{
  "scripts": {
    "dev": "bun run --bun next dev",
    "build": "bun run --bun next build"
  }
}
```

