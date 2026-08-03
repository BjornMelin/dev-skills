---
name: convex-helpers-guide
description: "Signr convex-helpers selection/audits (customFunctions authz, validators, triggers, pagination, migrations). TRIGGER when adding, reviewing, or debugging convex-helpers usage in packages/backend."
---

# Convex Helpers Guide

Use `convex-helpers` when it removes repeated Convex boilerplate without hiding
Signr authz, tenant scope, indexing, or API ownership. Prefer the package helper
over custom glue when the helper is already installed, typed, and fits the
actual access pattern. Do not add helper indirection for one-off code.

## Source stamp

This skill is based on `convex-helpers@0.1.120`, Signr direct dependency
`packages/backend/package.json`, upstream tag `npm/0.1.120`, and package source
at `/home/bjorn/.opensrc/repos/github.com/get-convex/convex-helpers/0.1.120/packages/convex-helpers`.
Refresh with `$opensrc` when the installed version changes.

## First decision

- Use indexes and native Convex queries first for simple reads.
- Use custom functions for repeated auth, org scope, ctx enrichment, trigger/RLS
  DB wrapping, or dynamic per-function requirements.
- Use relationship helpers for clear point reads and indexed back-references.
- Use `filter`, `paginator`, or `stream` only after bounding reads with indexes
  or explicit pagination limits.
- Use triggers for invariant maintenance inside wrapped mutations only; dashboard
  edits, imports, and raw unwrapped mutations do not run triggers.
- Use CRUD only for prototypes or internal functions unless paired with explicit
  auth/RLS. Public Signr APIs should stay purpose-built.

## Reference routing

- Full module and import map: `references/package-map.md`.
- Backend/server patterns: `references/server-patterns.md`.
- React, sessions, browser, testing, and CLI: `references/client-and-tooling.md`.
- Signr-specific choices and anti-patterns: `references/signr-guidance.md`.
- `/help`, `/audit`, and `/review` command protocol:
  `references/audit-review.md`.

Load only the file matching the task. If unsure, read `package-map.md` first.

## Bundled scripts

- `scripts/convex-package-snapshot.mjs`: snapshot installed/source package
  metadata, exports, source paths, and README headings.
- `scripts/convex-helper-import-map.mjs`: render a markdown or JSON import map
  from a `convex-helpers` package source directory.
- `scripts/convex-helpers-audit.mjs`: print `/help` and produce a structured
  candidate ledger for `/audit` and `/review`; validate candidates before final
  findings.

## High-value import map

| Need | Prefer | Import |
| --- | --- | --- |
| Auth/org wrappers | `customQuery`, `customMutation`, `customAction`, `customCtx` | `convex-helpers/server/customFunctions` |
| Fetch related docs | `getOrThrow`, `getAll`, `getOneFrom`, `getManyFrom`, `getManyVia` | `convex-helpers/server/relationships` |
| Convex validators | `typedV`, `doc`, `partial`, `literals`, `nullable`, `validate`, `parse` | `convex-helpers/validators` |
| Zod 4 functions | `zCustomQuery`, `zCustomMutation`, `zCustomAction`, `zid` | `convex-helpers/server/zod4` |
| Zod 3 legacy | same family | `convex-helpers/server/zod3` |
| Convex <-> Zod conversion | `zodToConvex`, `convexToZod`, `withSystemFields` | `convex-helpers/server/zod4` or `server/zod3` |
| Standard Schema | `toStandardSchema` | `convex-helpers/standardSchema` |
| JS/TS post-filtering | `filter` | `convex-helpers/server/filter` |
| Manual pages | `getPage`, `paginator` | `convex-helpers/server/pagination` |
| Merge/paginate multiple indexed reads | `stream`, `mergedStream`, `MergedStream` | `convex-helpers/server/stream` |
| Mutation side effects/invariants | `Triggers`, `writerWithTriggers` | `convex-helpers/server/triggers` |
| Stateful backfills | `makeMigration`, `startMigration`, `getStatus`, `cancelMigration` | `convex-helpers/server/migrations` |
| RLS-style DB wrapper | `RowLevelSecurity`, `wrapDatabaseReader`, `wrapDatabaseWriter` | `convex-helpers/server/rowLevelSecurity` |
| Simple CRUD generators | `crud` | `convex-helpers/server/crud` |
| HTTP CORS | `corsRouter` | `convex-helpers/server/cors` |
| Hono HTTP router | `Hono`, `HttpRouterWithHono`, `HonoWithConvex` | `convex-helpers/server/hono` |
| Local helper rate limit | `defineRateLimits`, `rateLimit`, `checkRateLimit` | `convex-helpers/server/rateLimit` |
| Retry actions | `makeActionRetrier`, `withJitter` | `convex-helpers/server/retries` |
| React query status | `useQuery`, `usePaginatedQuery` | `convex-helpers/react` |
| React query cache | `ConvexQueryCacheProvider`, cached hooks | `convex-helpers/react/cache` |
| Anonymous sessions | `SessionProvider`, `useSessionQuery`, `useSessionId` | `convex-helpers/react/sessions` |
| Server session arg | `SessionIdArg`, `runSessionFunctions` | `convex-helpers/server/sessions` |
| Client arg binding | `withArgs` | `convex-helpers/browser` |
| Test helper client | `ConvexTestingHelper` | `convex-helpers/testing` |
| External API typings/specs | `ts-api-spec`, `open-api-spec` | `bunx --bun convex-helpers ...` |

## Guardrails

- Signr apps import backend functions through `@signr/backend/api` and
  `@signr/backend/dataModel`; do not leak Convex internals into app packages.
- Do not use helpers to justify unbounded `.collect()` or client-side authz.
- Do not wrap raw mutation/query builders in multiple invisible layers. Compose
  ordinary shared functions and expose one named custom builder per policy.
- Prefer `server/zod4` for new Zod work. Use `server/zod3` only for legacy Zod 3.
- Keep `convex-helpers` examples using Signr's Bun command policy:
  `bun add convex-helpers`, `bunx --bun convex-helpers ts-api-spec`.
