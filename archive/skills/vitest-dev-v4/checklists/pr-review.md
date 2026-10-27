# Test code review checklist

## Design

- [ ] Tests assert behavior, not implementation details
- [ ] Each test name describes the scenario and expected outcome
- [ ] Avoids over-mocking; seams are mocked, core logic is real

## Determinism

- [ ] No real network calls (unless explicitly an integration test)
- [ ] No real sleeps (`setTimeout` waits); uses fake timers if time-based
- [ ] No reliance on test order

## Hygiene

- [ ] Mocks restored in `afterEach`
- [ ] Globals/env stubs are cleaned up
- [ ] Test fixtures are minimal and readable

## Performance

- [ ] Avoids heavy global setup per test file
- [ ] Uses the lightest environment possible (`node` vs `jsdom`)
- [ ] Does not introduce excessive `test.concurrent` usage without reason

## CI readiness

- [ ] Passes with `vitest run`
- [ ] Reports/artifacts are produced as required (JUnit/JSON/blob)
- [ ] Coverage config is intentional (no accidental slowdown)
