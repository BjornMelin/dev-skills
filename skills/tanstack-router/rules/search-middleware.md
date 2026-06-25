# Search Middleware

Use route-level search middleware to retain or strip selected search params across navigation.

## Rules

- Configure search middleware with `search: { middlewares: [...] }`.
- Use `retainSearchParams` for shared cross-route params such as workspace or locale.
- Use `stripSearchParams` to remove defaults or transient params.
- Keep middleware predictable; avoid silently carrying sensitive params.
- Validate retained params at the route that consumes them.
