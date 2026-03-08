# troubleshooting-types-bun

## Why

Type errors like “Cannot find name 'Bun'” or missing `bun:*` module types are usually caused by missing type packages or missing `compilerOptions.types`.

## Do

- Install `@types/bun`.
- Add `compilerOptions.types: ["bun-types"]` when you need Bun globals.
- Restart the TS server after changing types.

## Don't

- Don’t paper over missing types with `any`.

## Example

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

