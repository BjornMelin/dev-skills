# tsconfig-bun-recommended

## Why

Bun can run TypeScript directly, but TypeScript tooling still needs correct compiler options. Bun’s recommended `tsconfig.json` reduces ESM/CJS surprises and makes resolution behavior predictable.

## Do

- Start from `bun init` (it generates a Bun-friendly tsconfig) and adjust minimally.
- Prefer:
  - `target: "ESNext"`
  - `module: "Preserve"`
  - `moduleResolution: "Bundler"`
  - `verbatimModuleSyntax: true`
  - `noEmit: true`

## Don't

- Don’t use legacy `moduleResolution` modes unless you have a specific requirement.

## Example

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "Preserve",
    "moduleResolution": "Bundler",
    "verbatimModuleSyntax": true,
    "allowImportingTsExtensions": true,
    "noEmit": true,
    "strict": true
  }
}
```

