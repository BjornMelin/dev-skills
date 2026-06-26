# Cache Defaults

Set cache defaults at the `QueryClient` where app-wide behavior is consistent.

## Rules

- Set `staleTime` based on data volatility and UX.
- Use `gcTime` for inactive cache retention; v5 uses `gcTime`, not `cacheTime`.
- Disable noisy retries or focus refetches only when the product behavior requires it.
- Use longer server-side stale times to avoid immediate hydration refetch.
- Document exceptions in the query option factory, not in component comments.
