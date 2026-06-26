# loaderDeps

Use `loaderDeps` to declare the exact search-derived dependencies that key loader caching.

## Rules

- Return only the fields the loader actually uses.
- Do not return the whole search object by default.
- Keep deps serializable and stable.
- Include pagination, filters, sort, and route-specific IDs that affect loader output.
- Pair `loaderDeps` with `validateSearch` for typed cache keys.
