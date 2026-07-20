# pm-no-mixed-lockfiles

## Why

Multiple lockfiles (`bun.lock`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`)
create nondeterministic installs and CI/CD drift. Many platforms (including Vercel)
select an install strategy by lockfile presence, so multiple lockfiles can trigger the
wrong toolchain.

This is the canonical lockfile rule. `pm-commit-bun-lockb` and
`vercel-bun-install-detection` defer here for the "exactly one lockfile" policy.

## Do

- Keep exactly one lockfile committed for the repo's chosen package manager.
- For Bun-first repos, commit `bun.lock` (Bun's text lockfile, default since Bun 1.2)
  and delete competing lockfiles.
- Standardize all scripts and docs on `bun install` / `bun run` / `bunx`.

## Don't

- Don't keep `bun.lock` alongside `package-lock.json` / `pnpm-lock.yaml` / `yarn.lock`.
- Don't run `npm install` / `pnpm install` / `yarn install` in a Bun-first repo.

## Legacy: `bun.lockb`

Bun 1.1 and earlier wrote a binary `bun.lockb`. Since Bun 1.2 the default is the text
`bun.lock`. If a repo still carries `bun.lockb`, migrate to the text lockfile:

```bash
bun install --save-text-lockfile --frozen-lockfile --lockfile-only
git rm bun.lockb   # delete the legacy binary lockfile from disk and the index
git add bun.lock
```

## Examples

Bad (mixed lockfiles):

```text
bun.lock
package-lock.json
```

Good (Bun-only):

```text
bun.lock
```
