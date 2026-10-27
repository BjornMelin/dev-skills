# Performance

## Measure the work that actually runs

Preserve the workload: same revision, selected tests, coverage mode, machine budget, and outputs. Record elapsed wall time, peak memory when available, and test/skip counts. Distinguish cold transforms, warm transforms, watch reruns, and Turbo cache hits. Use repeated comparable runs and report a median or range, not a best-run percentage presented as universal.

The summary's phase percentages aggregate tracked worker time, not elapsed wall time. Inspect per-project differences. Optimize the dominant cost:

| Cost | First intervention |
| --- | --- |
| Discovery | Narrow actual include roots/`test.dir`; remove duplicate runner ownership. |
| Environment | Keep non-DOM tests in Node; investigate expensive DOM startup. |
| Transform | Preserve compatible shared servers and correctly invalidated filesystem cache. |
| Import | Replace accidental barrel imports with public direct imports; trim heavy setup graphs. |
| Setup | Move truly run-wide resources to `globalSetup`; pass serializable values through `provide`/`inject`. Keep per-test state independent. |
| Tests | Remove real sleeps, unnecessary network trips, duplicated scenarios, and excessive polling. |

Run `vitest doctor` for an authorized performance investigation. It executes the suite repeatedly under alternative configurations, including isolation checks; do not run it against uncontrolled production side effects. Reproduce its proposed setting under representative conditions. Passing its shuffled runs is evidence, not a proof of independence. Do not run doctor for an ordinary assertion fix.

## Budget parallelism, then isolation

In 5.0.5, default workers use available parallelism minus one in run mode and half in watch mode, with a minimum of one. `maxWorkers` controls file workers; `maxConcurrency` controls concurrent tests within a file. Turbo jobs, worktrees, browser instances, and shards multiply demand. Reserve capacity for the parent Vite process, browser processes, and databases instead of giving every process all cores.

Retain `forks` and isolation initially. Measure `threads` when compatible with process APIs/native dependencies. A small suite can improve with fewer workers or `fileParallelism: false`. CPU-bound synchronous tests do not become parallel by adding `.concurrent`.

Only relax isolation for a bounded suite after checking cleanup, global/module state, timers, order, and representative repeat/shuffle runs. `maxWorkers: 1` is not equivalent to disabling isolation. DOM-heavy suites may benefit from `vmThreads` or `vmForks` while keeping per-file VM contexts, but test cross-realm behavior and memory retention. Do not replace jsdom with a less complete environment solely for a benchmark.

## Three different caches

**Vitest transform cache:** `fsModuleCache` was opt-in in 5.0.0-5.0.3 and defaults on from 5.0.4. It does not accelerate Browser Mode. Inspect the workspace-wide `fsModuleCachePath`; do not mistake results-cache files for transforms. Persist only where restoration costs justify it.

Custom plugin options, environment inputs, or files read outside the transform graph can invalidate correctness. Use the public `configureVitest` cache-key generator for those inputs, or return `false` for an uncacheable module. `ignoreFsModuleCache` is not a fix for missing dependency keys. Compare edited-source/config/fixture runs to uncached execution before trusting a new cache policy. `vitest --clearCache` is diagnostic, not a routine pre-test step.

**Node compile cache:** `NODE_COMPILE_CACHE` caches bytecode, not test results. It helps only when retained across processes. Vitest disables it in workers with V8 coverage to preserve coverage precision.

**Turbo task cache:** replays a completed task, so validate its full dependency/input contract separately.

## Escalate only with evidence

Use import-duration diagnostics, CPU/heap profiles, or async-leak detection for a localized bottleneck. `detectAsyncLeaks`, tracing, and OpenTelemetry add overhead; do not benchmark with them permanently enabled.

`experimental.viteModuleRunner: false` is for a native-runtime-compatible suite, not a global fast preset. It loses Vite transforms/plugins/aliases, `import.meta.env`, Istanbul, and `vi.resetModules`; native hoisting requires supported Node loader hooks. Preserve the default when JSX, framework transforms, or those APIs matter. Any experiment must retain test selection, failure detection, and coverage correctness.

Sources: [performance](https://vitest.dev/guide/improving-performance), [profiling](https://vitest.dev/guide/profiling-test-performance), [cache](https://vitest.dev/config/fsmodulecache), [experimental options](https://vitest.dev/config/experimental), [worker source](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/utils/workers.ts).
