# Root Route Context

Use router context for dependency injection into `beforeLoad` and loaders.

## Pattern

```tsx
export const Route = createRootRouteWithContext<{
  queryClient: QueryClient
  user?: User | null
}>()({ component: Root })
```

## Rules

- Type context at the root route with `createRootRouteWithContext<T>()`.
- Pass runtime values through `createRouter({ context })`.
- Do not import global singleton clients into route loaders when SSR is involved.
- Keep context values serializable only when they cross the wire; `QueryClient` stays runtime-only.
