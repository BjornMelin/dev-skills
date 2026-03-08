# test-bun-test-runner

## Why

`bun test` is Bun’s built-in test runner. Using it avoids extra runtime/tooling overhead and keeps the toolchain consistent.

## Do

- Use `bun test` for running tests.
- Use `--watch` for iterative TDD.
- Use `--coverage` when you need coverage reporting.
- Use `--retry` when you need a global retry budget in CI.

## Don't

- Don’t add Jest/Vitest by default in Bun-first repos unless you need ecosystem-specific features.

## Examples

Run all tests:

```bash
bun test
```

Watch mode:

```bash
bun test --watch
```

Retry in CI:

```bash
bun test --retry 3 --coverage
```

Minimal test:

```ts
import { expect, test } from "bun:test";

test("adds", () => {
  expect(1 + 1).toBe(2);
});
```
