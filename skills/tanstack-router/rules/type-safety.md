# Type Safety

TanStack Router is strongest when route types are inferred.

## Rules

- Register the router once with module declaration so hooks and `Link` infer valid routes.
- Use route APIs (`Route.useParams`, `Route.useSearch`, `Route.useLoaderData`) in route files.
- Use `getRouteApi('/route')` in code-split or extracted components.
- Use `from` in shared components to narrow route-specific hooks.
- Avoid casts and manual annotations for route-derived types.
