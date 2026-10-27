---
name: vitest-dev
description: "Write, debug, migrate, and speed up Vitest v5+ tests, Browser Mode, coverage, and Bun/Turborepo CI without weakening behavior checks."
license: MIT
metadata:
  version: "1.0.0"
  verified: "2026-10-27"
  vitest: "5.0.5"
---

# Vitest engineering

Improve confidence per test and feedback time, not test count or feature adoption. Work in the existing repository; do not scaffold a testing framework around Vitest.

## Execute the task

1. Inspect the affected implementation, nearest tests, package scripts, and applicable configuration. Read installed versions from the lockfile/manifests. Preserve the runner, runtime, and package manager unless changing them is the task. For upgrades, check matching addon peers and migration notes; do not install `latest` during an ordinary test repair.
2. Identify the observable contract and realistic regression. Exercise owned code directly; mock external boundaries only when needed. Add the smallest meaningful test or repair the existing one. Delete redundant coverage only when the retained test protects the same behavior.
3. Run the repository's scoped command, using `vitest run` semantics for noninteractive execution. Confirm selected files/tests, failures, skips, and exit status. A static `vitest list`, a cached task, or a test with every dependency mocked is not fresh behavioral verification.
4. Widen verification for shared configuration, dependency, runtime, or cross-package changes. For a local test edit, stop after the relevant checks pass. Report executed checks, unresolved failures, and measured performance changes; do not claim unexecuted browser/provider/CI checks passed.

## Read only what the task needs

An ordinary test edit can finish with this entry point. Read the indicated reference before changing that surface, not all references on activation. They are independent, with no nested local-reference chain.

| Task or uncertainty | Reference |
| --- | --- |
| Upgrade, removed API, projects, config inheritance, version mismatch | [Configuration and migration](references/configuration.md) |
| Slow tests, workers, isolation, transform caches, `doctor`, experimental execution | [Performance](references/performance.md) |
| Mocking, timers, async failures, fixtures, types, lint enforcement | [Test patterns](references/testing.md) |
| Real-browser components, locators, screenshots, traces, framework boundaries | [Browser testing](references/browser.md) |
| Bun/Turbo, affected tests, sharding, reports, coverage, type tests, benchmarks | [CI and execution](references/ci.md) |

Use the installed public types and CLI help for exact signatures. Consult the matching official release docs for unresolved details. Context7 and community skills are discovery aids, not authority over a newer installed patch. Never fetch the entire documentation corpus into a routine task.

## Keep tests honest

- Preserve type evidence: use inferred/native contracts, `satisfies`, typed mocks, and `expect.assert` for narrowing. No `any`, double assertions, fake generated IDs, or production interfaces added solely to appease tests.
- Assert the business result, including relevant failure and permission paths. Avoid tautologies, implementation snapshots, assertions on mocks alone, swallowed errors, and conditional assertions that can never run. Keep exact boolean assertions exact.
- Await asynchronous assertions and browser actions. Prefer observable completion or bounded polling over sleeps. Keep clocks, globals, network, and resources test-owned; concurrent tests must not mutate shared mocks or fake clocks.
- In v5, `clearMocks` clears history by default, not implementations. Hoisted `vi.mock`/`vi.hoisted` calls belong at module scope. Replace removed `.sequential` with `{ concurrent: false }`, which does not serialize other files.
- Preserve useful native defaults. Do not blanket-disable isolation, enable retries, raise timeouts, update snapshots, exclude coverage, or accept zero tests to manufacture green results. Use a feature only when it removes a demonstrated problem.
