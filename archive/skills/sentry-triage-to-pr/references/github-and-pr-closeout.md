# GitHub And PR Closeout

## GitHub Issue Plans

Use one GitHub issue per conservative Sentry group. The issue body should
include:

- hidden marker: `<!-- sentry-triage-to-pr:v1 group=... issues=... -->`
- Sentry issue IDs and permalinks
- impact summary: users, events, priority, recency
- suspected root cause and affected implementation surface
- proposed fix plan and validation checklist
- residual risk and any `UNVERIFIED` claims

Before applying generated commands:

```bash
gh issue list --repo OWNER/REPO --search "sentry-triage-to-pr:v1 PROJ-123"
gh issue create --repo OWNER/REPO --title "..." --body-file issue.md --label sentry
```

Prefer update over duplicate creation when a marker already exists.

## Branches And Worktrees

Branch format:

```text
fix/sentry-PROJ-123-auth-timeout
fix/sentry-PROJ-123-PROJ-456-session-null
```

Parallel work is allowed only when groups have disjoint likely file ownership.
Do not parallelize groups that touch:

- lockfiles or package manager state
- database migrations or schema migrations
- release, deployment, source-map, or observability configuration
- shared auth/session/routing code unless the groups are explicitly split by
  owner and tests

## Subspawn Assignments

Each implementation worker prompt should include:

- Sentry group ID and issue IDs
- allowed repo/worktree path
- suspected files/symbols and forbidden shared surfaces
- exact verification commands to run
- required output: files changed, tests run, Sentry evidence used, residual
  risk, and PR/branch status

The parent session owns synthesis, final review, and PR closeout.

## PR Closeout

Open PRs as draft unless the repo’s normal process expects ready PRs. Closeout
means:

- implementation is committed on a semantically named branch;
- focused and repo-native checks pass or failures are classified with evidence;
- no unresolved hosted review comments remain;
- required GitHub checks are green;
- an approving review exists when the repo requires approval;
- Sentry status changes are deferred until merge/deploy evidence exists.

If CI or review comments fail, route through existing PR remediation workflows
instead of duplicating that logic in this skill.
