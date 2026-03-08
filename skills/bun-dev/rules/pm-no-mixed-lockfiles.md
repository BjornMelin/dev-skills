# pm-no-mixed-lockfiles

## Why

Multiple lockfiles (`bun.lockb`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`) create nondeterministic installs and CI/CD drift. Many platforms (including Vercel) select an install strategy by lockfile presence; multiple lockfiles can trigger the wrong toolchain.

## Do

- Keep exactly one lockfile committed for the repo’s chosen package manager.
- For Bun-first repos, commit `bun.lockb` and delete other lockfiles.
- Standardize all scripts and docs on `bun install` / `bun run` / `bunx`.

## Don't

- Don’t keep `bun.lockb` alongside `package-lock.json`/`pnpm-lock.yaml`/`yarn.lock`.
- Don’t run `npm install`/`pnpm install`/`yarn install` in a Bun-first repo.

## Examples

Bad (mixed lockfiles):

```text
bun.lockb
package-lock.json
```

Good (Bun-only):

```text
bun.lockb
```

