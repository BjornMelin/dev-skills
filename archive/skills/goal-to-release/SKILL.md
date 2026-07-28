---
name: goal-to-release
description: Orchestrate issue-backed goals from ambiguous request to shipped release with ledgers, adaptive PR graphs, preflight gates, review loops, deploy evidence, closeout audits, and retrospectives. Use for multi-issue/product/platform/maintenance goals, GitHub issue/PR planning, stacked or parallel branches, release babysitting, or goal handoff. Skip isolated fixes unless they need release planning.
---

# Goal to Release

Turn broad goals into issue-backed, reviewable, verified releases. Own
sequencing, evidence, review hygiene, and closeout. Route concrete domain work
to narrower skills instead of duplicating their mechanics.

## Operating Model

Use live evidence first: repo instructions, worktree, branch, open issues, PRs,
review threads, CI, deploy state, docs, scripts, and recent reviewer feedback.
Treat memory and prompt history as leads to verify when facts can drift.

Keep a working goal ledger. Use working notes for short, single-PR work. Use a
durable ledger by default when the goal spans multiple issues/PRs, stacked or
parallel branches, repo-wide modernization, dependency upgrades, release
orchestration, cross-session execution, or any user-requested handoff. Then use
`scripts/new_goal_ledger.py` to create an isolated untracked goal directory.
Read [goal-ledger-lifecycle.md](references/goal-ledger-lifecycle.md) before
creating, auditing, distilling, or handing off durable ledgers.

If live inspection is forbidden, produce a provisional ledger and mark
unverified live facts as `TBD` or `UNVERIFIED`.

Modes:
- `plan-only`: issue/PR/release plan only.
- `preflight`: discover repo surfaces, gates, risks, and PR-readiness before
  implementation or publication.
- `implement`: build one or more lanes.
- `monitor`: babysit hosted CI/reviews/deploys.
- `closeout`: prove issues, PRs, docs, deploys, evidence, and summaries are
  complete.
- `retrospective`: extract reusable workflow lessons.

## Core Workflow

1. **Ground**
   - Read repo instructions and relevant docs/code first.
   - Inspect worktree and branch state; preserve unrelated dirty work.
   - If user edits are unrelated, keep them out of staging and commits. If they
     overlap files you must touch, inspect the exact diff and adapt around it;
     use a separate branch/worktree when that is the cleanest safe path. Never
     stash, discard, reset, or rewrite user work without explicit approval.
   - Identify existing issues, branches, PRs, reviews, checks, deployments, and
     goal ledgers before creating new ones.

2. **Charter**
   - State objective, non-goals, success criteria, release/deploy boundary, and
     user steering.
   - If the goal is larger than one reviewable PR, create or update one issue
     per coherent lane.
   - Reorder lanes when evidence shows a force multiplier; record the rationale
     in the ledger and issue/PR text.

3. **Preflight**
   - For nontrivial lanes, run or reason through `scripts/goal_preflight.py`.
   - Build a repo-adapted gate plan from discovered surfaces, not a generic
     checklist.
   - Read [preflight-gates.md](references/preflight-gates.md) for conditional
     gates and evidence expectations.

4. **Plan the PR graph**
   - Default to serial PRs for coupled work.
   - Use parallel or stacked PRs only when lanes are independent or explicitly
     ordered.
   - Every stack records base branch, dependency, retarget plan, validation
     scope, and merge order.
   - For stacked work, open each PR against its immediate base branch. Merge the
     root PR first, then retarget or rebase the next PR onto the new target
     branch, refresh hosted checks/reviews, and only then merge the next stack
     layer.

5. **Research and decide**
   - Use current official docs, source, package repos, or deployed behavior for
     dependency, platform, API, deploy, security, and architecture choices.
   - Score meaningful choices when the decision is not obvious.
   - Encode durable decisions in issues, PR bodies, ADRs, specs, or docs.

6. **Implement lanes**
   - Keep the active branch narrow and semantic.
   - Hard-cut obsolete paths when the goal requires entropy reduction.
   - Update tests, docs, fixtures, generated metadata, and runbooks that are
     part of the changed contract.
   - Do not absorb adjacent redesigns silently; split them into follow-up
     issues or lanes.

7. **Verify before PR publication**
   - Run focused checks while iterating.
   - Before opening a nontrivial PR, run repo-native gates required by touched
     surfaces and mandatory bounded subagent review.
   - Record exact local command outcomes separately from hosted CI and deploy
     proof.

8. **Ship and monitor**
   - Commit by semantic intent; do not stage unrelated files.
   - PR bodies include linked issue, summary, validation, docs impact,
     deploy/provider notes, screenshots when relevant, and residual risks.
   - Read [review-and-merge-loop.md](references/review-and-merge-loop.md) when
     opening, updating, or merging PRs.
   - Refetch live hosted review threads after every push. Fix valid comments,
     rebut stale or suboptimal comments with evidence, resolve addressed
     threads, and wait for fresh checks/reviews.

9. **Close out**
   - Verify issues, PRs, branches, docs, deploys, review decisions, unresolved
     review threads, and follow-ups from live sources.
   - If deploy evidence is not applicable, record the exact reason instead of
     leaving deploy state blank.
   - Run `scripts/audit_goal_ledger.py --strict` for durable ledgers.
   - When a durable goal is achieved, run `scripts/distill_goal_summary.py` so
     future agents can load `summary.md` or `summary.json` before raw ledgers.

10. **Retrospect**
    - Summarize shipped changes, decisions, user steering, validation, deploy
      evidence, and reusable workflow lessons.
    - Generalize only repeated cross-repo patterns into skills. Repo-specific
      lessons belong in repo docs, issues, PRs, or AGENTS guidance.

## Skill Routing

Use narrower skills just in time:
- `$dependency-upgrade` plus package-manager/Turborepo skills for dependency
  modernization, release notes, lockfiles, and package graph risk.
- `$subspawn` for bounded independent sidecars and pre-PR expert review.
- `$commit` for semantic staging and commits.
- `$babysit-pr` for hosted PR monitoring.
- `$repo-docs-align` for source-of-truth docs reconciliation.
- `$hard-cut`, `$reducing-entropy`, `$clean-code`, security, web, Rust, Vercel,
  or repo-specific skills when the active lane touches those domains.

Do not load or restate those skills before their lane needs them.

## Stop Conditions

Stop and ask only when proceeding risks data loss, secrets exposure, production
impact without approval, destructive git history, or an ambiguous public/API
contract break. Otherwise make a reasonable decision, document evidence, and
continue.

## Failure Modes To Guard

- **Review-reactive churn**: run preflight and subagent review before PRs.
- **Scope ballooning**: split adjacent redesigns into follow-up lanes.
- **Stale hosted state**: refetch reviews/checks after every push.
- **False closeout**: audit the ledger and live sources before claiming done.
- **Ledger archaeology**: distill achieved goals into `summary.md` and
  `summary.json`.
- **Over-delegation**: keep critical path and final synthesis local.
- **Plan fossilization**: update issues, PRs, and docs when reality changes.
