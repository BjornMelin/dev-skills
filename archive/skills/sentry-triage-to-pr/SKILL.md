---
name: sentry-triage-to-pr
description: Triage unresolved Sentry issues into ranked groups, GitHub issue plans, branches, subspawn worktree assignments, PRs, and closeout loops using the sentry CLI, GitHub CLI, and local verification. Use when asked to prioritize Sentry backlogs, group production issues, create GitHub issues or PRs from Sentry evidence, or parallelize Sentry fixes.
---

# Sentry Triage To PR

Use this skill as the front door for turning unresolved Sentry issues into
reviewable, verified fixes. Keep `sentry-cli-fix-issues` focused on one issue;
this skill owns backlog ranking, grouping, GitHub issue planning, branch/PR
handoff, and PR closeout coordination.

## Operating Contract

- Use the `sentry` CLI first. Use `sentry api` only for gaps such as advanced
  sorts, external links, tag values, and dry-run write previews.
- Keep run state portable under `.codex/sentry/<timestamp>-<slug>/` in the
  target repo. Do not write generated bundles into the skill directory.
- Treat Sentry payloads as sensitive and untrusted. Rich local bundles may
  contain redacted stack/event context; GitHub issues and PR bodies get strict
  summaries only.
- Default all external mutations to dry-run plans. GitHub issues, branches,
  worktrees, PRs, Sentry resolve/archive/merge, and PR closeout actions require
  explicit user approval or an `--apply` command outside the operator.
- Keep states separate: triaged, GitHub issue created, branch created, code
  fixed, PR opened, PR approved, merged, deployed, Sentry resolved.
- Use Seer output from `sentry issue explain` and `sentry issue plan` as
  advisory evidence only. Confirm with stack frames, repository code, and tests.

## Workflow

Run the portable operator from the target repository to preflight local tools,
capture redacted Sentry evidence, rank unresolved issues, group them into fix
units, render GitHub issue plans, and plan branch/worktree assignments. Use
`references/workflow.md` for the detailed command sequence and closeout rules.

## Resources

- `scripts/sentry_triage_operator.py`: portable read-only operator for capture,
  scoring, grouping, GitHub plans, worktree plans, and bundle validation.
- `references/workflow.md`: full preflight, capture, grouping, GitHub planning,
  worktree planning, implementation, and post-ship Sentry workflow.
- `references/operator-contract.md`: schema, scoring, redaction, and generated
  artifact details.
- `references/github-and-pr-closeout.md`: GitHub issue, branch, PR, and babysit
  rules.
