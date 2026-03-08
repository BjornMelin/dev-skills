# tsconfig-bun-types

## Why

Without Bun’s type definitions, TypeScript won’t understand Bun globals (like `Bun`) and Bun-specific modules (`bun:*`), causing false errors and poor editor support.

## Do

- Install `@types/bun` as a dev dependency.
- Add `"types": ["bun-types"]` to `compilerOptions` when you need Bun globals in TS.

## Don't

- Don’t rely on implicit global types being present.

## Examples

```bash
bun add -d @types/bun
```

```json
{
  "compilerOptions": {
    "types": ["bun-types"]
  }
}
```

