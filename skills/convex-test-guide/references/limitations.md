# convex-test limitations and mock semantics

Use this before relying on `convex-test` as production proof.

## Official limitation categories

`convex-test` is a mock implementation. Official docs call out differences in:

- error message content;
- size and time limits;
- document/storage ID format;
- runtime built-ins because Vitest uses an Edge Runtime mock, not the exact
  Convex runtime;
- simplified text search semantics;
- simplified vector search implementation;
- no cron job support.

When behavior depends on these, mark `convex-test` coverage as necessary but not
sufficient and add a real-backend/manual/CI gate.

## Search and vector search

Text search in the mock is prefix/word based and does not rank by real Convex
relevance. Vector search sorts in memory by cosine similarity and does not use an
efficient backend vector index. Use it for branch coverage and result-shape
logic, not for proving production relevance or performance.

## IDs

Current source generates IDs with numeric prefixes and table suffixes targeting a
regular Convex-like length. This is intentionally not the real backend ID format.
Never assert exact ID strings or parse table names from IDs in product logic.

## Runtime and network

Vitest `edge-runtime` is close enough for many Convex functions but not exact.
If code uses unusual globals, Web APIs, crypto, blobs, streams, timers, or
provider SDK behavior, keep a manual/real-backend check.

No real network calls in unit or integration tests. For actions that call
external providers, mock `fetch` or provider clients at the boundary.

## Transactions and parallelism

`convex-test@0.0.53` uses AsyncLocalStorage-scoped global state and a transaction
manager that serializes top-level function executions. Nested transactions are
not isolated the same way as independent top-level calls. This is good for test
stability, but do not use it to prove high-concurrency production behavior.

Mutations roll back staged writes on thrown errors. Queries run in rolled-back
transactions. Snapshot queries hide pending writes from child queries.

## Scheduled functions

`0.0.53` fixes scheduled functions with or without fake timers by serializing
scheduled mutations through the global transaction manager. Still, scheduled
function tests should use fake timers and cleanup. Cron jobs are unsupported;
trigger cron targets manually as functions.

`finishAllScheduledFunctions` has bounded iteration/pump limits and will throw
if recursive scheduling or unresolved timers never settle.

## Components

Components require explicit `t.registerComponent(componentPath, schema, glob)`.
Component auth isolation differs from app auth propagation; source intentionally
resets auth across component boundaries. Test app wrappers and component public
APIs separately.

## Return and argument validators

The mock validates args and return values. This catches useful contract defects,
but exact error strings are not production-stable. Assert that validation fails,
not the full implementation message.

## Choosing real backend tests

Use local/open-source/backend deployment tests instead of or in addition to
`convex-test` when verifying:

- cron scheduling;
- hosted runtime built-ins;
- exact search/vector relevance/performance;
- production limits at scale;
- deployment, codegen, dashboard/import behavior;
- real auth provider token verification;
- external network/provider integrations.
