# Opensrc CLI Reference

This file captures the current `opensrc` 0.7.x command surface and the cache
model that replaced the older project-local workflow.

## Canonical Baseline

- Local binary verified in this session: `opensrc 0.7.2`
- Architectural cutover: `v0.7.0`
- Current patch baseline: `v0.7.2`

## Command Surface

### `opensrc fetch`

Cache-prewarm command. Use when you want sources downloaded without printing
paths.

```bash
opensrc fetch zod
opensrc fetch --cwd /path/to/project zod react next
opensrc fetch <pkg>@<current_version> <pkg>@<target_version>
opensrc fetch bitbucket:workspace/private-repo
```

Options:
- `--cwd <path>`: working directory for lockfile version resolution
- `--quiet` / `-q`: suppress progress output

### `opensrc path`

Primary composition command. Prints the absolute path to source and fetches on
cache miss.

```bash
opensrc path zod
opensrc path pypi:requests
opensrc path crates:serde
opensrc path vercel/next.js

rg "parse" $(opensrc path zod)
cat $(opensrc path zod)/src/types.ts
find $(opensrc path pypi:requests) -name "*.py"
git diff --no-index "$(opensrc path <pkg>@<current_version>)" "$(opensrc path <pkg>@<target_version>)"
```

Options:
- `--cwd <path>`: working directory for lockfile version resolution
- `--verbose`: show fetch progress

Multiple specs print one path per line:

```bash
opensrc path zod react next
opensrc path pypi:requests pypi:flask
opensrc path crates:serde crates:tokio
```

### `opensrc list`

Lists the global cache index.

```bash
opensrc list
opensrc list --json
```

### `opensrc remove`

Removes one or more cached entries. `rm` is an alias.

```bash
opensrc remove zod
opensrc remove pypi:requests
opensrc remove vercel/ai
opensrc rm github:owner/repo
```

### `opensrc clean`

Removes cache contents by category.

```bash
opensrc clean
opensrc clean --packages
opensrc clean --repos
opensrc clean --npm
opensrc clean --pypi
opensrc clean --crates
```

## Supported Spec Forms

Documented/current forms:

- npm: `zod`
- npm alias: `npm:zod`
- PyPI: `pypi:requests`
- PyPI aliases: `pip:requests`, `python:requests`
- crates.io: `crates:serde`
- crates.io aliases: `cargo:serde`, `rust:serde`
- GitHub shorthand: `owner/repo`
- explicit hosts: `github:owner/repo`, `gitlab:owner/repo`, `bitbucket:owner/repo`
- full repo URLs
- package version pins: `zod@4.3.6`, `pypi:flask@3.0.0`
- repo refs: `owner/repo@v1.0.0`, `owner/repo#main`

## Cache Model

`opensrc` 0.7.x uses a global cache, not a project-local `opensrc/` folder.

Default root:

```text
~/.opensrc/
```

Override:

```bash
export OPENSRC_HOME=/custom/cache/path
```

Typical layout:

```text
~/.opensrc/
├── repos/
│   └── github.com/
│       └── owner/
│           └── repo/
│               └── version-or-ref/
└── sources.json
```

The cache key includes the resolved version or ref. Different repos can safely
use different cached versions at the same time.

## Version Resolution

Upstream code currently resolves npm-family package versions in this order:

1. `node_modules/<pkg>/package.json`
2. `package-lock.json`
3. `pnpm-lock.yaml`
4. `yarn.lock`
5. `package.json`

Implications:

- The main failure mode is stale local install state, not the global cache.
- In Bun repos, stale `node_modules` can win before any lockfile parsing.
- Use the repo root as `--cwd` by default in workspaces.
- Verify the returned path/version before trusting the result.
- For upgrade or migration work, pin both versions explicitly.

### Documented vs Observed

Documented:
- npm lockfile and install detection only mention `node_modules`,
  `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, and `package.json`.

Observed in upstream `v0.7.2` source:
- version resolution was substantially rewritten for pnpm workspaces and Yarn
  workspace/protocol edge cases
- transitive pnpm resolution now has dedicated fixture coverage
- workspace/link/file/git protocol specs are explicitly filtered so they do not
  get treated as registry versions

Observed in this session:
- `opensrc path react --cwd /home/bjorn/repos/signr` resolved `19.2.0`
- `opensrc path zod --cwd /home/bjorn/repos/signr` resolved `4.3.6`

Interpretation:
- opensrc can still work correctly in Bun workspaces when `node_modules` is
  current.
- Do not claim documented `bun.lock` support unless upstream docs/code say so.

## Auth and Environment

Current repo-auth environment variables:

- `GITHUB_TOKEN`
- `GITLAB_TOKEN`
- `BITBUCKET_TOKEN`
- `OPENSRC_HOME`

Do not use the older provider-specific legacy token variable names from the
pre-0.7 workflow.

## Release Deltas That Matter

### `v0.7.0`

- Rust rewrite with native binary
- global cache at `~/.opensrc/`
- `opensrc path` as the primary command
- docs site added at `opensrc.sh`
- repo reorganized into CLI plus docs monorepo
- cross-platform binary releases

### `v0.7.1`

- private repo support documented around standard `GITHUB_TOKEN` and
  `GITLAB_TOKEN`
- `remove` accepts the same repo formats as the fetch/path-facing specs

### `v0.7.2`

- dedicated `opensrc fetch` subcommand for cache-only workflows
- Bitbucket Cloud repo support plus `BITBUCKET_TOKEN`
- auth docs page covering GitHub, GitLab, and Bitbucket tokens
- pnpm workspace and transitive lockfile parser rewrite
- Yarn workspace/protocol edge-case coverage
- upstream official skill moved to top-level `skills/opensrc/`

## Operational Rules

- Prefer `opensrc path` when the next step composes the resolved path with shell
  tools.
- Prefer `opensrc fetch` when you only need the cache primed for CI, scripts,
  or multi-version prep.
- Prefer direct `opensrc` invocation. Use `bunx opensrc` only as a fallback.
- For current-versus-target analysis, pin both versions explicitly. Optional:
  prewarm with `opensrc fetch <pkg>@<current> <pkg>@<target>` before diffing.
- Use web/docs sources for official changelogs and API contracts.
- Use opensrc for implementation internals and tree diffs.
- Do not claim documented `bun.lock` support unless upstream docs/code say so.
