# pm-bun-install-ci-frozen-lockfile

## Why

CI should be deterministic: dependency versions must come from the lockfile, not from
whatever the registry serves at build time.

## Do

- Use `bun ci` in CI. It is the reproducible spelling and is equivalent to
  `bun install --frozen-lockfile`.
- Fail fast if the lockfile is out of sync.

## Don't

- Don't use a non-frozen install in CI unless you explicitly want dependency drift.

## Examples

Preferred:

```bash
bun ci
```

Equivalent long form:

```bash
bun install --frozen-lockfile
```
