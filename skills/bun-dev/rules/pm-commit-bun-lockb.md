# pm-commit-bun-lockb

## Why

The Bun lockfile is the source of truth for deterministic installs. If it is missing
(or git-ignored), dependency resolution can change silently between machines and CI
runs.

Rule id kept for engine compatibility. Bun's current lockfile is the text `bun.lock`
(default since Bun 1.2); `bun.lockb` is the legacy binary format. See
`pm-no-mixed-lockfiles` for the one-lockfile policy and `pm-bun-install-ci-frozen-lockfile`
for the CI install.

## Do

- Commit `bun.lock` (or a legacy `bun.lockb` if you have not migrated yet).
- Ensure `.gitignore` does **not** ignore `bun.lock` / `bun.lockb`.
- Verify installs match the lockfile in CI with `bun ci`.

## Don't

- Don't git-ignore the Bun lockfile.
- Don't regenerate lockfiles with other package managers.

## Examples

Reproducible CI install:

```bash
bun ci
```
