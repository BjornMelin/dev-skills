# migrate-node-to-bun-checklist

## Why

Node -> Bun migrations are usually straightforward, but toolchain drift (lockfiles, scripts, TS config) is the most common failure mode.

## Do

- Install/upgrade Bun.
- Remove Node package manager lockfiles and install artifacts.
- Install dependencies with Bun and commit `bun.lockb`.
- Update scripts to use `bun run` / `bunx`.
- Add Bun types (`@types/bun`) and confirm TS config.
- Run your full verification suite.

## Don't

- Don’t keep multiple lockfiles “for compatibility”.
- Don’t assume every Node package behaves identically under Bun without running tests.

## Checklist

```bash
rm -rf node_modules
rm -f package-lock.json pnpm-lock.yaml yarn.lock
bun install
bun add -d @types/bun
bun test
```

