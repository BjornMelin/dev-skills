# Execution Model

TanStack Start code is isomorphic by default. Route loaders run on the server during SSR and on the client during client navigation.

## Rules

- Do not put DB clients, secrets, filesystem calls, or Node-only APIs directly in loaders.
- Put server-only work behind `createServerFn`, server routes, `.server.*` modules, or server-only helpers.
- Treat loader output as route data and cache input, not as a security boundary.
- Guard browser-only APIs with client-only components, `ssr: false`, or explicit runtime checks.
- When hydration mismatches appear, first audit environment-dependent render branches.
