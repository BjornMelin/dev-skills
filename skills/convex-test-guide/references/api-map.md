# convex-test API map

Use this when you need exact `convex-test` capabilities and imports.

## Package evidence

- Package: `convex-test@0.0.53`.
- Peer range: `convex ^1.32.0`.
- Signr branch evidence: root and backend depend on `convex-test ^0.0.53` and
  `convex 1.41.0`.
- Upstream package description: JS mock of the Convex backend for testing Convex
  functions.
- Upstream Vitest config: `environment: 'edge-runtime'` and
  `server.deps.inline: ['convex-test']`.

## Import

```ts
import { convexTest } from 'convex-test';
```

Types exported by source:

```ts
import type {
  TestConvex,
  TestConvexForDataModel,
  TestConvexForDataModelAndIdentity,
} from 'convex-test';
```

## Initialize

Supported forms:

```ts
const t = convexTest();
const t = convexTest(schema);
const t = convexTest(schema, modules);
const t = convexTest({ schema, modules });
const t = convexTest({ schema, modules, transactionLimits: true });
const t = convexTest({ schema, modules, transactionLimits: { bytesRead: 1024 } });
```

- Pass `schema` for schema validation, indexes, vector indexes, and table/id
  validation.
- Pass `modules` from `import.meta.glob(...)` when functions are outside the
  default assumptions or when testing functions, HTTP actions, scheduled work,
  or components by API reference.
- Use `transactionLimits` only when intentionally testing limit behavior. `false`
  is the default; `true` enables default Convex-like limits; an object overrides
  individual metrics.

## Public test handle

The `t` object supports:

- `t.query(functionReference, args?)`: call public/internal query.
- `t.query(async (ctx) => value)`: run inline query.
- `t.mutation(functionReference, args?)`: call public/internal mutation.
- `t.mutation(async (ctx) => value)`: run inline mutation.
- `t.action(functionReference, args?)`: call public/internal action.
- `t.action(async (ctx) => value)`: run inline action.
- `t.run(async (ctx) => value)`: direct setup/assertion context with mutation DB
  access plus storage support.
- `t.fetch(pathQueryFragment, init?)`: call HTTP actions registered in
  `http.ts`; path must start with `/`.
- `t.withIdentity(partialIdentity)`: return a `t` handle with auth identity.
- `t.finishInProgressScheduledFunctions()`: wait for scheduled functions already
  fired and in progress.
- `t.finishAllScheduledFunctions(advanceTimers)`: repeatedly advance timers and
  wait for recursively scheduled functions.
- `t.registerComponent(componentPath, schema, glob)`: register component schema
  and modules for component API references.

Internal implementation also has helpers such as `runInComponent`, `queryFromPath`,
`mutationFromPath`, `actionFromPath`, and `fun`; treat them as implementation
surface unless source/types require them for a very specific component test.

## What the mock implements

Source and tests cover:

- queries, mutations, actions, nested `ctx.runQuery`, `ctx.runMutation`, and
  `ctx.runAction`;
- inline query/mutation/action handlers;
- public and internal API references;
- argument validators and return validators;
- schema validation and relaxed schema validation;
- `db.get`, `db.system.get`, `db.insert`, `db.patch`, `db.replace`, `db.delete`,
  `db.query`, `withIndex`, order, filters, `.first`, `.unique`, `.collect`,
  `.take`, `.paginate`, and hidden `.count()`;
- explicit table-name db syntax;
- search indexes with simplified text semantics;
- vector search with cosine similarity over in-memory rows;
- file storage via `ctx.storage` and storage system rows;
- HTTP actions through `t.fetch`;
- scheduler rows and scheduled query/mutation/action execution;
- component registration and component API calls;
- `ctx.auth.getUserIdentity()` through `withIdentity`;
- `ctx.meta.getDeploymentMetadata`, `ctx.meta.getFunctionMetadata`,
  `ctx.meta.getTransactionMetrics`, and internal audit-log syscall support;
- `ConvexError` deserialization across nested functions;
- transaction rollback, nested function calls, and serialization of top-level
  function executions.

## Transaction limit metrics

`transactionLimits` can enforce these metrics:

- `bytesRead`
- `bytesWritten`
- `databaseQueries`
- `documentsRead`
- `documentsWritten`
- `functionsScheduled`
- `scheduledFunctionArgsBytes`

Use this to catch accidentally broad scans or huge writes in tests. Do not turn
it on globally without a clear migration plan; it can expose unrelated existing
test debt.
