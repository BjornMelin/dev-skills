# Signr guidance

Use this when applying `convex-helpers` inside the Signr repo.

## Repo boundaries

- Backend state and policy live in `packages/backend/convex/**`.
- Apps consume only `@signr/backend/api` and `@signr/backend/dataModel`.
- Shared code stays deterministic and runtime-agnostic; do not pull Convex
  helpers into `packages/shared`.
- Generated Convex files are read-only.
- Convex-loaded module path segments must be alphanumeric, `_`, or `.` only.

## Preferred uses in Signr

- Custom functions for org-scoped `query`, `mutation`, `action`, internal
  functions, operator-only functions, and test-only functions gated by env.
- Triggers for app-owned maintenance rows, denormalized read models, aggregate
  component triggers, and invariant-preserving cascades.
- Relationship helpers for small, obvious normalized joins in non-hot paths.
- Validators for schema-derived argument/doc validators and runtime parsing.
- Streams or custom pagination for bounded multi-index read composition where
  native Convex pagination is insufficient.
- CORS/Hono only in `http.ts` when native `httpRouter` becomes noisier than the
  helper and security wrappers remain explicit.
- CLI generators only for external-repo integration or non-TS clients; do not
  replace normal generated `api` imports inside Signr apps.

## Avoid in Signr

- Public `crud()` exports without explicit authz and review.
- Helper `filter` over broad tenant/product tables when an index or read model
  should exist.
- Anonymous sessions on protected Signr workspaces unless the product flow is
  explicitly guest-capable.
- Query cache as a default performance fix. Measure UX and bandwidth tradeoff.
- Helper rate limits if the existing `@convex-dev/rate-limiter` component is the
  canonical path for that surface.
- RLS wrappers that make endpoint auth look optional. Endpoint auth remains
  explicit; DB wrapping is defense in depth.

## Review checklist

- Does every protected read/write prove user, org, tenant, role, and entitlement
  server-side?
- Is the primary read indexed before helper filtering or relationship traversal?
- Could native Convex query/pagination do the same job with less code?
- Are triggers guaranteed to run through a custom mutation builder, and are raw
  builders restricted or linted?
- Is there any dashboard/import path that needs a separate reconciliation job
  because triggers will not run?
- Does any generated API/spec expose internal functions, private table fields,
  or non-product errors?
- Does the helper introduce another source of truth for validation, state,
  status, auth, or routing?

## Version refresh

When `convex-helpers` changes, refresh before relying on this skill:

```bash
opensrc path --cwd . convex-helpers
bun pm view convex-helpers version --json
```

Then compare `package.json` exports, README headings, and `CHANGELOG.md` recent
entries. Update `package-map.md` first, then narrower references.
