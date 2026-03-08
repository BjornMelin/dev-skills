# tsconfig-module-resolution-bundler

## Why

`moduleResolution: "Bundler"` matches modern ESM + bundler-style resolution and aligns better with Bun’s behavior than legacy Node resolution modes.

## Do

- Prefer `compilerOptions.moduleResolution: "Bundler"` in Bun-first TS projects.
- Use `verbatimModuleSyntax: true` to keep imports/exports consistent.

## Don't

- Don’t default to `node16`/`nodenext` unless you specifically need Node’s conditional exports semantics in TS.

## Example

```json
{
  "compilerOptions": {
    "moduleResolution": "Bundler",
    "verbatimModuleSyntax": true
  }
}
```

