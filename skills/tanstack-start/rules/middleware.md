# Middleware

Use `createMiddleware` for cross-cutting request or server-function behavior.

## Current APIs

- Request middleware: `createMiddleware().server(...)`
- Function middleware: `createMiddleware({ type: 'function' }).client(...).server(...)`
- Attach middleware before `.handler(...)`; current types support either order relative to `.validator(...)`, but verify inference in the target repo.

## Rules

- Pass typed context through `next({ context })`.
- Validate `sendContext` shape, then authorize it server-side before use.
- Keep request middleware server-only.
- Use function middleware for client/server phases around a specific server function family.
- Prefer small composable middleware over one global catch-all.

## Authenticated middleware pattern

- Extract request/session state once in server middleware when multiple server functions share the same auth requirement.
- Return failures before `next()` for unauthenticated or malformed sessions; do not pass nullable private context downstream.
- Pass only the typed user/session fields needed by handlers through `next({ context })`.
- Attach the middleware explicitly to each protected server function and still validate function input separately.
- Keep redirects in route `beforeLoad` for UX; keep authorization decisions in server middleware or handlers.

