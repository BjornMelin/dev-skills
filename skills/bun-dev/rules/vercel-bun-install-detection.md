# vercel-bun-install-detection

## Why

Vercel selects the install strategy based on detected lockfiles. To get Bun installs on
Vercel, commit `bun.lock` and avoid competing lockfiles.

See `pm-no-mixed-lockfiles` for the canonical one-lockfile policy.

## Do

- Commit `bun.lock` (Bun's text lockfile, default since Bun 1.2).
- Delete other lockfiles (`package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`).
- Keep `packageManager` consistent (`bun@...`).

## Don't

- Don't rely on "install command overrides" as the primary mechanism; prefer
  lockfile-based detection.

## Example

```text
bun.lock
package.json
```
