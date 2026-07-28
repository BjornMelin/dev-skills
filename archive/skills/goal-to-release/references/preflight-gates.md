# Preflight Gates

Use this when a lane is entering `preflight`, before PR creation, or when a PR
has repeated CI/reviewer churn.

## Surface Discovery

Inspect before choosing gates:
- repo instructions: `AGENTS.md`, `CLAUDE.md`, README, docs
- package managers: `package.json`, `pnpm-lock.yaml`, `Cargo.toml`,
  `pyproject.toml`, lockfiles
- CI: `.github/workflows`, Vercel/Netlify/Cloudflare config
- generated outputs: schemas, SDKs, migrations, snapshots, codegen manifests
- deploy/runtime surfaces: web apps, services, workers, CLIs, containers
- review history: open PR comments, recent stale findings, required checks

Prefer repo-native scripts over invented commands. If a repo has no canonical
gate, choose the smallest relevant check and record the gap.
`scripts/goal_preflight.py --run` is intentionally light and only executes
commands marked `run_by_default`; use `--run-all` only when broad local
validation is explicitly intended.

For repo-wide modernization or dependency work, route the lane through the
dependency/package-manager skill before changing versions. Capture upstream
release notes, changelog/API-breaking changes, lockfile policy, generated
artifact expectations, and rollback/deploy risk in the issue or ledger.

## Universal Checks

Always consider:
- `git status --short`
- `git diff --check`
- repo lint/typecheck/test/build scripts
- generated artifact drift checks when generated files changed
- docs alignment when user-facing, API, schema, CLI, deploy, or runtime
  behavior changed
- secret redaction when logs, artifacts, telemetry, screenshots, or PR text are
  produced

Dirty worktree:
- list user-owned dirty files before edits
- do not stage unrelated paths
- if required edits overlap dirty user changes, inspect exact hunks and either
  adapt narrowly, create an isolated worktree/branch, or stop when ownership is
  ambiguous
- never stash or discard user changes without explicit approval

## Conditional Gates

JavaScript/TypeScript:
- package-manager install policy
- lint/format check
- typecheck
- unit/integration tests
- build for touched apps/packages
- no unhandled promises in async/runtime code
- exported API docs when repo/reviewer rules require them

Rust:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo test --workspace --all-targets --all-features --locked`
- generated contract parity when Rust consumes schemas

Python:
- repo-native formatter/linter/typecheck/test
- `uv` or project-selected environment policy

Web/UI:
- component tests
- build
- browser smoke with screenshots for user-facing flows
- accessibility, metadata, responsive layout, and console-error checks

Deploy/platform:
- preview deployment status
- env var requirements
- migration/deploy runbook proof
- rollback or residual-risk notes

Security:
- secret scanning
- authz/authn checks for changed routes
- no raw secrets in logs or artifacts
- dependency/supply-chain checks when lockfiles or CI changed

Dependency modernization:
- outdated/dependency graph inventory
- release notes and migration guides for upgraded packages
- native package capabilities that can replace custom code
- compatibility checks for peer dependencies, engines, bundlers, and deploy
  platforms
- focused regression tests for changed APIs plus repo aggregate gates before PR

## PR Readiness

A nontrivial implementation PR is not ready until:
- focused tests pass
- repo-native gates for touched surfaces pass
- generated artifacts are in sync
- docs/deploy impact is handled or explicitly not required
- bounded subagent review has no unresolved blocking findings
- PR body has issue link, validation, docs/deploy notes, and residual risks
