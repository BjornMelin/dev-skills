# test-bun-test-runner

## Why

`bun test` is Bun's built-in test runner. Using it avoids extra runtime/tooling overhead
and keeps the toolchain consistent. Bun 1.3.x adds scale-oriented flags for large suites
and CI.

## Do

- Use `bun test` for running tests; `--watch` for iterative TDD; `--coverage` for
  coverage.
- Filter by test name with `-t` / `--test-name-pattern "<pattern>"`.
- Use `--retry <n>` for a global retry budget in CI (see `test-bun-retry`).
- Scale large suites and CI with the 1.3.x flags:
  - `--isolate` runs each test file in a fresh global object in the same process.
  - `--parallel[=N]` distributes files across worker processes (implies isolation);
    `--parallel-delay=<ms>` ramps workers up.
  - `--shard=M/N` splits files deterministically across CI jobs.
  - `--changed[=<ref>]` runs only test files affected by git changes.

## Don't

- Don't add Jest/Vitest by default in Bun-first repos unless you need ecosystem-specific
  features.
- Don't rewrite a mature Vitest/Jest/Playwright suite just to adopt these flags; plan a
  separate migration.

## Examples

```bash
bun test
bun test --watch
bun test -t "adds"
bun test --retry 3 --coverage
```

Scale in CI (shard 3 of 4, only changed files, isolated parallel workers):

```bash
bun test --shard=3/4
bun test --changed
bun test --parallel --coverage
```

Minimal test:

```ts
import { expect, test } from "bun:test";

test("adds", () => {
  expect(1 + 1).toBe(2);
});
```
