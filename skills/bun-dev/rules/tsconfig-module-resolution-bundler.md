# tsconfig-module-resolution-bundler

## Why

`moduleResolution: "Bundler"` matches modern ESM + bundler-style resolution and aligns
with Bun's behavior better than legacy Node resolution modes.

See `tsconfig-bun-recommended` for the full Bun-friendly tsconfig baseline.

## Do

- Set `compilerOptions.moduleResolution: "Bundler"` in Bun-first TS projects.
- Pair it with `verbatimModuleSyntax: true` for consistent import/export elision.

## Don't

- Don't default to `node16` / `nodenext` unless you specifically need Node's conditional
  exports semantics in TS.

## Example

```json
{
  "compilerOptions": {
    "module": "Preserve",
    "moduleResolution": "Bundler",
    "verbatimModuleSyntax": true
  }
}
```
