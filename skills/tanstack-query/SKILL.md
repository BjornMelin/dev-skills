---
name: tanstack-query
description: "TanStack Query v5 guidance for React server-state: queryOptions, query keys, caching, mutations, invalidation, optimistic updates, SSR hydration, error boundaries, disabling/lazy queries, infinite queries, and Router/Start integration. Use when designing or reviewing React Query data flows."
---

# TanStack Query

Use this skill to apply current, source-backed TanStack guidance. Keep answers aligned with installed package source first, official TanStack docs second, and package-shipped skills third. Read [references/current-authority.md](references/current-authority.md) when APIs, versions, CLI commands, or source freshness materially affect the task.

## Decision Tree

- **Need server-state cache?** Use Query with stable `queryOptions` and hierarchical keys.
- **Need live Convex subscriptions?** Let Convex own live state; do not duplicate it into Query unless using the Convex Query adapter intentionally.
- **Need route prefetch?** Use the same `queryOptions` in loaders and components.
- **Need mutation UI?** Choose invalidation, optimistic cache update, or UI-level optimistic variables based on scope.

## Rules

- Read `rules/loading-states.md` for first-load, background-refresh, Suspense, and pending UI ownership.
- Read `rules/mutations.md` for `useMutation`, lifecycle callbacks, shared pending UI, and side-effect placement.
- [rules/cache-defaults.md](rules/cache-defaults.md)
- [rules/disabling-queries.md](rules/disabling-queries.md)
- [rules/error-boundaries.md](rules/error-boundaries.md)
- [rules/eslint.md](rules/eslint.md)
- [rules/infinite-queries.md](rules/infinite-queries.md)
- [rules/invalidation.md](rules/invalidation.md)
- [rules/key-factories.md](rules/key-factories.md)
- [rules/optimistic-updates.md](rules/optimistic-updates.md)
- [rules/query-keys.md](rules/query-keys.md)
- [rules/query-options.md](rules/query-options.md)
- [rules/ssr-hydration.md](rules/ssr-hydration.md)

## Cross-Stack Cautions

- In TanStack Start apps, loaders are isomorphic; server-only work belongs behind server functions, server routes, or server-only modules.
- In Convex apps, Convex reactive queries own live backend state unless the repo intentionally routes through `@convex-dev/react-query`.
- In Clerk apps, Router guards improve UX but server functions, server routes, and Convex functions must enforce authorization.
- In Bun-first repos, prefer Bun commands for local workflow examples unless quoting official docs.

## Verification

- Check exact installed package versions and source when behavior is version-sensitive.
- Use `tanstack search-docs ... --json` or `tanstack doc ... --json` for official docs lookup when the CLI is installed.
- Run the repo's typecheck, route generation, and relevant tests after implementing guidance.
- For skill maintenance, run `python3 tools/skill/quick_validate.py skills/tanstack-query` and `python3 tools/skill/check_tanstack_skills.py --root .` from `dev-skills`.


