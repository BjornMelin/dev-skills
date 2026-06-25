# Auth Boundaries

Route guards are not data security. TanStack Router `beforeLoad` can redirect before UI renders, but server functions and server routes remain independently callable endpoints.

## Rules

- Use `beforeLoad` for UX gating and redirect preservation.
- Enforce private-data access inside server functions, server routes, or server middleware.
- For Clerk or another auth provider, map the request/session identity to the server-side principal before reading or writing private data.
- Apply CSRF middleware to mutation surfaces when cookies/session credentials are in play.
- Do not trust client-sent organization, workspace, or tenant IDs without server-side membership checks.
