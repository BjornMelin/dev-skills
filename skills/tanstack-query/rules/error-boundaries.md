# Error Boundaries

Design query errors as UI states and recovery paths.

## Rules

- Use Router or React error boundaries for route-critical suspense queries.
- Use `QueryErrorResetBoundary` where retry/reset UX is needed.
- Configure retry behavior based on idempotency and failure type.
- Do not retry non-idempotent mutations blindly.
- Convert server errors into safe user-facing messages.
