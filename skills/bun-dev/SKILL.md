---
name: bun-dev
description: "Definitive, rule-first Bun development/build/runtime guidance plus native dev-skills automation. Use when adopting Bun, migrating from Node.js, auditing or fixing Bun package-management posture, optimizing scripts/monorepos, configuring Bun + TypeScript, using bun test/build, or deploying Bun workloads/Vercel Functions with Bun runtime."
---

# Bun Dev

Rule-first Bun guidance plus the native `codex-dev bun` command surface.

Use `bun-dev` for operating-model decisions, rule lookup, reference sync, and
platform routing. Use `bun-audit` when the task is specifically a scan,
safe-fix, or validation workflow.

## Navigation

1. If changing package manager/runtime, open P1 rules first:
   `pm-*`, `runtime-*`, `vercel-*`.
2. If working in a monorepo, open:
   `scripts-bun-run-parallel-sequential`, `scripts-bun-filter-and-workspaces`.
3. If deploying to Vercel, open:
   `vercel-bun-runtime-enable`, `vercel-bun-runtime-limitations`.
4. For repo-wide enforcement, use:
   `codex-dev bun audit`, `codex-dev bun fixes plan`,
   `codex-dev bun fixes apply`, and `codex-dev bun validate run`.
5. For current vendor-backed references, use:
   `codex-dev bun references status` and `codex-dev bun references plan`.

## Quick Start

Audit:

```bash
codex-dev --json bun audit --root .
```

Plan safe fixes:

```bash
codex-dev --json bun fixes plan --root .
```

Apply safe fixes:

```bash
codex-dev --json bun fixes apply --root .
```

Validate after remediation:

```bash
codex-dev --json bun validate run --root . --fail-on warn
```

## Priority Table

| Priority | Category | Impact | Prefix |
| --- | --- | --- | --- |
| 1 | Package manager + lockfiles | Critical | `pm-` |
| 1 | Runtime selection + one runtime where possible | Critical | `runtime-` |
| 1 | Vercel Bun runtime | Critical | `vercel-` |
| 2 | Scripts + monorepo orchestration | High | `scripts-` |
| 2 | TypeScript + tooling | High | `tsconfig-` |
| 3 | Testing | Medium | `test-` |
| 3 | Bundling + build | Medium | `build-` |
| 4 | Performance | Medium | `perf-` |
| 5 | Migration + troubleshooting | Low/medium | `migrate-`, `troubleshooting-` |

## Quick Reference

Package manager + lockfiles:

- `pm-bun-add-remove-update`
- `pm-no-mixed-lockfiles`
- `pm-commit-bun-lockb`
- `pm-bun-install-ci-frozen-lockfile`
- `pm-package-manager-field`
- `pm-bunx-vs-npx`

Runtime selection:

- `runtime-bun-vs-node-choose`
- `runtime-bun-run-bun-flag`
- `runtime-ts-direct-execution`
- `runtime-watch-and-hot-reload`
- `runtime-env-files`

Vercel Bun runtime:

- `vercel-bun-install-detection`
- `vercel-bun-runtime-enable`
- `vercel-bun-runtime-limitations`
- `vercel-nextjs-bun-runtime-scripts`
- `vercel-bun-function-fetch-handler`

Scripts + monorepos:

- `scripts-bun-run-parallel-sequential`
- `scripts-bun-filter-and-workspaces`
- `scripts-no-npm-in-bun-repos`

TypeScript:

- `tsconfig-bun-recommended`
- `tsconfig-bun-types`
- `tsconfig-module-resolution-bundler`

Testing + build:

- `test-bun-test-runner`
- `test-bun-retry`
- `test-mocking-and-spying`
- `build-bun-build-bundler`
- `build-compile-executables`
- `build-bun-compile-browser`

Performance:

- `perf-prefer-bun-native-apis`
- `perf-avoid-node-fs-promises-hot-paths`

Migration + troubleshooting:

- `migrate-node-to-bun-checklist`
- `troubleshooting-esm-cjs-and-exports`
- `troubleshooting-types-bun`

## Native Commands

- `codex-dev bun audit --root .`: report Bun findings.
- `codex-dev bun rules list`: print rule ids.
- `codex-dev bun rules show <rule-id>`: print one rule.
- `codex-dev bun fixes plan --root .`: print safe fix candidates with hashes and diffs.
- `codex-dev bun fixes apply --root .`: apply safe rewrites and write rollback artifacts under external dev-skills state.
- `codex-dev bun validate plan --root .`: print validation commands.
- `codex-dev bun validate run --root . --fail-on warn`: audit then run validation commands.
- `codex-dev bun benchmark --root .`: time audit and fix planning.
- `codex-dev bun references status`: inspect reference hashes and integrity.
- `codex-dev bun references plan`: fetch vendor docs and preview changed reference files.
- `codex-dev bun references sync`: refresh tracked references and rebuild indexes.
- `codex-dev bun doctor`: inspect paths, version pin, and integrity.
- `codex-dev tool import`: import an external JSON report into a task capsule.

## Platform State

- Config file: `bun-platform.config.json`
- External config: `${XDG_CONFIG_HOME:-~/.config}/dev-skills/bun-platform`
- External state: `${XDG_STATE_HOME:-~/.local/state}/dev-skills/bun-platform`
- External cache: `${XDG_CACHE_HOME:-~/.cache}/dev-skills/bun-platform`
- Audit cache is read-only by default; opt in with `--write-cache`.
- Safe fixes write rollback artifacts under external state, never under the repo root.

Config keys:

- `disabledRules`
- `severityOverrides`
- `adapters`
- `includePaths`
- `excludeDirs`
- `baseline`
- `maxFiles`
- `maxBytes`
- `validationCommands`
- `writeCache`

Example template: `assets/templates/bun-platform.config.example.json`.

## References

Start with `references/index.md`.

| Topic | Reference | Start with rules |
| --- | --- | --- |
| Bun latest release notes | `references/ref-bun-release-notes-latest.md` | `scripts-bun-run-parallel-sequential`, `build-bun-compile-browser`, `test-bun-retry`, `test-bun-test-runner` |
| Bun capabilities | `references/ref-bun-capabilities-latest.md` | `runtime-*`, `build-*`, `test-*` |
| Bun package-manager fallbacks | `references/ref-bun-package-manager-fallbacks.md` | `pm-*`, `scripts-*` |
| Vercel Bun runtime | `references/ref-vercel-bun-runtime.md` | `vercel-bun-runtime-enable`, `vercel-bun-runtime-limitations` |

Refresh reference snapshots:

```bash
codex-dev --json bun references plan
codex-dev --json bun references sync
```

Skill integrity check:

```bash
codex-dev --json bun doctor
codex-dev --json bun references status
```
