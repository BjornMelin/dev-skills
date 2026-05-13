---
name: gh-deps-intel
description: JS/TS + Python dep upgrade intel (monorepos). Outdated package-manager checks + GitHub release/changelog → Markdown+JSON. Audits, compatible bumps, deprecations, breaking changes, full upgrades.
---

# Gh Deps Intel

Use this as the canonical dependency-upgrade workflow skill. The shared `deps-workbench` helpers provide lightweight preflight inventory, classification, and usage mapping; this skill remains the deeper GitHub-aware analysis and reporting engine. This skill absorbs the former `dep-upgrade-spec` role.

## Invocation Defaults

- Explicit invocation with upgrade language such as "fully upgrade", "apply the upgrade", or a specific target package defaults to `execute`.
- Explicit invocation without an implementation request defaults to `plan`.
- In `execute`, still stop if auth, package-manager access, or repo safety checks fail.

## Tool Routing (MUST FOLLOW)

1. If you need a cheap preflight, run the shared helpers first:
   - `/home/bjorn/.codex/skill-support/bin/deps-workbench inventory --cwd <repo> --out <json>`
   - `/home/bjorn/.codex/skill-support/bin/deps-workbench classify --input <json> --out <json>`
   - `/home/bjorn/.codex/skill-support/bin/deps-workbench usage-scan --cwd <repo> --packages <pkg...> --out <json>`
2. If the user explicitly requests `gh api`/CLI only, use `scripts/gh_release_diff.py`, `scripts/gh_compare_notes.py`, or `scripts/gh_rate_limit_diag.py` directly.
3. For single-package requests ("fully upgrade X package"), use `scripts/gh_deps_intel.py package --dependency <name>`.
4. Otherwise, use `scripts/gh_deps_intel.py full` as the single orchestrator path (REST-first with GraphQL fallback for release retrieval).
5. Default to `--mode safe`; use `--mode fast` only when the user opts in.

## Workflow Modes

- `plan`: Markdown + JSON report + order; no dep mutations.
- `execute`: run batches, apply upgrades, repo-native verification, refresh report.

## Quick Start

1. Prerequisites:

- `gh` logged in (`gh auth status`)
- `python3`
- Optional: `bun`/`pnpm`/`npm`/`yarn`, `uv`

1. From repo root:

- `python /home/bjorn/.agents/skills/gh-deps-intel/scripts/gh_deps_intel.py full --repo . --out reports --mode safe`

1. Outputs:

- `reports/dependency-upgrade-report.md`
- `reports/dependency-upgrade-report.json`

## Execution Modes

- `safe` (default): serial GitHub API pacing + backoff/retries.
- `fast`: bounded parallel enrich; auto fallback to safe on rate limit/errors.

## Primary Commands

- Full pipeline (default):
  - `python scripts/gh_deps_intel.py full --repo . --out reports --mode safe`
- Single dependency full spec:
  - `python scripts/gh_deps_intel.py package --repo . --out reports --mode safe --dependency workflow`
  - `python scripts/upgrade_one_dep.py workflow --repo . --out reports --mode safe`
- Stage outputs:
  - `python scripts/gh_deps_intel.py scan --repo . --out reports`
  - `python scripts/gh_deps_intel.py enrich --repo . --out reports --mode safe`
  - `python scripts/gh_deps_intel.py analyze --repo . --out reports --mode safe`
- Rate-limit diagnostic:
  - `python scripts/gh_deps_intel.py rate-limit`

## Resource Map

| Path | Type | Purpose | Use When |
| --- | --- | --- | --- |
| `scripts/gh_deps_intel.py` | orchestrator | Scan/enrich/analyze/report end-to-end | Any dep intel request |
| `scripts/upgrade_one_dep.py` | utility | Single-dep `package` wrapper | One-package refactor spec |
| `scripts/detect_repo.py` | script | Runtime, package managers, workspace | Repo/runtime context only |
| `scripts/collect_deps.py` | script | Deps from `package.json` + `pyproject.toml` | Manifest inventory |
| `scripts/outdated_probe.py` | script | Outdated cmds per manager | current/wanted/latest |
| `scripts/repo_resolver.py` | script | Registry meta → GitHub repos | package → repo map |
| `scripts/gh_release_fetch.py` | script | Releases/tags/changelog, cache+retries | Release/changelog text |
| `scripts/runtime_policy.py` | script | Node/Python compatibility targets | Pick compatible version |
| `scripts/impact_analyzer.py` | script | Breaking/deprecations/features, actions | Migration deltas |
| `scripts/render_report.py` | script | Markdown + JSON reports | Final outputs |
| `scripts/gh_release_diff.py` | utility | Release window for `owner/repo` | Ad-hoc release research |
| `scripts/gh_compare_notes.py` | utility | Compare two refs/tags | No changelog; need commits |
| `scripts/gh_rate_limit_diag.py` | utility | Current rate limit | Check API budget first |
| `references/workflow.md` | reference | Run order + decision tree | Process |
| `references/command-mapping.md` | reference | PM command equivalents | bun/pnpm/npm/yarn/uv/pip map |
| `references/github-api-endpoints.md` | reference | Endpoints + backoff | API details |
| `references/compatibility-policy.md` | reference | Runtime-pinned rules | Version policy |
| `references/report-spec.md` | reference | Schema + sections | Machine+human contract |
| `references/troubleshooting.md` | reference | Failures + mitigations | Auth/rate/parse issues |

## Reporting Contract

Always produce both files in the same stable contract:

- Markdown: concise upgrade/refactor plan by dependency.
- JSON: full structured data for downstream automation.

For `package` mode, include:

- targeted dependency selectors
- repository impact map (usage hits + affected files)
- explicit refactor checklist for that package only

## Additional GH Workflows

- Release window diff:
  - `python scripts/gh_release_diff.py owner/repo --current 1.2.3 --target 2.0.0`
- Compare summary:
  - `python scripts/gh_compare_notes.py owner/repo v1.2.3 v2.0.0`
- Rate-limit budget:
  - `python scripts/gh_rate_limit_diag.py`

## Shared Support Alignment

- Preflight + classify: `/home/bjorn/.codex/skill-support/bin/deps-workbench`.
- Long GitHub release intel: here.
- Both used: Markdown + JSON from this skill = final contract.
