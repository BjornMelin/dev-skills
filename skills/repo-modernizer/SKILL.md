---
name: repo-modernizer
description: "Repo/monorepo modernization: dependency upgrades, security fixes, deprecation cleanup, framework migrations, dependency-native refactors, and verified hard-cut simplification."
---

# Repo Modernizer

Use this skill when you need to audit, upgrade, refactor, and simplify a repo
or monorepo end-to-end around dependencies, security findings, deprecations,
and framework migrations.

## Variant References

Use the base prompt below by default. If the repo shape or workflow calls for a
specialized operating mode, load the matching reference first:

- `references/00-variant-index.md`
- `references/framework-lane-selection.md`
- `references/dependency-decision-matrix.md`
- `references/verification-matrix.md`
- `references/report-template.md`
- `references/bun-first.md`
- `references/pnpm-turbo-monorepo.md`
- `references/plan-first.md`
- `references/gh-deps-workflow.md`
- `references/gh-deps-report-spec.md`

Selection guidance:

- Bun-managed repos: start with `references/bun-first.md`
- pnpm + Turbo monorepos: start with `references/pnpm-turbo-monorepo.md`
- review-heavy or approval-gated work: start with `references/plan-first.md`
- GitHub release/changelog dependency intel: use the scripts in `scripts/` and
  load `references/gh-deps-workflow.md`
- framework-heavy repos: also read `references/framework-lane-selection.md`
- during package triage: read `references/dependency-decision-matrix.md`
- before execution wrap-up: read `references/verification-matrix.md`
- for final delivery: read `references/report-template.md`
- if unclear: read `references/00-variant-index.md` first

## Intent

- Find every vulnerable dependency, outdated dependency, deprecated API,
  obsolete custom abstraction, and dependency-driven cleanup opportunity.
- Deeply research the latest official docs, changelogs, migration guides,
  release notes, GitHub releases, advisories, and upstream issues.
- Upgrade dependencies to the latest safe and supportable versions.
- Refactor the codebase to adopt modern dependency-native APIs and delete
  obsolete wrappers, shims, fallbacks, and duplicate implementations.
- Leave the repo in a verified, production-ready, simplified state.

## Required Skills and Tools

Use these throughout the task:

- `$bun-dev`
- `$hard-cut`
- `$clean-code`
- `$reducing-entropy`
- `$github`
- `context7`
- `web.run`

Use `$opensrc` conditionally as an escalation path for package upgrades
that need source-level proof.

Framework-specific routing is defined in the execution prompt below and refined
further in `references/framework-lane-selection.md`.

## GitHub Dependency Intel Lane

Resolve `skill_dir` as the directory containing this skill before running the
bundled dependency-intel scripts.

- Full dependency report:
  - `python3 "$skill_dir/scripts/gh_deps_intel.py" full --repo . --out reports --mode safe`
- Single dependency migration plan:
  - `python3 "$skill_dir/scripts/gh_deps_intel.py" package --repo . --out reports --mode safe --dependency <name>`
- Stage-level debugging:
  - `python3 "$skill_dir/scripts/gh_deps_intel.py" scan --repo . --out reports`
  - `python3 "$skill_dir/scripts/gh_deps_intel.py" enrich --repo . --out reports --mode safe`
  - `python3 "$skill_dir/scripts/gh_deps_intel.py" analyze --repo . --out reports --mode safe`
- Direct GitHub helpers:
  - `python3 "$skill_dir/scripts/gh_release_diff.py" owner/repo --current <version> --target <version>`
  - `python3 "$skill_dir/scripts/gh_compare_notes.py" owner/repo <old-ref> <new-ref>`
  - `python3 "$skill_dir/scripts/gh_rate_limit_diag.py"`

Default to `--mode safe`. Use `--mode fast` only when the user accepts bounded
parallel API calls. Produce both Markdown and JSON reports.

Load these references when using this lane:

- `references/gh-deps-workflow.md`
- `references/gh-deps-command-mapping.md`
- `references/gh-deps-github-api-endpoints.md`
- `references/gh-deps-compatibility-policy.md`
- `references/gh-deps-report-spec.md`
- `references/gh-deps-troubleshooting.md`

## Execution Prompt

For full execution mode, load [references/execution-prompt.md](references/execution-prompt.md). It contains the complete modernization prompt, research requirements, dependency audit workflow, framework lanes, upgrade strategy, verification expectations, and final report contract.

Keep this entrypoint focused on routing and reference selection. Do not duplicate framework-specific playbooks here; use the variant references above.
