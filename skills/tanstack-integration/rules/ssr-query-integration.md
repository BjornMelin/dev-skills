# Router SSR Query Integration

Use `@tanstack/react-router-ssr-query` for Router + Query hydration in Start/SSR apps.

## Current API

```tsx
setupRouterSsrQueryIntegration({
  router,
  queryClient,
  dehydrateOptions,
  hydrateOptions,
  handleRedirects: true,
})
```

## Rules

- Prefer the integration package over hand-written dehydrate/hydrate boilerplate.
- Keep `handleRedirects` enabled unless a route-specific reason says otherwise.
- The integration wraps with `QueryClientProvider` by default. Set `wrapQueryClient: false` only when the app already provides its own QueryClientProvider or custom wrapper.
- Track streaming behavior carefully; TanStack Router issue #7529 is a known caution area.
- When Query owns loader data freshness, configure Router with `defaultPreloadStaleTime: 0` unless the repo intentionally keeps Router preload caching in front of Query.
