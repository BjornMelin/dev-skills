# perf-prefer-bun-native-apis

## Why

Bun provides native APIs that are often faster than Node-compat alternatives (especially filesystem I/O and HTTP). Using them intentionally can yield large wins.

## Do

- Prefer `Bun.file()` + `file.text()/json()` for reads.
- Prefer `Bun.write()` for writes.
- Prefer Bun-native server patterns (when your deployment target supports them).

## Don't

- Don’t use `Bun.serve()` on platforms that don’t support it (e.g., Vercel Functions).

## Examples

```ts
const file = Bun.file("./data.json");
const data = await file.json();
```

```ts
await Bun.write("./out.txt", "hello");
```

