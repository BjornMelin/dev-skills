# Errors and Not Found

Handle route failures with Router error and not-found contracts.

## Rules

- Configure useful root defaults for error and not-found components.
- Throw `notFound()` for missing route resources when the URL is valid but data is absent.
- Throw `redirect(...)` from `beforeLoad` for auth or canonicalization.
- Keep route error UI accessible and recovery-oriented.
- Log server-side details without leaking secrets to client error components.
