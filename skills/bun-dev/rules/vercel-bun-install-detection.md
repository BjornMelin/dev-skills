# vercel-bun-install-detection

## Why

Vercel selects the install strategy based on detected lockfiles. If you want Bun installs on Vercel, commit `bun.lockb` and avoid competing lockfiles.

## Do

- Commit `bun.lockb`.
- Delete other lockfiles (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`).
- Keep `packageManager` consistent (`bun@...`).

## Don't

- Don’t rely on “install command overrides” as the primary mechanism; prefer lockfile-based detection.

## Example

```text
bun.lockb
package.json
```

