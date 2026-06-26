# SSR Hydration

Use prefetch/dehydrate/hydrate to avoid duplicate initial client fetches.

## Rules

- Create a fresh `QueryClient` per request.
- Prefetch or ensure route-critical queries on the server.
- Dehydrate serializable cache state and hydrate on the client.
- In Router/Start apps, prefer `@tanstack/react-router-ssr-query` integration over manual boilerplate.
- Be aware of current streaming limitations and verify behavior for pending queries.
