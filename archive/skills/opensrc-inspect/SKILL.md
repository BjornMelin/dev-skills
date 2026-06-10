---
name: opensrc-inspect
description: opensrc CLI—dep + upstream source. Triggers—impl beyond docs/types, version compare, upgrade diff audit, prewarm w/ `opensrc fetch` then `opensrc path`. Not general web or release-note-only.
---

# Opensrc Inspect

Use this skill when source-level dependency inspection materially changes the
answer. Prefer docs and types first; pull source when behavior, migration risk,
or internal implementation details matter.

## Core Workflow

1. Read the repo `AGENTS.md` and inspect the repo's manifests and lockfiles.
2. Use the global `opensrc` binary when available. Fall back to `bunx opensrc`
   only if the binary is unavailable.
3. Use `opensrc fetch` when the goal is cache-only prep for CI, scripts, or
   multi-package warmup:
   - `opensrc fetch --cwd <repo-root> zod react next`
   - `opensrc fetch <pkg>@<current_version> <pkg>@<target_version>`
4. Use `opensrc path` for actual inspection, composition, and diffs:
   - `rg "pattern" $(opensrc path zod)`
   - `cat $(opensrc path zod)/src/types.ts`
   - `find $(opensrc path pypi:requests) -name "*.py"`
   - `git diff --no-index "$(opensrc path <pkg>@<current_version>)" "$(opensrc path <pkg>@<target_version>)"`
5. Cite exact versions and local paths when source evidence affects the
   recommendation.
6. Use web/docs sources for release notes, API references, and changelogs;
   use opensrc for implementation internals and source diffs.
7. If `/home/bjorn/.codex/skill-support/bin/deps-workbench` exists and the task
   is an npm/Bun dependency upgrade, use it as the fast prep layer before the
   deeper source reasoning:
   - `deps-workbench upgrade-prep --cwd <repo-root> --package <pkg> --out <tmp.json>`
   - `deps-workbench report --input <tmp.json> --format md`

## Version Resolution Guardrails

- `opensrc` 0.7.x caches globally at `~/.opensrc/` and keys cache entries by
  resolved version or ref. The global cache is not the main risk.
- The real risk is incorrect version resolution before fetch.
- `opensrc` 0.7.2 improved pnpm workspace and Yarn workspace/protocol handling,
  but you still need to verify the resolved version before trusting it.
- For npm-family packages, current upstream code checks:
  1. `node_modules/<pkg>/package.json`
  2. `package-lock.json`
  3. `pnpm-lock.yaml`
  4. `yarn.lock`
  5. `package.json`
- In Bun repos, use the workspace root as `--cwd` by default. If `node_modules`
  is stale, opensrc can resolve a stale installed version first.
- Verify the resolved version from the returned path before trusting it.
- For upgrade work or any ambiguity, pin versions explicitly:
  - `opensrc path pkg@current_version`
  - `opensrc path pkg@target_version`

## Cache and Specs

- Source is cached globally in `~/.opensrc/`. `OPENSRC_HOME` overrides the
  cache root.
- Metadata lives in `~/.opensrc/sources.json`, not in the project.
- Use current spec forms only:
  - npm: `zod` with accepted alias `npm:zod`
  - PyPI: `pypi:requests` with accepted aliases `pip:requests`,
    `python:requests`
  - crates.io: `crates:serde` with accepted aliases `cargo:serde`,
    `rust:serde`
  - repos: `owner/repo`, `github:owner/repo`, `gitlab:owner/repo`,
    `bitbucket:owner/repo`, or full URLs
  - pinned refs: `owner/repo@tag`, `owner/repo#branch`, `pkg@version`
- Private repo auth uses `GITHUB_TOKEN`, `GITLAB_TOKEN`, and
  `BITBUCKET_TOKEN`.

## Prefer `fetch` vs `path`

- Use `opensrc fetch` when you want deterministic cache warmup without printing
  paths, especially in CI, prep scripts, or before a multi-version comparison.
- Use `opensrc path` when the next command needs the resolved filesystem path.
- For current-versus-target analysis, prewarming with `fetch` is optional; the
  important rule is that the actual inspection uses explicitly pinned versions.

## Load These References When Needed

- Read `references/opensrc-cli-reference.md` when you need the exact modern CLI
  surface, cache model, supported spec forms and aliases, auth env vars, or
  release deltas.
- Read `references/dependency-upgrade-audit.md` when the task is a package
  upgrade, current-versus-target comparison, migration audit, or hard-cut
  removal of obsolete package integrations.
- Use the shared helper only to gather fast inventory, Bun signals, usage hits,
  and current/target opensrc paths. Keep the actual migration reasoning with
  the model.

## Do Not Use This Skill For

- broad web research that does not require source inspection
- release-note summaries where docs alone answer the question
- simple API usage questions that types and official docs already resolve

## Outputs

- resolved current and target versions
- exact local source paths used for analysis
- concise note on whether source inspection changed the conclusion
- for upgrade work, a hard-cut migration brief with obsolete code to remove
