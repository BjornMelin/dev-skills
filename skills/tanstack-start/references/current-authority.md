# Current Authority and Refresh Procedure

Use this reference when a TanStack API, package version, or example matters to the answer. Treat this file as a source ledger, not a copied docs mirror.

## Authority Order

1. Installed package source and types in the target repo.
2. Official TanStack docs fetched with `tanstack search-docs ... --json` or `tanstack doc ... --json`.
3. Context7 library docs for version-aware lookup.
4. Official package-shipped skills under `node_modules/@tanstack/*/skills`.
5. GitHub source, issues, releases, and examples in `TanStack/router`, `TanStack/query`, and `TanStack/tanstack.com`.
6. Community skills such as `DeckardGer/tanstack-agent-skills` as seed material only.

## Example Evidence Snapshot

- TanStack CLI: `tanstack --version` -> `0.69.5`; use JSON commands for deterministic docs lookup.
- CLI MCP migration: `tanstack mcp` has been removed; use direct CLI commands.
- Installed packages observed in `career-os`: `@tanstack/react-query@5.101.1`, `@tanstack/react-router@1.170.16`, `@tanstack/react-router-ssr-query@1.167.1`, `@tanstack/react-start@1.168.26`, `@tanstack/router-plugin@1.168.18`, `@tanstack/router-cli@1.167.17`.
- Start is documented as RC/stable API posture; pin versions for production and treat upgrades as planned work.
- Official package-shipped skills can lag exact package manifests; verify source before copying examples.
- TanStack Intent is the official package-shipped skill mechanism, but these custom skills remain curated dev-skills guidance.

## Useful Commands

```bash
tanstack libraries --json
tanstack search-docs "server functions validator" --library start --framework react --json
tanstack search-docs "validateSearch loaderDeps" --library router --framework react --json
tanstack search-docs "queryOptions ensureQueryData" --library query --framework react --json
tanstack doc cli mcp-migration --json
opensrc path @tanstack/react-router
opensrc path @tanstack/react-start
opensrc path @tanstack/react-query
```

## Source Paths Worth Checking

- `node_modules/@tanstack/react-start/src/index.ts`
- `node_modules/@tanstack/start-client-core/src/createServerFn.ts`
- `node_modules/@tanstack/start-client-core/src/createMiddleware.ts`
- `node_modules/@tanstack/router-core/src/router.ts`
- `node_modules/@tanstack/router-core/src/searchParams.ts`
- `node_modules/@tanstack/router-core/src/searchMiddleware.ts`
- `node_modules/@tanstack/router-generator/src/config.ts`
- `node_modules/@tanstack/router-plugin/src/vite.ts`
- `node_modules/@tanstack/react-query/src/queryOptions.ts`
- `node_modules/@tanstack/query-core/src/queryClient.ts`
- `node_modules/@tanstack/react-router-ssr-query/src/index.tsx`
- `node_modules/@tanstack/router-ssr-query-core/src/index.ts`

## Known Stale Traps

- Do not use `.inputValidator` in new Start examples; use `.validator(...)`.
- Do not import Start Vite plugin from `@tanstack/start/plugin/vite`; use `@tanstack/react-start/plugin/vite`.
- Do not use deprecated `TanStackRouterVite`; use `tanstackRouter`.
- Do not configure custom search serialization as `search: { serialize, parse }`; use top-level `parseSearch` and `stringifySearch`.
- Do not recommend `tanstack mcp`; it was removed.
- Do not treat Router `beforeLoad` as a data authorization boundary.

Snapshot last verified: 2026-06-25. Treat this as a lead; current installed source and official docs win.
