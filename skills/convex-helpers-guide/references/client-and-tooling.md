# Client and tooling patterns

Use this for React, session, browser, testing, and generated API work.

## Sessions

Server import: `convex-helpers/server/sessions`.
React import: `convex-helpers/react/sessions`.

Use for anonymous users, guest carts, pre-signup preferences, or A/B state that
must be stable across requests before auth exists.

Modern pattern:

```tsx
import { SessionProvider, useSessionQuery } from 'convex-helpers/react/sessions';

<ConvexProvider client={convex}>
  <SessionProvider>
    <App />
  </SessionProvider>
</ConvexProvider>;

const result = useSessionQuery(api.example.withSession, { limit: 10 });
```

Server-side, compose with custom functions:

```ts
import { SessionIdArg } from 'convex-helpers/server/sessions';
import { customQuery } from 'convex-helpers/server/customFunctions';

export const queryWithSession = customQuery(query, {
  args: SessionIdArg,
  input: async (ctx, { sessionId }) => {
    const anonymousUser = await getAnonUser(ctx, sessionId);
    return { ctx: { ...ctx, anonymousUser }, args: {} };
  },
});
```

For Signr, do not introduce anonymous identity where Clerk/Convex identity is
required by product requirements. Sessions are for explicitly guest-capable
flows.

## Rich React query hooks

`convex-helpers/react` exports `useQuery`, `makeUseQueryWithStatus`, and
`usePaginatedQuery`.

Use when you need better loading/status handling than raw `convex/react` or when
using helper pagination/streams. Do not swap every query hook repo-wide without a
measured UX need.

## Query caching

`convex-helpers/react/cache` exports `ConvexQueryCacheProvider`, cached
`useQuery`, `useQueries`, and `usePaginatedQuery`.

Use when route/view changes repeatedly unmount and remount the same reactive
queries and faster perceived reload matters. It keeps subscriptions open after
unmount, so bandwidth can increase. Tune provider props:

- `expiration`: default 300000 ms.
- `maxIdleEntries`: default 250.
- `debug`: false by default.

For Next.js, import provider/hooks from subpaths if required by bundling:
`convex-helpers/react/cache/provider` and `convex-helpers/react/cache/hooks`.

## Browser helper

`convex-helpers/browser` exports `withArgs` for binding static args to Convex
browser or HTTP clients. Use for external clients that should not repeat common
args manually. Do not use it to hide auth-sensitive values in client code.

## Testing helper

`convex-helpers/testing` exports `ConvexTestingHelper`. Prefer Signr's existing
`convex-test` harness and backend test utilities unless a task specifically
needs a client-like Convex test helper.

## Standard Schema

`convex-helpers/standardSchema` exports `toStandardSchema`. Use when a tool or
library expects Standard Schema and Convex validators are already the source of
truth. Do not add a second validation source if Convex validators already cover
the boundary.

## API generators

Run from the Convex folder for the deployment whose functions you want to
expose:

```bash
bunx --bun convex-helpers ts-api-spec
bunx --bun convex-helpers open-api-spec
```

- `ts-api-spec`: generate TypeScript API objects for a separate repo.
- `open-api-spec`: generate OpenAPI YAML for non-TypeScript clients or external
  tools.
- Both default to the dev deployment. Use `--prod` only for intentional
  production introspection.

Generated specs may include internal functions. Remove or protect anything that
should not leave the backend boundary.
