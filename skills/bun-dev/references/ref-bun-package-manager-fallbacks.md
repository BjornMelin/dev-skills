# Package Manager Fallback Lanes

Use this when a repo is not fully Bun-first.

## Canonical postures

### 1. Bun-first repo

- `packageManager` is `bun@...`
- `bun.lock` or `bun.lockb` is canonical
- install / run / test / build commands are Bun-first

### 2. Node runtime + Bun package manager

Use this when framework or hosting still requires Node runtime semantics.

Do:

- keep Bun for install + script entrypoints
- keep Node runtime explicit where required
- keep one lockfile

Example:

```bash
bun install
bun run build
node dist/server.js
```

### 3. Non-Bun repo under evaluation

Do not rewrite command surfaces blindly.

Do:

- inspect existing manager first: npm, pnpm, or Yarn
- map Bun migration risk before editing lockfiles or CI
- prefer advisory guidance unless the task explicitly requests migration

## Mapping guide

| Intent | Bun | npm | pnpm | Yarn |
| --- | --- | --- | --- | --- |
| install deps | `bun install` | `npm install` | `pnpm install` | `yarn install` |
| add dep | `bun add x` | `npm install x` | `pnpm add x` | `yarn add x` |
| remove dep | `bun remove x` | `npm uninstall x` | `pnpm remove x` | `yarn remove x` |
| run script | `bun run build` | `npm run build` | `pnpm build` | `yarn build` |
| exec package bin | `bunx vite` | `npx vite` | `pnpm dlx vite` | `yarn dlx vite` |
| audit | `bun audit` or `bun pm scan` | `npm audit` | `pnpm audit` | `yarn npm audit` |

## Hard rules

- No mixed lockfiles.
- No dual install instructions unless user explicitly asks for multi-manager docs.
- If Bun is only the package manager, do not introduce Bun-only runtime APIs.
- If runtime is Bun, prefer Bun-native APIs before extra packages.
