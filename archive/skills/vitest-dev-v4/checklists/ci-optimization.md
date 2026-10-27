# CI optimization checklist (Vitest)

## Mode

- [ ] CI uses `vitest run` (single-run, non-interactive)
- [ ] Watch mode is not relied upon

## Parallelism

- [ ] `fileParallelism` is enabled unless debugging
- [ ] `maxWorkers` tuned for CI resource limits (memory/cpu)
- [ ] Optional: sharding used for multi-machine CI

## Caching

- [ ] Persist `node_modules/.vite/vitest` between CI runs (Vitest cache)

## Reporting

- [ ] JUnit XML output configured when needed
- [ ] If using sharding:
  - [ ] `blob` reporter used per shard
  - [ ] `--merge-reports` run in a final aggregation step

## Coverage

- [ ] Coverage provider selected intentionally (v8 vs istanbul)
- [ ] Thresholds set intentionally (avoid “coverage theater”)

## Flake detection (optional)

- [ ] High-risk tests are run multiple times in CI nightly or on demand
