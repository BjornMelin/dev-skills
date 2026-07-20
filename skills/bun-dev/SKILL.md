---
name: bun-dev
description: "Definitive Bun development, build, and runtime skill: package manager and lockfiles, Node-to-Bun migration, monorepo scripts and workspaces, bun test and bun build, TypeScript config, Bun-native APIs (Bun.serve, Bun.file, Bun.$ shell, bun:sqlite), and Vercel Bun-runtime Functions, plus an optional native audit/fix/validate engine (codex-dev bun). Use when adopting or migrating to Bun, choosing Bun vs Node as runtime or package manager, auditing or fixing a repo's Bun posture, resolving lockfile or mixed-package-manager drift, configuring bunfig.toml or tsconfig for Bun, running or scaling bun test, bundling or compiling with bun build, or deploying Bun Functions on Vercel."
metadata:
  version: "2.0"
---

# Bun Dev

The single Bun skill: self-contained knowledge (below) + an opinionated rule set
(`rules/`) + an optional native audit/fix engine (`codex-dev bun`). It works with zero
tooling; the Power Tools section adds automation when `codex-dev` is installed.

## Product summary

Bun is an all-in-one JavaScript/TypeScript toolkit: a fast runtime (drop-in Node.js
replacement on JavaScriptCore), package manager, bundler, and test runner, all shipped as
one `bun` binary. Key files: `bunfig.toml` (config), `bun.lock` (text lockfile, default
since Bun 1.2), `package.json`. Primary commands: `bun run`, `bun install` / `bun ci`,
`bun build`, `bun test`. Full docs: <https://bun.com/docs> (agent index:
<https://bun.com/docs/llms.txt>).

## When to use

- **Runtime**: execute `.ts/.tsx/.jsx/.js` directly (`bun run <file>` or `bun <file>`).
- **Package manager**: `bun install` / `bun add` / `bun ci`; lockfile + workspace mgmt.
- **Bundler**: `bun build` for browsers/servers; single-file executables (`--compile`).
- **Test runner**: Jest-compatible `bun test` with scale flags for CI.
- **Servers / IO**: `Bun.serve()`, `Bun.file()`, `Bun.write()`, `bun:sqlite`, `Bun.$`.
- **Monorepos**: `bun install`, `bun run --filter`, `--workspaces`, `--parallel`.
- **Vercel**: deploy Functions on the Bun runtime (Beta).

## Quick reference

Essential commands (full cheatsheet: `references/ref-bun-cli-cheatsheet.md`):

| Task | Command |
|------|---------|
| Run a file / script | `bun index.ts` / `bun run start` |
| Install deps | `bun install` (writes `bun.lock`) |
| Deterministic CI install | `bun ci` |
| Add / remove | `bun add react` / `bun add -d @types/bun` / `bun remove react` |
| Audit deps | `bun audit` |
| Run tests | `bun test` (`--coverage`, `--shard=M/N`, `--changed`) |
| Build a bundle | `bun build ./index.ts --outdir ./dist` |
| Execute a package | `bunx <bin>` |

### Decision guidance

`bun run` vs `bun <file>`:

| Scenario | Use |
|----------|-----|
| Script from `package.json` | `bun run start` |
| A file directly | `bun index.ts` (or `bun run index.ts`) |
| A system command / package bin | `bun run <cmd>` / `bunx <bin>` |

`hoisted` vs `isolated` linker (see `pm-linker-and-streaming-install`):

| Linker | Use when |
|--------|----------|
| `hoisted` | Traditional flat `node_modules`; default for single packages. |
| `isolated` | Strict, pnpm-style isolation; prevents phantom deps; faster in monorepos. |

`Bun.serve()` vs a framework, and `bun build` vs `bun run`:

| Choice | Use when |
|--------|----------|
| `Bun.serve()` | Simple APIs/static servers, zero deps (Bun runtime only, not Vercel). |
| Express/Hono/Elysia | Middleware, validation, ecosystem integrations. |
| `bun run` | Executing source directly (dev, scripts, CLIs). |
| `bun build` | Bundling for production, single-file executables, browser output. |

Bun as **runtime** vs Bun as **package-manager only** is the operating-model spine - see
`runtime-bun-vs-node-choose` and `references/ref-bun-package-manager-fallbacks.md`.

## Gotchas

- **Lockfile is text now**: Bun 1.2+ writes `bun.lock`; `bun.lockb` is legacy. Commit the
  lockfile; migrate binaries with `bun install --save-text-lockfile --frozen-lockfile
  --lockfile-only`.
- **Lifecycle scripts disabled by default**: `bun install` skips `postinstall` for
  security; add trusted packages to `trustedDependencies` in `package.json`.
- **`run` flags go before the script**: `bun --watch run dev` works; `bun run dev
  --watch` passes `--watch` to the script.
- **`require()` + top-level await**: a file using top-level `await` cannot be
  `require()`'d; use `import` / dynamic `import()`.
- **Env vars in bundles**: `bun build` does not inline `process.env` unless you pass
  `--env inline` (or `--env PUBLIC_*`).
- **`Bun.serve()` idle timeout**: closes idle connections after ~10s; set `idleTimeout`
  (or `server.timeout(req, 0)`) for SSE / long-lived streams.
- **Workspaces need names**: every workspace package must have a `name` in its
  `package.json`.
- **`--bun` to force Bun for Node-shebang bins**: `bun run --bun <bin>` (or
  `[run] bun = true` in `bunfig.toml`) - see `runtime-bun-run-bun-flag`.
- **`Bun.serve()` is unsupported on Vercel Functions** - see
  `vercel-bun-runtime-limitations`.

## Rules (opinionated operating model)

Open a rule for the "do this / not that" with exact commands. Full list:
`rules/_index.md`. Route by priority:

| Priority | Category | Prefix | Key rules |
| --- | --- | --- | --- |
| 1 | Package manager + lockfiles | `pm-` | `pm-no-mixed-lockfiles`, `pm-commit-bun-lockb`, `pm-bun-install-ci-frozen-lockfile`, `pm-linker-and-streaming-install`, `pm-bun-audit-security` |
| 1 | Runtime selection | `runtime-` | `runtime-bun-vs-node-choose`, `runtime-bun-run-bun-flag`, `runtime-bun-shell`, `runtime-env-files` |
| 1 | Vercel Bun runtime | `vercel-` | `vercel-bun-runtime-enable`, `vercel-bun-runtime-limitations`, `vercel-nextjs-bun-runtime-scripts` |
| 2 | Scripts + monorepos | `scripts-` | `scripts-bun-run-parallel-sequential`, `scripts-bun-filter-and-workspaces` |
| 2 | TypeScript + tooling | `tsconfig-`, `tooling-` | `tsconfig-bun-recommended`, `tsconfig-bun-types`, `tooling-bunfig` |
| 3 | Testing | `test-` | `test-bun-test-runner`, `test-bun-retry`, `test-mocking-and-spying` |
| 3 | Build + bundling | `build-` | `build-bun-build-bundler`, `build-compile-executables`, `build-bun-compile-browser` |
| 4 | Performance | `perf-` | `perf-prefer-bun-native-apis` |
| 5 | Migration + troubleshooting | `migrate-`, `troubleshooting-` | `migrate-node-to-bun-checklist`, `troubleshooting-esm-cjs-and-exports` |

## Power Tools (optional - requires `codex-dev`)

When the `codex-dev` binary is installed, the native engine audits, safe-fixes,
validates, and keeps references current. Skip this section entirely if you only need the
knowledge and rules above.

Quick start:

```bash
codex-dev --json bun audit --root .          # report Bun findings
codex-dev --json bun fixes plan --root .     # preview safe fixes (diffs + hashes)
codex-dev --json bun fixes apply --root .    # apply safe rewrites (+ rollback artifact)
codex-dev --json bun validate run --root . --fail-on warn
```

Full command surface:

- `codex-dev bun audit --root .`: report Bun findings.
- `codex-dev bun rules list`: print rule ids.
- `codex-dev bun rules show <rule-id>`: print one rule.
- `codex-dev bun fixes plan --root .`: safe fix candidates with hashes and diffs.
- `codex-dev bun fixes apply --root .`: apply safe rewrites; rollback artifact under
  external dev-skills state.
- `codex-dev bun validate plan --root .`: print validation commands.
- `codex-dev bun validate run --root . --fail-on warn`: audit then validate.
- `codex-dev bun benchmark --root .`: time audit and fix planning.
- `codex-dev bun references status`: inspect reference hashes and integrity.
- `codex-dev bun references plan`: fetch vendor docs and preview changed references.
- `codex-dev bun references sync`: refresh tracked references and rebuild indexes.
- `codex-dev bun doctor`: inspect paths, version pin, and integrity.
- `codex-dev tool import`: import an external JSON report into a task capsule.

Platform state: config `bun-platform.config.json` (keys: `disabledRules`,
`severityOverrides`, `adapters`, `includePaths`, `excludeDirs`, `baseline`, `maxFiles`,
`maxBytes`, `validationCommands`, `writeCache`); external config/state/cache under
`${XDG_*}/dev-skills/bun-platform`. Audit cache is read-only unless `--write-cache`; safe
fixes write rollback artifacts under external state, never in the repo. Example template:
`assets/templates/bun-platform.config.example.json`.

## References

Prefer rules for decisions; references for exact commands or API details. Start at
`references/index.md`.

| Topic | Reference |
| --- | --- |
| CLI + workflow cheatsheet | `references/ref-bun-cli-cheatsheet.md` |
| Built-in APIs cheatsheet | `references/ref-bun-builtins-cheatsheet.md` |
| Latest release notes | `references/ref-bun-release-notes-latest.md` |
| Capability map | `references/ref-bun-capabilities-latest.md` |
| Package-manager fallbacks | `references/ref-bun-package-manager-fallbacks.md` |
| Vercel Bun runtime | `references/ref-vercel-bun-runtime.md` |

**Freshness**: vendored references are snapshots for a pinned Bun version. For anything
newer or not covered here (e.g. `Bun.WebView`, `Bun.cron`, markdown entrypoints), consult
the live docs at <https://bun.com/docs/llms.txt> rather than assuming the snapshot is
current. With `codex-dev`, refresh snapshots via `codex-dev bun references plan` then
`codex-dev bun references sync`.

## Verification checklist

Before submitting Bun work:

- [ ] `bun install` resolves cleanly; `bun.lock` is committed (single lockfile).
- [ ] `bun test` passes (`bun test --coverage` if coverage is required).
- [ ] `bun run build` (or equivalent) succeeds.
- [ ] `bunfig.toml` / `tsconfig.json` match project needs (linker, `moduleResolution`).
- [ ] `@types/bun` installed for TypeScript; `bun run` (no args) lists scripts.
- [ ] `bun audit` reviewed for advisories.
- [ ] With `codex-dev`: `codex-dev --json bun audit --root .` is clean or triaged.
