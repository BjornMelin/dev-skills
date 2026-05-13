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

1. **Preflight**
   - Confirm `sentry --version`, `sentry auth whoami` when credentials are
     uncertain, `gh auth status` before GitHub writes, and `git status --short`.
   - Resolve repo/project mapping. If multiple Sentry projects map to one repo,
     prefer an explicit `ORG/` or `ORG/PROJECT` target.
   - Use the operator doctor when local tooling is uncertain:

     ```bash
     python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py doctor --json
     ```

2. **Capture and rank**
   - Run the operator from the target repo:

     ```bash
     python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py capture \
       --target ORG/ --query "is:unresolved" --period 7d --limit 100 \
       --hydrate-top 5 --include-seer --include-plan --out .codex/sentry/run.json

     python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py triage \
       .codex/sentry/run.json --out .codex/sentry/triaged.json
     ```

   - For a known issue, use `--issue ISSUE`; this hydrates that issue directly
     without running a broad issue list unless `--target` or `--query` is also
     provided.

   - Score by impact plus fixability: Sentry priority, affected users, event
     count, unhandled status, severity, recency/regression, project ownership
     confidence, and `seerFixabilityScore` when present.

3. **Group into fix units**
   - Create conservative evidence clusters:

     ```bash
     python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py group \
       .codex/sentry/triaged.json --out .codex/sentry/groups.json
     ```

   - Group only issues that plausibly share a root cause and implementation
     surface. Do not merge Sentry issues just because titles look similar.
   - Block parallel execution for groups that share lockfiles, migrations,
     build config, release wiring, or unknown ownership.

4. **Create GitHub issue plans**
   - Render strict GitHub-safe issue bodies and idempotent commands:

     ```bash
     python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py render-github \
       .codex/sentry/groups.json --repo OWNER/REPO --out-dir .codex/sentry/github
     ```

   - Search for existing GitHub issues before applying generated create/update
     commands. Use the hidden marker in the body for dedupe.

5. **Plan branches and subspawn worktrees**
   - Generate branch/worktree plans:

     ```bash
     python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py plan-worktrees \
       .codex/sentry/groups.json --repo-root "$PWD" --out .codex/sentry/worktrees.json
     ```

   - Branches use `fix/sentry-IDS-slug`.
   - Spawn at most 2-3 non-overlapping implementation workers. Follow
     `$subspawn` strict rendezvous behavior: the parent waits for the batch,
     reviews results, then synthesizes.

6. **Implement and ship**
   - For each selected group, use `sentry-cli-fix-issues` for the per-issue
     investigation/fix loop.
   - Create a conventional branch, patch the repo, run focused and repo-native
     checks, then use existing branch/PR workflows to commit, push, and open a
     PR into `main`.
   - PR closeout requires no unresolved review comments, no failing required
     checks, and an approving review when the repo requires review approval.

7. **Post-ship Sentry handling**
   - After merge/deploy evidence exists, generate Sentry commands such as:

     ```bash
     sentry issue resolve ISSUE --in @commit --json
     sentry issue archive ISSUE --until auto --json
     ```

   - Re-read each issue with `--fresh` before applying any state change.

## Resources

- `scripts/sentry_triage_operator.py`: portable read-only operator for capture,
  scoring, grouping, GitHub plans, worktree plans, and bundle validation.
- `references/operator-contract.md`: schema, scoring, redaction, and generated
  artifact details.
- `references/github-and-pr-closeout.md`: GitHub issue, branch, PR, and babysit
  rules.
