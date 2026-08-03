# Signr and Vitest routing

Use this for repo-local `convex-test` work in Signr.

## Canonical locations

- Convex integration tests: `packages/backend/test/convex/**/*.test.ts`.
- Shared Convex test builders/helpers: `packages/backend/test_utils/convex/**`.
- Convex module glob: `packages/backend/test_utils/convex/modules.ts`.
- Production Convex modules: `packages/backend/convex/**`.

Do not put tests under production Convex modules, do not use `.convex-test.ts`,
and do not add file-level `@vitest-environment` pragmas. The repo-level Vitest
project owns routing.

## Current Vitest project

The root `vitest.config.ts` builds projects from `scripts/lib/vitest-projects.ts`.
The `backend-convex` project is canonical:

```ts
{
  name: 'backend-convex',
  include: ['packages/backend/test/convex/**/*.test.ts'],
  environment: 'edge-runtime',
  fileParallelism: true,
  server: { deps: { inline: ['convex-test'] } },
}
```

Root shared config also enables `clearMocks`, `restoreMocks`, `unstubEnvs`,
`isolate: true`, and `pool: 'forks'` unless a project overrides it.

## Commands

```bash
bun run test:backend:convex
bun run --filter @signr/backend test:convex
SIGNR_BACKEND_CONVEX_TEST_WORKERS=8 bun run test:backend:convex
```

`test:backend:convex` defaults to four workers through
`SIGNR_BACKEND_CONVEX_TEST_WORKERS:-4`. Lower it only for debugging leaks; do
not hard-code single-worker mode without new flake evidence and docs updates.

## Module setup

Use the repo helper:

```ts
import schema from '../../../convex/schema';
import { convexModules } from '../../test_utils/convex/modules';

const t = convexTest(schema, convexModules);
```

`convexModules` includes `../../convex/**/*.ts` and excludes tests,
`__tests__`, and Convex test utils. It also stubs
`CLERK_JWT_ISSUER_DOMAIN` before each test.

## Test design

- Cover unauthorized, authorized, wrong-role, cross-org, and replay/idempotency
  paths where relevant.
- Seed through minimal builders or `t.run`; avoid broad fixture worlds.
- Assert observable backend state and function return values.
- Keep external providers mocked. Webhook tests should cover invalid signature,
  valid event, duplicate/replay, and provider/environment mismatch.
- Use fake timers for scheduled work and restore them in `afterEach` or via
  `withoutScheduledTimers`.
- For component-backed flows such as Workpool, prefer official component test
  helpers like `@convex-dev/workpool/test` when already present, then assert
  app-owned wrapper behavior through `convex-test`.

## Validation policy

When the user asks to validate Convex test changes, run the narrowest relevant
command first, normally:

```bash
bun run --filter @signr/backend test:convex -- <path-or-test-filter>
```

Then use `bun run test:backend:convex` for the full backend Convex lane. If the
change touches broader backend contracts, follow with the repo local routing
command requested by AGENTS.md.
