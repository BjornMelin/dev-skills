# CI and execution

## Own execution once

Use repository scripts and pinned local binaries. `bun run test` executes the script; `bun test` is Bun's different test runner. Bun normally respects a CLI's Node shebang, while `--bun` or runtime configuration can override it. Confirm the runtime actually used before attributing a failure or claiming a speedup. Bun as package manager does not require Vitest to execute in Bun. Avoid `bunx`/`npx` silently downloading a missing runner during validation.

Choose package-owned Turbo test tasks or a root Vitest projects runner for a given suite, not both. Root discovery must exclude Playwright/native/benchmark files belonging to other lanes. A changed package should not accidentally launch every workspace's tests again.

## Cache complete inputs

Turbo can cache deterministic tests only when their hash accounts for imported workspace source, configuration, fixtures, generated inputs, and behavior-affecting environment/runtime identity. Preserve default inputs with `$TURBO_DEFAULT$` when adding custom inputs. Declare real report outputs, not invented files. An empty `outputs` list still permits log/result caching.

For source-consuming package tests that need dependency invalidation without serializing dependency test execution, use a scriptless transit node if the repo lacks an equivalent relationship:

```json
{
  "tasks": {
    "test-inputs": { "dependsOn": ["^test-inputs"] },
    "test": { "dependsOn": ["test-inputs"] },
    "test:watch": { "cache": false, "persistent": true }
  }
}
```

This assumes `test-inputs` is not a package script. It is a task-graph fragment, not a replacement for the existing config. Compiled consumers need their actual build dependency instead. Confirm with a cache miss after changing an upstream imported file, and a hit without input changes. Keep live service contracts, side-effecting tasks, and timing benchmarks uncached. Budget Turbo concurrency together with Vitest workers.

## Select and shard safely

Use file/name/project filters for local iteration. `--changed`/`related` need a valid Git base and a complete dependency model; external fixture reads and configuration changes may require broader execution. `vitest list` is static by default in v5; use it for discovery, not a passing-test claim.

`experimental.recordDependencies` exists from 5.0.4 and can avoid repeated graph analysis, but records describe prior executions. Refresh after import changes; filesystem reads and externalized module internals are not fully tracked. Keep an appropriate full-suite gate. `experimental.preParse` can help static targeted names/tags, but generated names can be missed and full runs can slow down. Neither option is a universal CI default.

Shard files with `vitest run --shard=1/3 --reporter=blob`; merge collected blobs with the intended reporters. File count does not equal balanced duration. Preserve all shards and attachments, include hidden `.vitest` files in uploads, and use distinct artifact names for each matrix/shard. Never treat partial artifacts as a complete green run. Merge coverage using compatible provider/configuration across shards. Do not run the full suite once more merely to produce a second report.

## Keep evidence lanes distinct

Use V8 coverage for supported V8 runtimes; use Istanbul when needed for other engines. Set meaningful source inclusion and inspect uncovered files, not only a percentage. Coverage is diagnostic, not proof of good assertions; retain critical branch/permission/failure tests. Do not instrument every local rerun or narrow coverage to touched files while claiming a whole-project gate. Subprocess coverage is opt-in and must be verified if child processes contain the behavior.

Run the repository's typechecker or enabled Vitest type tests separately from transpilation. Do not duplicate identical type checks across orchestration layers or ignore source errors to achieve green results.

For microbenchmarks, use the v5 test-context `bench` fixture in a benchmark-matched file, call `.run()` or `bench.compare()`, and run `vitest bench --run`. The removed top-level `bench` import is not compatible. Benchmark representative workloads independently from coverage and loaded CI workers; use distributions and baselines rather than fragile single-run timing assertions.

Sources: [Turbo task graphs](https://turborepo.dev/docs/crafting-your-repository/configuring-tasks), [Bun execution](https://bun.com/docs/runtime), [CLI](https://vitest.dev/guide/cli), [coverage](https://vitest.dev/guide/coverage), [benchmarks](https://vitest.dev/guide/benchmarking), [type tests](https://vitest.dev/guide/testing-types).
