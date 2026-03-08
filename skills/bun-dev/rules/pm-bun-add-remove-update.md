# pm-bun-add-remove-update

## Why

Most “Bun migrations” fail because day-to-day dependency operations still use npm/pnpm/yarn. Standardize the common lifecycle commands so installs and lockfiles stay consistent.

## Do

- Install from `package.json`:
  - `bun install`
- Add dependencies:
  - `bun add <pkg>`
  - `bun add -d <pkg>` (dev dependency)
  - `bun add --optional <pkg>`
- Remove dependencies:
  - `bun remove <pkg>`
- Update dependencies:
  - `bun update` (all)
  - `bun update <pkg>` (one)
  - `bun outdated`

## Don't

- Don’t run `npm install`/`pnpm add`/`yarn add` in Bun-first repos.

## Examples

```bash
bun add react@latest
bun add -d typescript @types/bun
bun remove left-pad
bun update --latest
bun outdated
```

