---
name: opensrc
description: "Inspect dependency/upstream source with opensrc CLI. Trigger for implementation beyond docs/types, package internals, source paths, version diffs, upgrade audits, npm/PyPI/crates/repo source. Avoid general web or release-note-only tasks."
allowed-tools: Bash(opensrc:*)
---

# Opensrc

Use this skill when source-level dependency inspection materially changes the
answer. Prefer docs and types first; pull source when behavior, migration risk,
or implementation details matter.

## Core Workflow

1. Read the repo `AGENTS.md` and inspect relevant manifests and lockfiles.
2. Use the global `opensrc` binary. Fall back to `bunx opensrc` only if the
   binary is unavailable.
3. Use `opensrc fetch` when the goal is cache-only prep:
   ```bash
   opensrc fetch --cwd <repo-root> zod react next
   opensrc fetch <pkg>@<current_version> <pkg>@<target_version>
   ```
4. Use `opensrc path` when the next command needs a filesystem path:
   ```bash
   rg "pattern" "$(opensrc path zod)"
   cat "$(opensrc path zod)"/src/types.ts
   find "$(opensrc path pypi:requests)" -name "*.py"
   git diff --no-index "$(opensrc path <pkg>@<current>)" "$(opensrc path <pkg>@<target>)"
   ```
5. Cite exact versions and local source paths when source evidence affects the
   recommendation.
6. Use web/docs sources for release notes, API references, changelogs, and
   migration guides; use `opensrc` for implementation internals and source
   diffs.

## Version Guardrails

- `opensrc` 0.7.x caches globally at `~/.opensrc/`; `OPENSRC_HOME` overrides it.
- The cache key includes the resolved package version or repo ref. The main risk
  is incorrect version resolution before fetch, not the global cache itself.
- For npm packages, current upstream resolution checks `node_modules`, then
  `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, then `package.json`.
- In Bun or workspace repos, use the workspace root as `--cwd` by default. If
  `node_modules` may be stale, pin versions explicitly.
- For upgrade work or ambiguity, inspect pinned versions:
  ```bash
  opensrc path pkg@current_version
  opensrc path pkg@target_version
  ```

## Supported Specs

- npm: `zod`, `npm:zod`
- PyPI: `pypi:requests`, `pip:requests`, `python:requests`
- crates.io: `crates:serde`, `cargo:serde`, `rust:serde`
- repos: `owner/repo`, `github:owner/repo`, `gitlab:owner/repo`,
  `bitbucket:owner/repo`, or full URLs
- pinned refs: `pkg@version`, `owner/repo@tag`, `owner/repo#branch`
- private repo auth: `GITHUB_TOKEN`, `GITLAB_TOKEN`, `BITBUCKET_TOKEN`

## Upgrade Audits

For npm/Bun dependency upgrades, if this helper exists, use it as the fast
inventory layer before deeper source reasoning:

```bash
deps-workbench upgrade-prep --cwd <repo-root> --package <pkg> --out <tmp.json>
deps-workbench report --input <tmp.json> --format md
```

Then compare current and target with official docs plus pinned source paths.
Prefer package-native capabilities, delete obsolete wrappers/shims/adapters, and
avoid dual-shape compatibility unless a real boundary requires it.

## References

- Read `references/opensrc-cli-reference.md` for exact CLI surface, cache model,
  supported spec forms, auth env vars, and release deltas.
- Read `references/dependency-upgrade-audit.md` for package upgrade, migration,
  current-versus-target, or hard-cut audits.

## Do Not Use For

- broad web research that does not require source inspection
- release-note summaries where docs alone answer the question
- simple API usage questions that types and official docs already resolve

## Output

Include the resolved current and target versions, exact local source paths used,
and whether source inspection changed the conclusion. For upgrade work, include
a concise hard-cut migration brief and verification checklist.
