---
description: Operating mode for pnpm + Turborepo monorepos. Use pnpm for dependency truth and turbo for graph-aware upgrade sequencing and verification.
---

# pnpm + Turbo Monorepo Variant

Use this reference when `pnpm-lock.yaml` and Turbo define the actual workspace
graph.

## Operating Assumptions

- Use pnpm for actual dependency mutations.
- Use Turbo for impact-aware validation.
- Treat workspace boundaries, shared package ripple effects, and pipeline
  ordering as first-class concerns.

## Tool Priority

1. Detect workspace truth:
   - `pnpm-workspace.yaml`
   - `turbo.json`
   - root `package.json`
   - per-package manifests
2. Use pnpm for dependency mutation and inventory:
   - install/update/remove/outdated/list commands as defined by the repo
3. Use Turbo for graph-aware validation:
   - filtered runs by package/app
   - targeted builds, tests, lint, and typecheck
4. Use Bun as an additional audit/research surface where helpful, but not as
   the source of lockfile truth.

## Monorepo Upgrade Bias

- Build a workspace dependency matrix, not just a flat package list.
- Identify:
  - shared deps pinned across many packages
  - framework-specific version alignment requirements
  - internal package contracts affected by external dependency changes
- Upgrade in waves that follow the graph:
  1. shared libs and config packages
  2. framework/runtime packages
  3. apps and leaf workspaces

## Cleanup Bias

- Delete duplicated utilities spread across workspaces when one internal package
  or upgraded external dependency can own the concern.
- Remove stale package-level exceptions once the shared dependency baseline is
  modernized.
- Collapse redundant scripts if Turbo can own orchestration directly.

## Verification Bias

- Use Turbo filters to validate only the impacted graph first, then run the
  broader repo validation.
- Confirm that lockfile, workspace ranges, and build graph remain internally
  consistent after upgrades.
- Watch for package drift where one workspace silently upgrades a dependency
  that others still assume is older.
