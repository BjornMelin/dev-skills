# perf-prefer-bun-native-apis

## Why

Bun provides native APIs that are often faster than Node-compat alternatives (especially
filesystem I/O and HTTP). Using them intentionally yields large wins; Node-compat paths
like `fs/promises` are noticeably slower in hot paths.

## Do

- Prefer `Bun.file()` + `file.text()/json()` for reads, including repeated reads in hot
  paths like request handlers.
- Prefer `Bun.write()` for writes.
- Prefer Bun-native server patterns when your deployment target supports them.
- Keep Node-compat APIs (`node:fs`, `fs/promises`) only at portability boundaries.

## Don't

- Don't use `Bun.serve()` on platforms that don't support it (e.g., Vercel Functions -
  see `vercel-bun-runtime-limitations`).
- Don't repeatedly `readFile` from `fs/promises` inside request handlers without
  profiling.

## Examples

Reads and writes:

```ts
const data = await Bun.file("./data.json").json();
await Bun.write("./out.txt", "hello");
```

Hot path - prefer Bun-native over `fs/promises`:

```ts
// Slower (Node-compat):
import { readFile } from "node:fs/promises";
export async function handler() {
  return await readFile("./data.txt", "utf8");
}

// Faster (Bun-native):
export async function handlerFast() {
  return await Bun.file("./data.txt").text();
}
```
