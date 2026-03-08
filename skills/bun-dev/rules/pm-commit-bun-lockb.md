# pm-commit-bun-lockb

## Why

`bun.lockb` is the source of truth for deterministic installs. If it’s missing (or ignored), dependency resolution can change silently between machines and CI runs.

## Do

- Commit `bun.lockb`.
- Use `bun install --frozen-lockfile` in CI to guarantee installs match the lockfile.
- Ensure `.gitignore` does **not** ignore `bun.lockb`.

## Don't

- Don’t ignore `bun.lockb`.
- Don’t regenerate lockfiles with other package managers.

## Examples

CI install:

```bash
bun install --frozen-lockfile
```

