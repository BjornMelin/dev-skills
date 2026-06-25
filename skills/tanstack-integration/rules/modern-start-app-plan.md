# Modern Start + Router + Query app plan

Use this skeleton when planning a full-stack TanStack app. Hydrate exact import names from current docs and installed source before implementation.

## Wiring order

1. Create a per-request `QueryClient` factory with shared defaults.
2. Create the router with route context that carries the request-scoped query client and only sanitized public auth claims needed by client-visible routes.
3. Register SSR Query integration with router dehydration and hydration before rendering.
4. Wrap the app with `RouterProvider` and the matching `QueryClientProvider` for the same client instance.
5. In loaders, derive `loaderDeps` from validated search/path params, call `ensureQueryData(queryOptions(...))`, and set Router `defaultPreloadStaleTime: 0` when Query owns freshness.
6. In route components, use `useSuspenseQuery` for critical prefetched data and non-suspense queries for optional widgets.
7. In server functions, validate input with `.validator(...).handler(...)` and enforce authorization inside the server boundary.
8. In mutations, await targeted invalidation or write returned data with `setQueryData` before navigation.

## Minimal planning skeleton

```tsx
// Pseudocode. Verify exact imports and file names against the repo.
const makeQueryClient = () => new QueryClient({ defaultOptions })

export function createAppRouter(requestContext: RequestContext) {
  const queryClient = makeQueryClient()
  const router = createRouter({
    routeTree,
    defaultPreloadStaleTime: 0,
    context: { queryClient, publicUser: requestContext.publicUser },
  })

  setupRouterSsrQueryIntegration({ router, queryClient })
  return { router, queryClient }
}
```

## Auth and deployment checklist

- Treat `beforeLoad` redirects as UX only. Loaders that need private data must call an authorized server function or server route; loader-only checks must not be treated as authorization.
- Attach authenticated middleware to server functions that need shared request/session extraction, and return typed context through `next({ context })`. Keep provider sessions, tokens, and private auth objects in server-only middleware, functions, or routes; put only sanitized public auth claims in router context.
- Verify plugin order, route generation, SSR rendering, bundle output, and provider-specific runtime requirements with current docs before shipping.
