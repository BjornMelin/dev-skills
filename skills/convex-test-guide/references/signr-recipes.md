# Signr convex-test recipes

Use this for common Signr Convex integration snippets. Keep recipes small and
prefer existing builders in `packages/backend/test_utils/convex/**` when they
exist.

## Base imports

```ts
import { convexTest } from 'convex-test';
import { describe, expect, it } from 'vitest';

import { api, internal } from '../../../convex/_generated/api';
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';
```

Adjust relative paths for deeper test folders.

## Minimal harness

```ts
const t = convexTest(schema, convexModules);
```

Use this for DB-backed behavior. Do not use `convexTest()` without schema in
Signr backend tests unless the test intentionally avoids schema/index behavior.

## Clerk-like identity

```ts
const asMember = t.withIdentity({
  issuer: 'https://example.clerk.accounts.dev',
  subject: 'user_member',
  tokenIdentifier: 'https://example.clerk.accounts.dev|user_member',
  name: 'Member User',
});
```

Use explicit `issuer`, `subject`, and `tokenIdentifier` when code reads Clerk
identity fields. Use different `subject` values for cross-user tests.

## Seed and assert through DB

```ts
const orgId = await t.run(async (ctx) => {
  return await ctx.db.insert('organizations', {
    name: 'Test Org',
    slug: 'test-org',
  });
});

await t.run(async (ctx) => {
  const org = await ctx.db.get(orgId);
  expect(org).toMatchObject({ slug: 'test-org' });
});
```

Keep direct DB writes for setup and invariant assertions. Exercise product
behavior through public/internal functions after setup.

## Denied auth branch

```ts
await expect(t.query(api.workspace.viewer, { orgId })).rejects.toThrow();
```

Prefer stable product errors or `ConvexError.data` when available. Avoid exact
mock backend validation strings.

## Authz matrix in one harness

Use one `convexTest` harness for wrong-org tests so both orgs and users exist in
the same mock database.

```ts
const TEST_ISSUER = 'https://example.clerk.accounts.dev';

function identity(subject: string, name: string) {
  return {
    issuer: TEST_ISSUER,
    subject,
    tokenIdentifier: `${TEST_ISSUER}|${subject}`,
    name,
  };
}

const t = convexTest(schema, convexModules);

const seeded = await t.run(async (ctx) => {
  const ownerUserId = await ctx.db.insert('users', {
    tokenIdentifier: `${TEST_ISSUER}|user_owner`,
    clerkId: 'user_owner',
    name: 'Owner',
  });
  const outsiderUserId = await ctx.db.insert('users', {
    tokenIdentifier: `${TEST_ISSUER}|user_outsider`,
    clerkId: 'user_outsider',
    name: 'Outsider',
  });
  const ownerOrgId = await ctx.db.insert('orgs', {
    clerkOrgId: 'org_owner',
    name: 'Owner Org',
  });
  const outsiderOrgId = await ctx.db.insert('orgs', {
    clerkOrgId: 'org_outsider',
    name: 'Outsider Org',
  });

  await ctx.db.insert('orgMemberships', {
    orgId: ownerOrgId,
    userId: ownerUserId,
    role: 'member',
  });
  await ctx.db.insert('orgMemberships', {
    orgId: outsiderOrgId,
    userId: outsiderUserId,
    role: 'member',
  });

  return { ownerOrgId, outsiderOrgId };
});

await expect(
  t.query(api.workspace.viewer, { orgId: seeded.ownerOrgId }),
).rejects.toThrow();

await expect(
  t
    .withIdentity(identity('user_outsider', 'Outsider'))
    .query(api.workspace.viewer, { orgId: seeded.ownerOrgId }),
).rejects.toThrow();

await expect(
  t
    .withIdentity(identity('user_owner', 'Owner'))
    .query(api.workspace.viewer, { orgId: seeded.ownerOrgId }),
).resolves.toMatchObject({ orgId: seeded.ownerOrgId });
```

Replace table names and required fields with the actual Signr schema. If the
repo exposes an error-code helper, prefer that over raw `.toThrow()` strings.

## Scheduled work

```ts
import { afterEach, beforeEach, vi } from 'vitest';

beforeEach(() => vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout'] }));
afterEach(() => {
  vi.clearAllTimers();
  vi.useRealTimers();
});

it('finishes scheduled work', async () => {
  const t = convexTest(schema, convexModules);

  await t.mutation(api.jobs.enqueue, {});
  await t.finishAllScheduledFunctions(vi.runAllTimers);

  const job = await t.run(async (ctx) => ctx.db.query('jobs').first());
  expect(job).toMatchObject({ status: 'done' });
});
```

Use `withoutScheduledTimers` from `packages/backend/test_utils/convex/scheduled.ts`
when the helper already fits.

## HTTP action

```ts
const response = await t.fetch('/webhooks/provider', {
  method: 'POST',
  body: JSON.stringify(payload),
  headers: { 'content-type': 'application/json' },
});

expect(response.status).toBe(200);
```

Mock outbound provider calls. Use `t.fetch` for inbound HTTP action routing.

## Workpool component tests

```ts
import { register as registerWorkpool } from '@convex-dev/workpool/test';

const t = convexTest(schema, convexModules);
registerWorkpool(t, 'billingReconciliationWorkpool');
```

Use component test helpers already provided by the component package before
writing local fakes.
