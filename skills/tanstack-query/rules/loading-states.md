# Loading states and pending UX

- Decide route-level versus component-level loading ownership before adding spinners.
- Prefer Router pending components or Suspense fallbacks for critical route data that is prefetched by loaders.
- Use `isPending` for the first unresolved query state and `isFetching` for background refresh indicators.
- Keep background refresh UI non-blocking; avoid replacing settled content with full-page loading states during refetches.
- Pair Suspense usage with error reset boundaries and route boundaries so retries reset failed queries predictably.
- Do not duplicate the same loading state in the route, parent component, and child component.
