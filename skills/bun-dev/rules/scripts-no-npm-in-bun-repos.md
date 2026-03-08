# scripts-no-npm-in-bun-repos

## Why

Mixing `npm`/`pnpm`/`yarn` commands with Bun creates split-brain dependency graphs and lockfile drift. It also makes onboarding and CI unpredictable.

## Do

- Use `bun install` for installs.
- Use `bun run <script>` for package scripts.
- Use `bunx <bin>` for one-off CLIs.

## Don't

- Don’t call `npm`, `pnpm`, `yarn`, or `npx` from `package.json` scripts in Bun-first repos.

## Examples

Bad:

```json
{
  "scripts": {
    "lint": "npm run biome",
    "gen": "npx prisma generate"
  }
}
```

Good:

```json
{
  "scripts": {
    "lint": "bunx biome check .",
    "gen": "bunx prisma generate"
  }
}
```

