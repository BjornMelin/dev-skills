---
description: Operating mode for Bun-native repos and Bun-managed workspaces. Use Bun as the primary package-manager, audit, and dependency-analysis surface.
---

# Bun-First Variant

Use this reference when Bun is the repo-native package manager and `bun.lock` is
the authoritative lockfile.

## Operating Assumptions

- Use Bun for actual dependency changes unless the repo explicitly says
  otherwise.
- Treat `bun.lock` as the source of truth.
- Prefer Bun workspace and package-management behavior over npm/pnpm/yarn
  equivalents.

## Tool Priority

1. Bun CLI help and docs first:
   - `bun --help`
   - `bun install --help`
   - `bun update --help`
   - `bun audit --help`
   - `bun pm --help`
2. Bun package analysis:
   - `bun audit`
   - `bun outdated`
   - `bun info <pkg> version --json`
   - `bun pm ls`
   - `bun pm why`
   - `bun pm pkg`
   - `/home/bjorn/.codex/skill-support/bin/deps-workbench inventory`
   - `/home/bjorn/.codex/skill-support/bin/deps-workbench upgrade-prep`
3. Bun-native dependency mutation:
   - `bun install`
   - `bun add`
   - `bun remove`
   - `bun update`

## Extra Bun-Specific Expectations

- Check Bun overrides/resolutions support before introducing transitive pins.
- Check Bun release notes if command behavior seems surprising.
- Use Bun docs as the primary source for package-manager behavior, not npm docs.
- If runtime execution or test behavior changes after upgrades, verify whether
  the issue is Bun-runtime specific versus generic JS ecosystem breakage.
- Use `deps-workbench upgrade-prep` to gather Bun outdated, audit, why, usage,
  and opensrc-path context quickly before invoking deeper source analysis.

## Cleanup Bias

- Remove package-manager cross-talk:
  - stale npm/yarn/pnpm command snippets
  - obsolete lockfiles if policy allows
  - script wrappers that only exist to compensate for older package-manager
    behavior
- Collapse duplicated package-manager logic into Bun-native flows where the repo
  already committed to Bun.

## Verification Bias

- Re-run Bun-based audit and outdated checks after the migration.
- Confirm repo scripts that rely on Bun still work with the upgraded set.
- Where the repo uses Bun runtime directly, verify runtime-facing behavior in
  addition to dependency resolution.
