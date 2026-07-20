# tsconfig-bun-types

## Why

Without Bun's type definitions, TypeScript doesn't understand Bun globals (like `Bun`)
or Bun-specific modules (`bun:*`), causing false errors like "Cannot find name 'Bun'"
and poor editor support.

## Do

- Install `@types/bun` as a dev dependency. This is the requirement: it provides the
  `Bun` global and `bun:*` module types.
- Explicitly list Bun in `compilerOptions.types`: `"types": ["bun"]`. This is what
  `bun init` writes, and it works on every TypeScript version.
- If you set `types`, include every ambient package you rely on (for example
  `["bun", "node"]`) - a `types` array replaces auto-inclusion.
- Restart the TS server after changing types.

## Don't

- Don't paper over missing types with `any`.
- Don't leave `compilerOptions.types` unset and assume Bun globals resolve. TypeScript
  5 and earlier auto-include `node_modules/@types/*` when `types` is unset, but
  **TypeScript 6 defaults `types` to `[]`** and no longer auto-includes anything - so set
  it explicitly.
- Don't scope `compilerOptions.types` to a list that omits Bun's types (it drops the
  `Bun` global).

## Examples

Install and configure (both steps matter):

```bash
bun add -d @types/bun
```

```json
{
  "compilerOptions": {
    "types": ["bun"]
  }
}
```

> Audit note: the `codex-dev bun audit` engine emits an Info nudge when `@types/bun` is
> not installed in a Bun-first repo, or when `compilerOptions.types` does not list Bun
> (unset, `[]`, or an array without `"bun"` / `"bun-types"`). An unset array is flagged
> because it is unsafe under TypeScript 6.
