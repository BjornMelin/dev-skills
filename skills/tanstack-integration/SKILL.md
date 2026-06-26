---
name: tanstack-integration
description: "Integration guidance for TanStack Start, Router, and Query together: QueryClient router context, router SSR query setup, loader prefetching with ensureQueryData, suspense reads, cache ownership, invalidation after mutations, streaming caveats, and Convex/Clerk/Bun integration cautions."
---

# TanStack Integration

Use this skill to apply current, source-backed TanStack guidance. Keep answers aligned with installed package source first, official TanStack docs second, and package-shipped skills third. Read [references/current-authority.md](references/current-authority.md) when APIs, versions, CLI commands, or source freshness materially affect the task.

## Decision Tree

- **Need Start + Router + Query SSR?** Use a fresh per-request QueryClient, router context, and `setupRouterSsrQueryIntegration`.
- **Need loader data in components?** Use `ensureQueryData` in loader and `useSuspenseQuery` in component.
- **Need Convex live data?** Let Convex own live subscriptions; use Query only through deliberate adapter/query ownership.
- **Need auth?** Use Router guards for UX and Start/Convex/server boundaries for private data.

## Rules

- Read `rules/modern-start-app-plan.md` for the combined Start + Router + Query SSR wiring skeleton and checklist.
- [rules/auth-and-data-boundaries.md](rules/auth-and-data-boundaries.md)
- [rules/bun-and-tooling.md](rules/bun-and-tooling.md)
- [rules/cache-ownership.md](rules/cache-ownership.md)
- [rules/loader-prefetch.md](rules/loader-prefetch.md)
- [rules/mutation-invalidation.md](rules/mutation-invalidation.md)
- [rules/query-client-context.md](rules/query-client-context.md)
- [rules/ssr-query-integration.md](rules/ssr-query-integration.md)
- [rules/suspense-components.md](rules/suspense-components.md)

## Cross-Stack Cautions

- In TanStack Start apps, loaders are isomorphic; server-only work belongs behind server functions, server routes, or server-only modules.
- In Convex apps, Convex reactive queries own live backend state unless the repo intentionally routes through `@convex-dev/react-query`.
- In Clerk apps, Router guards improve UX but server functions, server routes, and Convex functions must enforce authorization.
- In Bun-first repos, prefer Bun commands for local workflow examples unless quoting official docs.

## Verification

- Check exact installed package versions and source when behavior is version-sensitive.
- Use `tanstack search-docs ... --json` or `tanstack doc ... --json` for official docs lookup when the CLI is installed.
- Run the repo's typecheck, route generation, and relevant tests after implementing guidance.
- For skill maintenance, run `python3 tools/skill/quick_validate.py skills/tanstack-integration` and `python3 tools/skill/check_tanstack_skills.py --root .` from `dev-skills`.


