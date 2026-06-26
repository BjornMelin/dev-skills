---
name: tanstack-start
description: "TanStack Start guidance for full-stack React apps: server functions, middleware, server routes, execution boundaries, auth, SSR modes, deployment, and Start + Router setup. Use when building or reviewing TanStack Start routes, server logic, auth boundaries, SSR, or deployment behavior."
---

# TanStack Start

Use this skill to apply current, source-backed TanStack guidance. Keep answers aligned with installed package source first, official TanStack docs second, and package-shipped skills third. Read [references/current-authority.md](references/current-authority.md) when APIs, versions, CLI commands, or source freshness materially affect the task.

## Decision Tree

- **Need DB, secret, filesystem, or private API work?** Use a server function or server route; never put it directly in a loader.
- **Need same-origin app RPC?** Use `createServerFn(...).validator(...).handler(...)`.
- **Need public or cross-origin HTTP?** Use Start server routes through route `server.handlers`.
- **Need auth?** Enforce it in server functions/routes or middleware; `beforeLoad` is UX only.
- **Need to know where code runs?** Read `rules/execution-model.md` before designing loaders.

## Rules

- [rules/auth-boundaries.md](rules/auth-boundaries.md)
- [rules/deployment.md](rules/deployment.md)
- [rules/error-handling.md](rules/error-handling.md)
- [rules/execution-model.md](rules/execution-model.md)
- [rules/file-organization.md](rules/file-organization.md)
- [rules/input-validation.md](rules/input-validation.md)
- [rules/middleware.md](rules/middleware.md)
- [rules/selective-ssr.md](rules/selective-ssr.md)
- [rules/server-functions.md](rules/server-functions.md)
- [rules/server-routes.md](rules/server-routes.md)

## Cross-Stack Cautions

- In TanStack Start apps, loaders are isomorphic; server-only work belongs behind server functions, server routes, or server-only modules.
- In Convex apps, Convex reactive queries own live backend state unless the repo intentionally routes through `@convex-dev/react-query`.
- In Clerk apps, Router guards improve UX but server functions, server routes, and Convex functions must enforce authorization.
- In Bun-first repos, prefer Bun commands for local workflow examples unless quoting official docs.

## Verification

- Check exact installed package versions and source when behavior is version-sensitive.
- Use `tanstack search-docs ... --json` or `tanstack doc ... --json` for official docs lookup when the CLI is installed.
- Run the repo's typecheck, route generation, and relevant tests after implementing guidance.
- For skill maintenance, run `python3 tools/skill/quick_validate.py skills/tanstack-start` and `python3 tools/skill/check_tanstack_skills.py --root .` from `dev-skills`.
