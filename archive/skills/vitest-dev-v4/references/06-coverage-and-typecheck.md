# Coverage + Typechecking reference

## Coverage

Guide: https://vitest.dev/guide/coverage  
Config: https://vitest.dev/config/coverage

Key points:
- Coverage providers:
  - `v8` (default; fast, low memory; V8-only runtimes)
  - `istanbul` (instrumented; slower; works everywhere)
- Enable coverage:
  - CLI: `vitest run --coverage`
  - Config: `test.coverage.enabled = true`
- Include uncovered files by setting `coverage.include` to match your source globs.
- Thresholds can be expressed as:
  - positive: minimum percent required (e.g. `90`)
  - negative: max uncovered items allowed (e.g. `-10`)
  See thresholds: https://vitest.dev/config/coverage

## Typechecking (type tests)

Guide: https://vitest.dev/guide/testing-types  
Config: https://vitest.dev/config/typecheck

Key points:
- Type tests are files like `*.test-d.ts` by default.
- Vitest runs `tsc` (or `vue-tsc`) under the hood and parses output.
- Flags:
  - `--typecheck` / `typecheck.enabled`
  - `--typecheck.only` to run only typecheck tests
- `typecheck.ignoreSourceErrors` can be used to ignore non-test source errors (use carefully).
