# pm-bun-install-ci-frozen-lockfile

## Why

CI should be deterministic: dependency versions must come from the lockfile, not from whatever the registry serves at build time.

## Do

- Use `bun install --frozen-lockfile` in CI.
- Fail fast if the lockfile is out of sync.

## Don't

- Don’t use a non-frozen install in CI unless you explicitly want dependency drift.

## Examples

```bash
bun install --frozen-lockfile
```

