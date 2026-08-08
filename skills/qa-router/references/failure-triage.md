# Failure and Flake Triage

Use this reference for failing local checks, CI reproduction, recurring tests,
and nondeterministic suites.

## Root-Cause Loop

1. Capture the exact command, runner/project/shard, seed if any, and first
   actionable error.
2. Preserve raw evidence without copying secrets or private payloads.
3. Reproduce through the narrowest repository-owned command.
4. Classify the failure before editing.
5. Form one falsifiable hypothesis and run the cheapest discriminating check.
6. Read the owning test, setup, fixture, config, and production boundary.
7. Patch the first verified unintended behavior, not downstream noise.
8. Rerun the narrow command repeatedly or under the relevant stress dimension.
9. Run the wider owning gate.

## Failure Classes

- **Assertion/behavior:** implementation or expectation violates the contract.
- **Type/lint/build:** static contract, generated output, module boundary, or
  production compilation failure.
- **Environment:** missing binary, service, variable, permission, browser,
  simulator, locale, timezone, or filesystem capability.
- **Configuration:** wrong project selection, setup order, transform, alias,
  include/exclude, timeout, worker, or CI routing.
- **Dependency:** lock drift, incompatible version, platform artifact, or
  changed upstream behavior.
- **Resource:** memory, CPU, file descriptor, port, disk, process, or service
  exhaustion.
- **Provider/network:** uncontrolled external dependency, outage, rate limit,
  DNS, TLS, proxy, or incomplete interception.
- **Flake:** pass/fail changes without a relevant code/config change.

## Flake Taxonomy

### Timing

Signals: intermittent timeout, missing await, state observed before completion,
fake timers mixed with real timers, animation or debounce race.

Corrections: await a meaningful state transition, drive fake time explicitly,
subscribe before triggering, and assert eventual user-visible state. Do not add
wall-clock sleep.

### Order

Signals: a test passes alone but fails after another test or under randomized
order.

Corrections: restore mocks/env, recreate fixtures, remove singleton mutation,
reset database/files/DOM, and make setup independent. Do not encode the lucky
order.

### Parallelism

Signals: failures only with multiple workers/shards or in CI load.

Corrections: allocate unique ports/files/rows, make fixtures worker-safe, bound
shared resources, and isolate databases. Serial execution is acceptable only
when the behavior is inherently exclusive and that constraint is documented.

### Network

Signals: DNS/rate-limit/provider variation, leaked requests, inconsistent
latency, cassette drift, or incomplete service-worker interception.

Corrections: intercept at the real boundary, use deterministic local responses,
assert request shape, and fail on unhandled calls. Do not make unit/integration
tests depend on a live provider.

### Leak

Signals: open-handle warnings, process hangs, later-test contamination,
increasing memory, stale DOM/listeners, timers, files, or database rows.

Corrections: close servers/clients, clear timers, unsubscribe listeners,
restore globals, dispose DOM, remove temp files, roll back transactions, and
assert cleanup where valuable.

## Diagnostic Stress

Use only flags supported by the repository's runner/version:

- repeat the narrow test enough times to challenge the observed frequency;
- randomize test order with a recorded seed;
- compare one worker with normal parallelism;
- run the CI shard/project exactly;
- enable open-handle, resource, trace, or verbose diagnostics;
- vary timezone/locale only when evidence points there.

Consult current official docs or source before changing config or relying on a
runner flag. Diagnostic flags do not automatically belong in permanent scripts.

## Invalid Fixes

Reject these unless evidence proves they are the correct contract:

- arbitrary timeout increases;
- retries that merely hide failures;
- sleeps or polling without a state condition;
- weakened or deleted assertions;
- global serialization of a parallel-safe suite;
- skipped/quarantined tests without an owner and exit condition;
- production flags or branches used only by tests;
- silent skipping when a required environment prerequisite is absent.

## Confidence Reporting

Do not say "flake fixed" after one pass. Report:

- original reproduction frequency and conditions;
- root cause evidence;
- stress method after the fix;
- number of clean repetitions/shards/workers when meaningful;
- wider gate result;
- residual confidence limits and `UNVERIFIED` environments.

If the failure cannot be reproduced, say so. Preserve the leading hypotheses
and the next discriminating evidence rather than making speculative changes.
