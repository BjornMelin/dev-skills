# Test patterns

## Exercise real behavior

Start from the requirement or regression, not the implementation's branch inventory. Prove observable values, persistence, state transitions, permission boundaries, or an external protocol. Prefer a focused integration test over units that mock every owned collaborator. A useful test should fail for a plausible broken implementation; do not add tests that merely restate fixtures or exercise the mock framework.

Use a small scenario matrix for materially different outcomes, not an exhaustive cross-product. `test.for` preserves context and works with table cases; retain `test.each` when it is already clear. Keep names readable and statically analyzable when using static selection. Snapshots are suitable for a deliberate stable contract, not whole internal objects or blanket output approval.

## Own resources and asynchronous work

Await `.resolves`, `.rejects`, polling, file snapshots, and browser actions. Assert promise rejection directly instead of relying on a `catch` that might never run. Use assertion counts when callback execution is the contract, not as ceremony in every test.

Use `expect.poll` for a repeatedly observed value and `vi.waitFor` for a condition that succeeds without throwing. Keep the poll callback free of repeated mutations. Bound its timeout and pass cancellation to in-flight work where supported. `vi.waitUntil` waits for truthiness but propagates thrown errors instead of retrying them.

Register cleanup immediately after acquiring a resource. Use test-context `onTestFinished`, existing lifecycle hooks, or native fixture teardown. For genuinely repeated setup, `test.extend` fixtures or `aroundEach` can replace a custom resource manager. Concurrent tests use their context's `expect` and cleanup hooks; no shared fake clock, mutable singleton, or global spy cleanup that races another test.

Use `using`/`await using` only when the actual runtime/transform pipeline supports explicit resource management. A disposable spy or `vi.when` chain restores its scoped behavior without a custom helper. Otherwise use explicit cleanup. Fixture scope must match ownership: sharing a connection pool does not justify sharing mutable test data.

## Mock the boundary, preserve its type

Prefer a real provider-supported harness when feasible. Otherwise use a typed `vi.fn<typeof externalFunction>()`, `vi.spyOn` on an appropriate mutable object, or an existing network interceptor. Retain actual request construction, parsing, and business logic. `vi.mocked` supplies typing; it does not turn a real function into a runtime mock.

`vi.when(mock, { onUnmatched: 'throw' })` provides argument-specific `.calledWith(...).thenReturn/thenResolve/thenReject` behavior without a custom dispatcher. Choose strict unmatched behavior when unexpected calls are bugs; the default passes through. Do not install `jest-when` for this. Keep the mock local or restore the chain; automatic history clearing does not reset implementations. Repeated registrations for the same arguments use the latest behavior first.

Place hoisted module mocks at top level. Prefer the typed module-promise overload for a needed factory; use `vi.doMock` with a subsequent dynamic import for genuinely dynamic mocking. Do not mock internal modules to bypass the behavior under test. Avoid repeated `vi.resetModules` unless module initialization itself is the contract.

Fake timers should advance the specific behavior with async advancement when promise callbacks matter, then restore real timers. Do not fake `nextTick` in fork workers. `vi.setSystemTime` affects Date and available global Temporal; it does not execute timers. Keep unrelated real-I/O tests on real timers.

## Keep types and lint meaningful

Use `expect.assert` for runtime checks that also narrow TypeScript. Reuse existing Standard Schema contracts through `expect.schemaMatching`, but still assert business semantics; do not duplicate or invent a schema just for a test. Type assertions require the type-checking lane, not merely successful transpilation.

Extend the existing linter, scoped to Vitest files. Native Oxlint rules include `vitest/no-focused-tests`, `vitest/valid-expect`, `vitest/valid-expect-in-promise`, and `vitest/no-conditional-expect`. For `valid-expect`, consider `{ "alwaysAwait": true, "maxArgs": 2 }` to require awaits while permitting Vitest's assertion message. Verify recognition of context assertions and custom fixtures. Use `@vitest/eslint-plugin` when ESLint is already the owner, not as a redundant second pass. Reject automatic truthiness weakening and arbitrary assertion-count caps.

Sources: [Vi API](https://vitest.dev/api/vi), [context](https://vitest.dev/guide/test-context), [AI test guidance](https://vitest.dev/guide/learn/writing-tests-with-ai), [narrowing](https://vitest.dev/guide/recipes/type-narrowing), [Oxlint](https://oxc.rs/docs/guide/usage/linter/rules/vitest/valid-expect).
