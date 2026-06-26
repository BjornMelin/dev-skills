# QueryClient in Router Context

Pass `QueryClient` through router context instead of importing a global singleton.

## Rules

- Create a fresh QueryClient inside the router factory.
- Type it at the root route with `createRootRouteWithContext`.
- Pass it to `createRouter({ context: { queryClient } })`.
- Access it from loaders via `context.queryClient`.
- Keep test QueryClients isolated per render.
