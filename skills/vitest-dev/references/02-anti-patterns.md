# Testing anti-patterns to avoid (Vitest)

## 1) Asserting implementation details

Bad:
- asserting internal state, private function calls, or exact DOM structure
- brittle snapshots of huge trees

Better:
- assert externally observable behavior (returned values, emitted events, DOM roles/text)
- use Testing Library queries (`getByRole`, `getByText`) not CSS selectors

## 2) Sleeping (real timers) in tests

Bad:
- `await new Promise(r => setTimeout(r, 1000))`

Better:
- use fake timers:
  - `vi.useFakeTimers()`
  - `vi.runAllTimers()` / `vi.advanceTimersByTime(...)`

See: https://vitest.dev/guide/mocking/timers

## 3) Real network calls

Bad:
- tests depend on internet or real backend availability

Better:
- mock at the boundary
  - stub fetch (`vi.stubGlobal('fetch', ...)`)
  - or use MSW for request-level realism (recommended for component/integration tests)

## 4) Global leakage / order dependence

Symptoms:
- tests fail only when run together
- rerun fixes failures

Fixes:
- restore mocks every test (`vi.restoreAllMocks()`)
- reset modules if needed (`vi.resetModules()`), but avoid doing it globally unless necessary
- disable file parallelism temporarily to debug:
  - `--no-file-parallelism` (https://vitest.dev/config/fileparallelism)

## 5) Over-mocking modules

Bad:
- mocking core modules, “just because”
- huge `vi.mock(...)` objects that replicate real code

Better:
- prefer dependency injection or thin adapter modules you can stub
- use `vi.spyOn` for narrow interaction assertions

## 6) Too much concurrency too early

Bad:
- sprinkling `test.concurrent` everywhere

Better:
- rely on worker parallelism first (`maxWorkers`)
- only use `test.concurrent` when tests are I/O-bound and truly independent

`test.concurrent` is governed by `maxConcurrency` (default 5): https://vitest.dev/config/maxconcurrency

## 7) Running everything in jsdom

Bad:
- backend logic tested in DOM environment → slower, more globals

Better:
- multi-project setup:
  - node project for most logic
  - jsdom project only for UI component tests

See: https://vitest.dev/guide/projects

## 8) VM pools without measuring

Bad:
- using `vmThreads` because “it sounds faster”

Better:
- only adopt VM pools after profiling.
- understand the ESM/memory tradeoffs: https://vitest.dev/config/pool
