# Auth and Data Boundaries

Combine Router UX guards with real server-side authorization.

## Rules

- Use Router `beforeLoad` to redirect unauthenticated users early.
- Enforce private data authorization in Start server functions/server routes and Convex functions.
- With Clerk, validate server-side identity before using tenant/org IDs.
- Do not assume hydrated client context proves access.
