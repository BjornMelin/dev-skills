# Server patterns

Use this for Convex backend implementation decisions with `convex-helpers`.

## Custom functions

Import from `convex-helpers/server/customFunctions`.

Use for repeated policy on Convex function builders:

- auth and org membership checks;
- ctx enrichment such as `ctx.user`, `ctx.org`, `ctx.db` wrappers;
- consuming client-only args such as API keys or session IDs;
- dynamic per-function requirements via the third `input` parameter;
- `onSuccess` finalization after the handler succeeds.

Minimal pattern:

```ts
import { customQuery } from 'convex-helpers/server/customFunctions';
import { query } from './_generated/server';

export const orgQuery = customQuery(query, {
  args: { orgId: v.id('organizations') },
  input: async (ctx, { orgId }, config?: { role?: 'admin' | 'member' }) => {
    const membership = await requireOrgMembership(ctx, orgId, config?.role);
    return { ctx: { ...ctx, orgId, membership }, args: {} };
  },
});
```

Keep only one visible custom builder per policy. If policies need composition,
compose ordinary functions inside `input`; do not stack wrappers until the call
site hides which auth and args apply.

## Relationships

Import from `convex-helpers/server/relationships`.

Use when the relationship is backed by IDs and indexes and the helper makes the
read easier to audit:

- direct ID fetch: `getOrThrow(db, id)`;
- many direct IDs: `getAll(db, ids)`, `getAllOrThrow(db, ids)`;
- one indexed back-reference: `getOneFrom`, `getOneFromOrThrow`;
- many indexed back-references: `getManyFrom`;
- join table lookups: `getManyVia`, `getManyViaOrThrow`.

Index naming shortcut: helpers infer the field when the index name is the field
or `by_` plus the field. Pass the field explicitly when the index name differs.

Do not use relationship helpers to hide hot fan-out reads. For user-facing hot
lists, prefer denormalized read models, pagination, or targeted indexes.

## Validators and Zod

Use `convex-helpers/validators` when Convex validators are the source of truth:

- `typedV(schema)` for schema-aware `id()` and `doc()`;
- `doc(schema, table)` or `withSystemFields` for full document validators;
- `partial`, `pick`, `omit`, `literals`, `nullable`, `brandedString`,
  `deprecated` for concise schema reuse;
- `validate` or `parse` for runtime checks. Pass `{ db: ctx.db }` when ID table
  membership matters.

Use `server/zod4` only when Zod is already the boundary contract or Zod-specific
features matter. Prefer Convex validators for backend-internal function args in
Signr unless there is an existing shared Zod schema.

Zod 4 pattern:

```ts
import * as z from 'zod';
import { zCustomQuery, zid } from 'convex-helpers/server/zod4';
import { NoOp } from 'convex-helpers/server/customFunctions';

const zodQuery = zCustomQuery(query, NoOp);

export const byUser = zodQuery({
  args: { userId: zid('users'), email: z.email().optional() },
  handler: async (ctx, args) => args,
});
```

Use `server/zod3` for Zod 3 projects. Avoid ambiguous `server/zod` imports in
new code.

## Filtering, pagination, and streams

Use native `withIndex` first. Helpers are for cases native Convex does not cover
cleanly.

- `filter(query, predicate)`: arbitrary JS/TS predicate with query-like methods.
  It reads documents until the terminal method is satisfied. Good for bounded
  indexed reads; risky for broad tables.
- `getPage`: explicit index range pages with returned index keys.
- `paginator(ctx.db, schema)`: near drop-in `.paginate()` replacement that can
  be called multiple times in one query and works in mutations/actions. It does
  not subscribe to end cursors automatically.
- `stream(ctx.db, schema)`: build ordered async streams from indexed queries;
  use `mergedStream`/`MergedStream`, `.map`, `.flatMap`, `.filterWith`,
  `.paginate()` for union, join, distinct/group-like access patterns.

For reactive custom pagination, use the helper `usePaginatedQuery` from
`convex-helpers/react`, or cached version with `customPagination: true`. Pass
end cursors when required to avoid holes or overlaps.

Always add `maximumRowsRead` or `maximumBytesRead` where a predicate might skip
many documents.

## Triggers

Import from `convex-helpers/server/triggers` and pair with custom mutations:

```ts
const triggers = new Triggers<DataModel>();
triggers.register('users', async (ctx, change) => {
  if (change.operation === 'insert') await updateUserReadModel(ctx, change.id);
});
export const mutation = customMutation(rawMutation, customCtx(triggers.wrapDB));
```

Use for denormalized fields, counters, cascades, audit rows, or component trigger
hooks such as aggregates. Triggers run in the same transaction as the write and
can abort the write by throwing.

Trigger caveats:

- only wrapped mutations/internal mutations run triggers;
- dashboard edits and imports do not run triggers;
- recursive triggers can loop; use `ctx.innerDb` for writes that should not
  trigger more triggers;
- first thrown trigger error is rethrown, other trigger errors are logged.

## Row-level security wrappers

Import from `convex-helpers/server/rowLevelSecurity`.

Use RLS wrappers as a database reader/writer guard when a broad invariant should
apply to every DB access through a custom function. In Signr, prefer explicit
custom function auth as the primary boundary and use RLS wrappers for additional
invariant enforcement, not as a substitute for endpoint-level authorization.

Prefer deny-by-default rules for protected tables. Make public/system tables
explicit.

## Migrations

Import from `convex-helpers/server/migrations`.

Use `makeMigration` for resumable backfills and schema transitions. Use
`startMigration`, `startMigrationsSerially`, `getStatus`, and
`cancelMigration` for operation control. For Signr production migrations, align
with repo migration runbooks and release verification; do not run ad hoc
production migrations from a skill example.

## CRUD

Import from `convex-helpers/server/crud`.

Use only for prototypes, admin/internal functions, or generated scaffolding that
will be hard-cut into purpose-built functions before exposure. Public CRUD must
be paired with authz/RLS and should usually be replaced with product-specific
functions.

## HTTP, CORS, Hono

- `corsRouter(http, config)`: attach CORS preflight and headers to HTTP routes.
  Supports route-level overrides, allowed origins, credentials, exposed headers,
  cache age, debug, and `enforceAllowOrigins`.
- `Hono`, `HonoWithConvex`, `HttpRouterWithHono`: use when a Convex `http.ts`
  has enough routes/middleware to justify Hono. In Signr, keep webhook trust,
  auth, rate limits, and org scope in app-owned wrappers.

## Rate limits and retries

- `server/rateLimit` provides table-backed token bucket/fixed window helpers.
  Signr already has `@convex-dev/rate-limiter`; prefer the component when it is
  already the repo standard and use this local helper only for simple app-owned
  limits that do not need the component.
- `makeActionRetrier` wraps retryable actions; pair with `withJitter` to avoid
  synchronized retries. Use for external side effects, not transactional writes.
