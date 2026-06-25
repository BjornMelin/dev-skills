# Search Validation

Search params are user-controlled URL input. Validate before use.

## Rules

- Define `validateSearch` on routes that consume search params.
- With Zod v4, use `.catch(...)` or defaults to recover malformed URLs.
- Keep validation fast; it runs during navigation.
- Use parent search params only when inheritance is intentional.
- Do not read raw `window.location.search` in components for route state.
