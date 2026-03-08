# test-bun-retry

## Why

`bun test` in Bun v1.3.10 supports a global retry count for flaky environments.

Use retries for CI stability without per-test duplication.

## Do

- Start with `bun test --retry <count>` in CI jobs.
- Keep suite-level retry low and prefer fixing flaky behavior before raising it.
- Use per-test `{ retry: N }` only when behavior differs from the suite default.

## Don't

- Don’t use retries to mask deterministic failures.
- Don’t set a very high default retry globally and skip root-cause analysis.

## Examples

Global retry for CI:

```bash
bun test --retry 3 --coverage
```

Override globally with per-test options:

```ts
import { test } from "bun:test";

test(
  "very flaky test",
  { retry: 5 },
  () => {
    // ...
  },
);
```
