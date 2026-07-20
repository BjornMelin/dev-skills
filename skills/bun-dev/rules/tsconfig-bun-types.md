# tsconfig-bun-types

## Why

Without Bun's type definitions, TypeScript doesn't understand Bun globals (like `Bun`)
or Bun-specific modules (`bun:*`), causing false errors like "Cannot find name 'Bun'"
and poor editor support.

## Do

- Install `@types/bun` as a dev dependency. This is the actual requirement: it provides
  the `Bun` global and `bun:*` module types.
- Leave `compilerOptions.types` **unset** for most projects. TypeScript then auto-includes
  every `@types/*` package (including `@types/bun`), so Bun globals plus DOM/Node ambient
  types all resolve.
- Only scope `compilerOptions.types` when you deliberately restrict ambient types, and
  then keep Bun's types in the list.
- Restart the TS server after changing types.

## Don't

- Don't paper over missing types with `any`.
- Don't scope `compilerOptions.types` to a list that omits Bun's types (it silently drops
  the `Bun` global).

## Examples

Install (the important step):

```bash
bun add -d @types/bun
```

Scoped types (only if you must restrict) - keep Bun in the list:

```json
{
  "compilerOptions": {
    "types": ["bun"]
  }
}
```

> Audit note: the `codex-dev bun audit` engine emits an Info nudge in two cases - when
> `@types/bun` is not installed in a Bun-first repo, or when `compilerOptions.types` is
> scoped to a non-empty array that omits Bun's types. An unset `types` with `@types/bun`
> installed resolves Bun globals and is not flagged.
