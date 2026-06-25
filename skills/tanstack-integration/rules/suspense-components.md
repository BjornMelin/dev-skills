# Suspense Components

Read loader-prefetched Query data with suspense-aware hooks.

## Rules

- Use `useSuspenseQuery` when loader prefetch guarantees data for render.
- Use `useQuery` for optional or client-only data that may legitimately load after hydration.
- Keep Suspense boundaries intentional and route-level where possible.
- Avoid duplicate local loading state for route-critical prefetched data.
