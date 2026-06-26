---
name: tanstack-router
description: "TanStack Router guidance for React apps: file-based routing, generated route trees, root context typing, loaders, loaderDeps, validated search params, navigation, code splitting, error/not-found handling, and Query integration. Use when creating, refactoring, or reviewing TanStack Router routes and route contracts."
---

# TanStack Router

Use this skill to apply current, source-backed TanStack guidance. Keep answers aligned with installed package source first, official TanStack docs second, and package-shipped skills third. Read [references/current-authority.md](references/current-authority.md) when APIs, versions, CLI commands, or source freshness materially affect the task.

## Decision Tree

- **Need a new route?** Use file-based routing and let the plugin/CLI generate `routeTree.gen.ts`.
- **Need shared dependencies?** Type them in `createRootRouteWithContext<T>()` and pass them to `createRouter({ context })`.
- **Need URL state?** Validate with `validateSearch` and use `loaderDeps` for cache keys.
- **Need route data?** Use loaders; use TanStack Query only when Query owns server-state cache.
- **Need route splitting?** Prefer router plugin `autoCodeSplitting: true`; use `.lazy.tsx` as fallback.

## Rules

- [rules/code-splitting.md](rules/code-splitting.md)
- [rules/custom-search-serialization.md](rules/custom-search-serialization.md)
- [rules/data-loading.md](rules/data-loading.md)
- [rules/errors-not-found.md](rules/errors-not-found.md)
- [rules/loader-deps.md](rules/loader-deps.md)
- [rules/navigation.md](rules/navigation.md)
- [rules/root-context.md](rules/root-context.md)
- [rules/route-generation.md](rules/route-generation.md)
- [rules/search-middleware.md](rules/search-middleware.md)
- [rules/search-validation.md](rules/search-validation.md)
- [rules/type-safety.md](rules/type-safety.md)

## Cross-Stack Cautions

- In TanStack Start apps, loaders are isomorphic; server-only work belongs behind server functions, server routes, or server-only modules.
- In Convex apps, Convex reactive queries own live backend state unless the repo intentionally routes through `@convex-dev/react-query`.
- In Clerk apps, Router guards improve UX but server functions, server routes, and Convex functions must enforce authorization.
- In Bun-first repos, prefer Bun commands for local workflow examples unless quoting official docs.

## Verification

- Check exact installed package versions and source when behavior is version-sensitive.
- Use `tanstack search-docs ... --json` or `tanstack doc ... --json` for official docs lookup when the CLI is installed.
- Run the repo's typecheck, route generation, and relevant tests after implementing guidance.
- For skill maintenance, run `python3 tools/skill/quick_validate.py skills/tanstack-router` and `python3 tools/skill/check_tanstack_skills.py --root .` from `dev-skills`.
