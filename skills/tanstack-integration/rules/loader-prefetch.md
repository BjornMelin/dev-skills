# Loader Prefetch

Use route loaders to prefetch Query-owned data before route render.

## Rules

- Define reusable `queryOptions` factories.
- In loaders, call `context.queryClient.ensureQueryData(options)` for critical data.
- Use `prefetchQuery` for non-critical data that can stream or load later.
- Do not return large query data from the loader when the component will read from Query.
- Use `loaderDeps` when search params affect query options.
