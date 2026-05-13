---
name: repo-modernize-upgrade-audit
description: Repo/monorepo—dep modernize, vuln fix, framework-aware upgrade, hard-cut, dep-native refactors.
---

# Repo Modernize Upgrade Audit

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

Selection guidance:

- Bun-managed repos: start with `references/bun-first.md`
- pnpm + Turbo monorepos: start with `references/pnpm-turbo-monorepo.md`
- review-heavy or approval-gated work: start with `references/plan-first.md`
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

## Execution Prompt

For full execution mode, load [references/execution-prompt.md](references/execution-prompt.md). It contains the complete modernization prompt, research requirements, dependency audit workflow, framework lanes, upgrade strategy, verification expectations, and final report contract.

Keep this entrypoint focused on routing and reference selection. Do not duplicate framework-specific playbooks here; use the variant references above.
