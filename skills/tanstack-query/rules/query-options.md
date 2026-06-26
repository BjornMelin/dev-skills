# queryOptions

Use `queryOptions()` to colocate `queryKey`, `queryFn`, and cache options while preserving inference.

## Rules

- Export query option factories near the API or feature they represent.
- Reuse the same factory with `useQuery`, `useSuspenseQuery`, `prefetchQuery`, and Router loaders.
- Keep factories pure and parameterized by every changing input.
- Prefer object signatures; positional v4 hook signatures are stale.
