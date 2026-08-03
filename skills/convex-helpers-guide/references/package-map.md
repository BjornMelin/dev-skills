# convex-helpers package map

Use this when you need to know whether a helper exists, what it imports from,
or what capability was missing from the old skill.

## Current package evidence

- Package: `convex-helpers@0.1.119`.
- Peer range: `convex ^1.32.0`, `typescript ^5.5 || ^6.0.0`, optional peers
  `react`, `zod`, `hono`, `@standard-schema/spec`.
- Signr branch evidence: backend depends on `convex-helpers 0.1.119` and
  `convex 1.41.0`; Bun lock also contains older transitive copies under some
  Convex components.
- Upstream package README index lists: custom functions, relationship helpers,
  action retries, stateful migrations, rate limiting, sessions, richer
  `useQuery`, row-level security, Zod validation, Hono, CRUD, validator
  utilities, filter, manual pagination, composable streams, query caching,
  TypeScript API generation, OpenAPI generation, triggers, CORS, Standard
  Schema.

## Root utilities

Import from `convex-helpers`:

- `asyncMap`: typed `Promise.all` map helper.
- `pruneNull`: remove nulls from an array.
- `NullDocumentError`, `nullThrows`: null guard with explicit failure.
- `pick`, `omit`, `withoutSystemFields`: object field helpers.
- Types: `EmptyObject`, `BetterOmit`, `Expand`, `Equals`, `ErrorMessage`.
- `assert`: runtime assertion.

Use these instead of local one-off variants when the repo already has the
package. Do not import root utilities just to save one obvious line.

## Browser and React modules

- `convex-helpers/browser`: `withArgs` binds static args to `ConvexClient` or
  `ConvexHttpClient` calls. Use for external clients or generated API surfaces.
- `convex-helpers/react`: richer `useQuery`, `makeUseQueryWithStatus`, and
  `usePaginatedQuery` for custom pagination helpers.
- `convex-helpers/react/cache`: `ConvexQueryCacheProvider`, cached `useQuery`,
  `useQueries`, and `usePaginatedQuery` that keep subscriptions open briefly
  after components unmount.
- `convex-helpers/react/sessions`: `SessionProvider`, `useSessionQuery`,
  `useSessionPaginatedQuery`, `useSessionMutation`, `useSessionAction`,
  `useSessionId`, `useSessionIdArg`, `useSessionStorage`,
  `ConvexReactSessionClient`.

## Server modules

- `convex-helpers/server`: `Table`, `missingEnvVariableUrl`,
  `missingEnvVariableError`, `deploymentName`, and a server-level `crud` helper.
- `server/customFunctions`: `customQuery`, `customMutation`, `customAction`,
  `customCtx`, `customCtxAndArgs`, `NoOp`, `CustomCtx`, `CustomBuilder`,
  `Customization`.
- `server/relationships`: `getOrThrow`, `getAll`, `getAllOrThrow`,
  `getOneFrom`, `getOneFromOrThrow`, `getManyFrom`, `getManyVia`,
  `getManyViaOrThrow`.
- `server/filter`: `filter` with `.first()`, `.unique()`, `.take()`,
  `.paginate()`, `.collect()`, `.next()`, `.withIndex()`, `.withSearchIndex()`,
  and async predicates.
- `server/pagination`: `getPage`, `streamQuery`, `paginator` for manual or
  repeated pagination. Supports index ranges and `maximumBytesRead` in current
  versions.
- `server/stream`: `stream`, `mergedStream`, `MergedStream`, `QueryStream`,
  `SingletonStream`, `EmptyStream`, stream maps, filters, joins, merge ordering,
  and pagination.
- `server/triggers`: `Triggers`, `DatabaseWriterWithTriggers`,
  `writerWithTriggers`, trigger `Change` and `Trigger` types.
- `server/migrations`: `makeMigration`, `startMigration`,
  `startMigrationsSerially`, `getStatus`, `cancelMigration`, migration schema
  and status helpers.
- `server/rowLevelSecurity`: `RowLevelSecurity`, `BasicRowLevelSecurity`,
  `wrapDatabaseReader`, `wrapDatabaseWriter`, rule types, deny-by-default config.
- `server/crud`: table CRUD generator with create/read/update/destroy/paginate.
- `server/cors`: `corsRouter`, route-level CORS config, default exposed headers.
- `server/hono`: `Hono`, `HttpRouterWithHono`, `HonoWithConvex`,
  `normalizeMethod`.
- `server/rateLimit`: `defineRateLimits`, `rateLimit`, `checkRateLimit`,
  `resetRateLimit`, token bucket and fixed window config.
- `server/retries`: `makeActionRetrier`, `withJitter`.
- `server/sessions`: `SessionId`, `vSessionId`, `SessionIdArg`,
  `runSessionFunctions`.
- `server/compare`: `compareValues` for Convex value ordering.

## Validation modules

- `convex-helpers/validators`: `literals`, `nullable`, `partial`, primitive
  aliases, `systemFields`, `withSystemFields`, `addFieldsToValidator`, `doc`,
  `typedV`, `brandedString`, `deprecated`, `pretend`, `pretendRequired`,
  `ValidationError`, `validate`, `parse`, `vRequired`.
- `convex-helpers/server/zod4`: Zod 4-first `zCustomQuery`,
  `zCustomMutation`, `zCustomAction`, `zid`, `zodToConvex`,
  `zodOutputToConvex`, field converters, `convexToZod`, `withSystemFields`,
  conversion types.
- `convex-helpers/server/zod3`: Zod 3 equivalent. Use only for legacy projects.
- `convex-helpers/server/zod`: compatibility surface. Prefer explicit zod3/zod4
  imports in new code.
- `convex-helpers/standardSchema`: `toStandardSchema` converts Convex validators
  into Standard Schema validators.

## CLI

Run with Bun in this repo:

```bash
bunx --bun convex-helpers ts-api-spec
bunx --bun convex-helpers open-api-spec
```

Use `ts-api-spec` for type-safe Convex access from a separate repository. Use
`open-api-spec` for non-TypeScript clients or tools such as Retool. Both read a
configured Convex deployment; pass `--prod` only when production is intentional.

## Gaps fixed from the previous skill

The old skill covered only relationships, custom functions, filter, sessions,
Zod, RLS, migrations, triggers, and a non-existent aggregation helper example.
It omitted retries, rate limiting, richer React query hooks, query caching, Hono,
CRUD, validator utilities, manual pagination, streams, API generators, CORS,
Standard Schema, browser helpers, testing helpers, `Table`, root utilities, and
several relationship functions. It also used stale session and Zod examples and
understated trigger semantics.
