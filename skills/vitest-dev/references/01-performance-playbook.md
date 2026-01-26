# Performance playbook (Vitest)

This playbook focuses on turning a “works locally” suite into a **fast, scalable** suite on both developer machines and CI.

## Step 1 — Measure first (profiling)

1. Run the slowest subset in CI-like mode:
   - `vitest run`
2. Identify hotspots:
   - slow test files
   - slow global setup
   - expensive transforms and dependency processing
3. Only then change config. Avoid “cargo-cult” tuning.

See: https://vitest.dev/guide/profiling-test-performance

## Step 2 — Choose the right pool

Vitest pools (https://vitest.dev/config/pool):

- `forks` (default): runs tests in `child_process`
- `threads`: runs tests in `worker_threads` (often faster, but no `process.chdir()` and native modules can segfault)
- `vmThreads` / `vmForks`: uses Node’s VM context for speed, but has stability/memory tradeoffs (especially with ESM)

Recommendation:
- Start with `threads` for pure TS/JS unit suites.
- Use `forks` when:
  - native bindings are involved
  - you need process APIs in tests

Example:

```ts
// vitest.config.ts
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    pool: process.env.CI ? 'forks' : 'threads',
  },
})
```

## Step 3 — Isolation, safely

`test.isolate` (https://vitest.dev/config/isolate) defaults to `true`.

- `isolate: true`: strongest protection against global leakage
- `isolate: false`: can improve performance **only if** you have strict discipline

Safe pattern:
- Use projects to keep `jsdom` isolated but allow Node units to disable isolation.

```ts
import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    projects: [
      {
        name: 'unit-node',
        test: {
          environment: 'node',
          isolate: false,
          pool: 'threads',
        },
      },
      {
        name: 'ui-jsdom',
        test: {
          environment: 'jsdom',
          isolate: true,
        },
      },
    ],
  },
})
```

## Step 4 — Concurrency controls

### File-level parallelism

`test.fileParallelism` (https://vitest.dev/config/fileparallelism) controls whether test files run in parallel.
- Turning it off forces `maxWorkers = 1`.

Use it to:
- debug global leakage/order dependence
- reduce resource pressure in CI (rare)

### Worker count

`test.maxWorkers` (https://vitest.dev/config/maxworkers):
- default uses all available parallelism in non-watch mode
- default uses half in watch mode
- accepts percent strings like `"50%"`

Start with:
- local: default
- CI: `"50%"` to reduce memory pressure (adjust after measuring)

### test.concurrent

`test.maxConcurrency` (https://vitest.dev/config/maxconcurrency) defaults to `5` and limits the number of `test.concurrent` tests that can run at once.

Use `test.concurrent` sparingly; most suites get better throughput from file-level parallelism + multiple workers.

## Step 5 — Cache (CI must-have)

Vitest cache config: https://vitest.dev/config/cache

- `cache.enabled` default `true`
- `cache.dir` default `node_modules/.vite/vitest`

In CI:
- persist `node_modules/.vite/vitest` between runs
- key the cache by:
  - lockfile hash
  - Node version
  - OS

## Step 6 — Sharding across CI machines

Vitest’s `blob` reporter stores results per machine so you can merge later (https://vitest.dev/guide/reporters).

Recommended flow:

1. On each CI node:

```bash
npx vitest run --shard=1/4 --reporter=blob --outputFile=reports/blob-1.json
```

2. After all shards finish:

```bash
npx vitest --merge-reports=reports --reporter=default --reporter=json
```

Notes:
- `--reporter=blob` and `--merge-reports` don’t work in watch mode.
- Prefer `vitest run` in CI.

## Step 7 — Reduce per-test overhead

Checklist:
- Avoid heavy `setupFiles` work. They run before each test file (https://vitest.dev/config/setupfiles).
- Move expensive global initialization into `globalSetup` only if truly required.
- Prefer local fakes and dependency injection over module-level mocking in every file.

## Step 8 — Coverage without killing performance

Coverage overview: https://vitest.dev/guide/coverage

- Vitest supports V8 coverage and Istanbul.
- V8 is typically faster and lower-memory, but can be slower when loading many modules and can’t easily limit coverage to specific modules.
- Istanbul is slower due to instrumentation overhead but can be limited to specific files.

Practical approach:
- CI: run coverage in a separate job (or nightly) if it slows the main pipeline too much.
- Use thresholds to prevent regressions, but avoid 100% “coverage theater”.

See thresholds: https://vitest.dev/config/coverage
