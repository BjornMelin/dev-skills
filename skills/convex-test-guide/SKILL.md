---
name: convex-test-guide
description: "Signr convex-test/Vitest authoring: harness shape, authz branches, fake timers, HTTP, components. TRIGGER when writing or fixing tests under packages/backend/test/convex."
---

# Convex Test Guide

Use `convex-test` for fast deterministic Convex function tests in JS/Vitest. It
is a mock backend, not a hosted Convex deployment. Use it to prove business
logic, schema/index behavior, auth branches, scheduler effects, HTTP action
routing, storage metadata, and component wiring. Use real backend/manual gates
when runtime parity or deployment behavior matters.

## Source stamp

This skill is based on `convex-test@0.0.53`, Signr direct dependency
`packages/backend/package.json`, upstream tag `v0.0.53`, and package source at
`/home/bjorn/.opensrc/repos/github.com/get-convex/convex-test/0.0.53`.
Refresh with `$opensrc` when the installed version changes.

## First decision

- Use pure unit tests for pure helpers. Do not boot `convex-test` for logic that
  does not need Convex schema, indexes, auth, storage, scheduler, or function
  dispatch.
- Use `convexTest(schema, convexModules)` for Signr backend behavior involving
  persisted rows, indexes, authz, internal/public functions, scheduled work, or
  HTTP actions.
- Use `t.run` for direct DB/storage setup and assertions. Use `t.query`,
  `t.mutation`, `t.action`, and `t.fetch` for public/internal function behavior.
- Use `t.withIdentity(identity)` for authenticated branches. Keep identities
  deterministic and explicit.
- Use fake timers plus `finishInProgressScheduledFunctions` or
  `finishAllScheduledFunctions` for scheduled work. Always restore timers.
- Use real backend validation when the test depends on production limits,
  runtime built-ins, cron jobs, hosted deployment behavior, or exact search
  relevance.

## Reference routing

- Full API and capability map: `references/api-map.md`.
- Test authoring recipes: `references/authoring-patterns.md`.
- Mock semantics and limitations: `references/limitations.md`.
- Signr/Vitest routing: `references/signr-vitest.md`.
- Copy-ready Signr snippets: `references/signr-recipes.md`.
- `/help`, `/audit`, and `/review` command protocol:
  `references/audit-review.md`.

Load only the file matching the task. If unsure, read `api-map.md` first.

## Bundled scripts

- `scripts/convex-test-snippet.mjs`: print Signr-shaped `convex-test`
  boilerplate for `base`, `authz`, `scheduled`, `http`, or `workpool` tests.
- `scripts/convex-test-capability-check.mjs`: static guardrail check for
  Convex test placement, environment pragmas, scheduler timers, and sleeps.
- `scripts/convex-test-audit.mjs`: print `/help` and produce a structured
  candidate ledger for `/audit` and `/review`; validate candidates before final
  findings.

## Signr default shape

```ts
import { convexTest } from 'convex-test';
import { describe, expect, it } from 'vitest';

import { api, internal } from '../../../convex/_generated/api';
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

describe('feature', () => {
  it('enforces the behavior', async () => {
    const t = convexTest(schema, convexModules);
    const asUser = t.withIdentity({ subject: 'user_test', name: 'Test User' });

    await t.run(async (ctx) => {
      await ctx.db.insert('users', { /* minimal row */ });
    });

    const result = await asUser.query(api.someModule.someQuery, {});
    expect(result).toMatchObject({ /* behavior */ });
  });
});
```

## Guardrails

- In Signr, put Convex integration tests under
  `packages/backend/test/convex/**/*.test.ts` only.
- Do not add tests under `packages/backend/convex/**`, `.convex-test.ts`
  suffixes, or file-level environment pragmas.
- Let the repo `backend-convex` Vitest project own `edge-runtime`, dependency
  inlining, setup files, file parallelism, and workers.
- Keep shared builders in `packages/backend/test_utils/convex/**`.
- No real network calls. Mock `fetch` or test `t.fetch` HTTP actions directly.
- No wall-clock sleeps. Use fake timers or explicit state assertions.
- Do not assert exact Convex backend error strings; assert product-visible
  behavior or stable `ConvexError.data` shape.
