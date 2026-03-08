# perf-avoid-node-fs-promises-hot-paths

## Why

Node-compat filesystem APIs (`fs/promises`) can be slower than Bun’s native file APIs, especially in hot paths.

## Do

- Prefer `Bun.file()` for repeated reads.
- Keep Node-compat code for portability boundaries only.

## Don't

- Don’t repeatedly `readFile` from `fs/promises` inside request handlers without profiling.

## Examples

Bad:

```ts
import { readFile } from "node:fs/promises";

export async function handler() {
  return await readFile("./data.txt", "utf8");
}
```

Good:

```ts
export async function handler() {
  return await Bun.file("./data.txt").text();
}
```

