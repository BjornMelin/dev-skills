# pm-package-manager-field

## Why

The `packageManager` field in `package.json` makes the intended package manager explicit for humans, CI, and tooling.

## Do

- Set `"packageManager": "bun@<version>"` for Bun-first repos.
- Keep it aligned with the Bun version used in CI/build images.

## Don't

- Don’t leave `packageManager` unset in repos where consistency matters.
- Don’t set it to a different package manager while expecting Bun workflows.

## Examples

```json
{
  "packageManager": "bun@1.3.10"
}
```
