# convex-test authoring patterns

Use this when adding or repairing Convex integration tests.

## Basic behavior test

```ts
const t = convexTest(schema, convexModules);
await t.mutation(api.messages.send, { body: 'Hi', author: 'Sarah' });
const messages = await t.query(api.messages.list, {});
expect(messages).toMatchObject([{ body: 'Hi', author: 'Sarah' }]);
```

Test product behavior through API references first. Use inline functions only
for setup, direct DB assertions, or helper functions that need a Convex ctx.

## Setup and direct assertions

Use `t.run` for direct database/storage manipulation outside public functions:

```ts
const id = await t.run(async (ctx) => {
  return await ctx.db.insert('messages', { body: 'seed', author: 'test' });
});

await t.run(async (ctx) => {
  const doc = await ctx.db.get(id);
  expect(doc).toMatchObject({ body: 'seed' });
});
```

Keep seeded rows minimal. Prefer reusable builders in `test_utils/convex/**`
when multiple suites need the same valid graph.

## Auth tests

```ts
const t = convexTest(schema, convexModules);
const authed = t.withIdentity({
  subject: 'user_123',
  issuer: 'https://example.clerk.accounts.dev',
  tokenIdentifier: 'https://example.clerk.accounts.dev|user_123',
  name: 'Ada',
});

await expect(authed.query(api.secure.viewer, {})).resolves.toMatchObject({
  name: 'Ada',
});
```

If `issuer`, `subject`, or `tokenIdentifier` are omitted, `convex-test`
generates defaults. In Signr, prefer explicit Clerk-like values when auth code
keys on token identifiers or issuer domains.

## HTTP actions

```ts
const t = convexTest(schema, convexModules);
const response = await t.fetch('/webhooks/provider', {
  method: 'POST',
  body: JSON.stringify(payload),
  headers: { 'content-type': 'application/json' },
});
expect(response.status).toBe(200);
```

`t.fetch` routes through `convex/http.ts`. Use it for HTTP action behavior.
Mock external network inside the action; do not let Vitest hit real providers.

## Scheduled functions

Use fake timers and always restore them:

```ts
import { afterEach, beforeEach, vi } from 'vitest';

beforeEach(() => vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] }));
afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

it('runs scheduled chain', async () => {
  const t = convexTest(schema, convexModules);
  await t.mutation(api.jobs.enqueue, {});

  await t.finishAllScheduledFunctions(vi.runAllTimers);

  const row = await t.run(async (ctx) => ctx.db.query('jobs').first());
  expect(row).toMatchObject({ status: 'done' });
});
```

Use `finishInProgressScheduledFunctions()` after you manually advance time to a
known scheduled point. Use `finishAllScheduledFunctions(vi.runAllTimers)` for
recursive chains.

Signr has `withoutScheduledTimers` in `packages/backend/test_utils/convex/scheduled.ts`.
Use it for operations that may schedule Convex work and must not leak timers.

## Components

```ts
const t = convexTest(schema, convexModules);
t.registerComponent('counter', counterSchema, counterModules);
const count = await t.query(components.counter.public.count, { name: 'beans' });
```

Register every component path used by the test. Auth does not propagate across
component boundaries in the mock; assert component auth behavior explicitly if
that matters.

## Storage

Use `t.run` or functions that call `ctx.storage`. Assert storage IDs and metadata
through Convex-visible state. Do not assert test storage ID formatting.

## Errors

Prefer stable assertions:

```ts
await expect(t.mutation(api.fn.bad, {})).rejects.toMatchObject({
  data: { code: 'forbidden' },
});
```

Avoid exact backend error message strings. Convex docs explicitly warn that real
backend error message content is unstable, and the mock differs from it.

## Transaction limits

```ts
const t = convexTest({
  schema,
  modules: convexModules,
  transactionLimits: { documentsRead: 10 },
});
```

Use for targeted performance regressions. Keep limits local to the test; global
enforcement tends to create noisy unrelated failures.
