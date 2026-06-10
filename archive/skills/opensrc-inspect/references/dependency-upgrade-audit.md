# Dependency Upgrade Audit

Use this workflow when the task is to analyze a package upgrade deeply and
produce a migration brief that enables a hard-cut integration in the host repo.

## Goal

Compare the repo's current dependency version against a target or latest version
using:

- official docs and API references
- release notes and changelogs
- upstream source for both versions
- a deep audit of how the host repo currently uses the package

The output should make it obvious what to change, what to delete, and what new
native package capabilities should replace repo-owned code.

## Workflow

If `/home/bjorn/.codex/skill-support/bin/deps-workbench` exists and the package
is in the Bun/npm lane, start with:

```bash
tmp_json="$(mktemp)"
deps-workbench upgrade-prep --cwd <repo-root> --package <pkg> --out "$tmp_json"
deps-workbench report --input "$tmp_json" --format md
```

Use that output as the fast inventory layer, then continue with the deeper
source, docs, and migration reasoning below.

### 1. Establish Current Repo State

1. Read the repo's manifests and lockfiles.
2. Identify the currently declared version.
3. Resolve the current source with:

```bash
opensrc path <pkg> --cwd <repo-root>
```

4. Verify the returned version from the path or metadata.
5. In Bun repos, assume stale `node_modules` can mislead resolution. If there
   is any ambiguity, pin explicitly instead of trusting auto-detect.

### 2. Establish Target State

1. Identify the target version or latest release from official sources.
2. Read the package's:
   - docs
   - changelog
   - release notes
   - migration guide
   - API reference
3. If the package lives in a monorepo, locate the real package subtree before
   comparing source.

### 3. Fetch Both Source Trees Explicitly

Always pin both versions for upgrade analysis.

```bash
opensrc fetch <pkg>@<current_version> <pkg>@<target_version>
opensrc path <pkg>@<current_version>
opensrc path <pkg>@<target_version>
```

If you do not need cache warmup as a separate step, `opensrc path` alone is
still sufficient because it fetches on cache miss.

For repos:

```bash
opensrc fetch owner/repo@tag owner/repo#branch
opensrc path owner/repo@tag
opensrc path owner/repo#branch
```

### 4. Diff and Inspect the Upstream Source

Use direct tree and file comparison:

```bash
git diff --no-index "$(opensrc path <pkg>@<current>)" "$(opensrc path <pkg>@<target>)"
```

Then inspect surgically with:

- `rg` for renamed or removed APIs
- direct file reads for changed entrypoints
- tests to infer behavioral changes
- README/docs in the source tree to cross-check declared behavior

Look for:

- removed exports
- renamed imports
- changed configuration shapes
- new runtime requirements
- deprecated APIs
- moved package entrypoints
- newly added package-native capabilities

## Host Repo Audit

Search the host repo for all package touchpoints:

- imports
- wrappers
- config files
- adapters
- polyfills
- custom code that overlaps with package capabilities
- tests that encode now-obsolete package behavior

Ask:

- What custom repo code exists only because the older package lacked a feature?
- What package-native capability now replaces that custom code?
- What deprecated or removed APIs are still in use?
- What compatibility branches can be deleted in a hard-cut migration?

## Hard-Cut Rules

Default to one canonical target shape.

- Prefer package-native capabilities over repo-owned custom code.
- Delete obsolete wrappers, shims, adapters, and compatibility helpers.
- Do not preserve dual-shape integrations unless there is a real external
  boundary that requires it.
- Update tests and fixtures to the new canonical package behavior only.
- If a real compatibility boundary exists, name the exact file and reason.

## Required Output Format

Return a concise report with these sections:

### Current and Target

- current declared version
- current resolved source version
- target version
- exact source paths used

### Sources Consulted

- official docs
- changelog or release notes
- upstream source paths
- host repo files searched

### Breaking or Material Changes

- removed or changed APIs
- behavior changes
- config or runtime changes
- migration-sensitive edge cases

### New Capabilities to Adopt

List package-native capabilities in the target version that should replace
custom repo code or reduce maintenance.

### Obsolete Repo Code to Delete

Call out wrappers, helpers, adapters, or legacy config that should be removed
in a hard-cut integration.

### Required Changes

- import changes
- config changes
- runtime changes
- test updates
- docs updates

### Migration Checklist

Provide a short, actionable checklist for implementing the upgrade.

### Verification Checklist

List the commands or checks needed to prove the migration is complete.

### Unknowns

Mark anything that could not be confirmed as `UNVERIFIED`.

## Operating Notes

- Prefer the repo root for `--cwd` in workspaces.
- In Bun repos, explicitly verify resolved versions before analysis.
- Use opensrc for code, not as a substitute for official migration docs.
- If latest is risky or unstable, compare current against the exact intended
  target version instead of defaulting to latest.
- Do not over-automate the final migration decision. The helper is for prep, not
  for replacing source interpretation, API review, or hard-cut reasoning.
