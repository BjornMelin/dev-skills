# Server Routes

Use Start server routes for public HTTP endpoints, webhooks, cross-origin APIs, downloads, and non-RPC integrations.

## Rules

- Declare handlers through route `server.handlers` on `createFileRoute`.
- Use server routes for GitHub/Stripe/Clerk-style webhooks and verify signatures in the handler.
- Keep server functions for app-internal same-origin RPC.
- Return explicit status, headers, and body shape.
- Keep route params/search validation separate from webhook body validation.
