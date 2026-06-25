# Cache Ownership

Pick one owner for each server-state surface.

## Rules

- Router owns navigation state, params/search validation, route matching, preload, and loader lifecycle.
- Query owns server-state cache, background refetch, mutation state, and invalidation.
- Convex owns live subscription state unless the repo deliberately uses `@convex-dev/react-query`.
- Do not mirror the same live resource in Convex hooks and plain Query keys without a clear invalidation contract.
- Server/client state must remain separate from UI-only component state.
