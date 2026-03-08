# build-bun-build-bundler

## Why

`bun build` provides a fast bundler for JS/TS, can target browser/node/bun, and supports minification/sourcemaps/splitting. Bundling reduces runtime overhead and simplifies deployments.

## Do

- Use `bun build` for production bundles where appropriate.
- Choose a `--target` that matches the deployment runtime (`bun`, `node`, `browser`).
- Use `--minify` and `--sourcemap` for production builds as needed.
- Use `bun build --compile --target=browser` only for HTML entrypoints that must be fully inlined.

## Don't

- Don’t bundle server frameworks that expect filesystem access or dynamic requires unless you verify runtime behavior.

## Examples

```bash
bun build ./src/index.ts --outdir ./dist --target=bun --minify --sourcemap
```

Build API:

```ts
const result = await Bun.build({
  entrypoints: ["./src/index.ts"],
  outdir: "./dist",
  target: "bun",
  minify: true,
  sourcemap: "external",
});

if (!result.success) console.error(result.logs);
```
