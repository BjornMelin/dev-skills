# build-bun-compile-browser

## Why

Bun v1.3.10 can compile HTML entrypoints into self-contained `.html` output.

Use this when you need offline/disconnected delivery or a single-file artifact for sharing UI demos.

## Do

- Use `bun build --compile --target=browser` only with `.html` entrypoints.
- Keep asset-heavy pages in mind because everything is inlined.
- Prefer `bun build` + regular `--target=browser` for multi-file apps that should keep runtime chunking.

## Don't

- Don’t use `--splitting` with this mode; it is not supported.
- Don’t rely on non-local asset loading assumptions, because browser output is inlined.

## Examples

CLI self-contained HTML:

```bash
bun build --compile --target=browser ./index.html --outfile ./dist/index.html
```

Programmatic equivalent:

```ts
await Bun.build({
  entrypoints: ["./index.html"],
  target: "browser",
  compile: true,
  outdir: "./dist",
});
```

