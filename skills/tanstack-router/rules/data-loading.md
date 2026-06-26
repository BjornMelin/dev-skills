# Data Loading

Use route loaders for navigation-critical data and preloading.

## Rules

- Use `beforeLoad` for guards and context extension before loaders.
- Use loaders for route data, route preloading, and suspense coordination.
- Use Query `ensureQueryData` in loaders only when TanStack Query owns that server-state cache.
- Split critical awaited data from non-critical prefetched or deferred data.
- Keep server-only logic behind Start server functions in Start apps.
