# Operator Contract

`scripts/sentry_triage_operator.py` is a portable, read-only helper. It shells
out to `sentry` only for capture. All write operations are rendered as plans.

## Subcommands

- `capture`: runs `sentry issue list` and optionally hydrates top issues with
  `issue view`, bounded `issue events --full`, `issue explain`, and
  `issue plan`. Detected traces and replays are captured only when IDs appear.
- `doctor`: checks local `sentry`, `gh`, `git`, and `codex` CLI availability.
  It can emit JSON and optionally run auth probes.
- `triage`: scores issues deterministically and adds ranked score components.
- `group`: creates conservative implementation groups from project, normalized
  title, culprit, and suspected ownership signals.
- `render-github`: renders GitHub-safe issue bodies plus `gh issue create`
  command plans. Bodies include a hidden dedupe marker.
- `plan-worktrees`: renders branch, worktree, and subspawn prompt plans. It does
  not create branches or worktrees.
- `validate-bundle`: checks schema shape and obvious sensitive values.

## Schemas

The canonical schema is `sentry-triage-to-pr.bundle.v1`. Bundles may contain:

- `issues`: normalized issue list rows from `sentry issue list`.
- `issue_contexts`: rich redacted per-issue sections from native Sentry
  commands.
- `ranked_issues`: deterministic ranking output.
- `groups`: PR-sized issue groups.
- `github_plan`: generated issue body paths and `gh` command plans.
- `worktree_plan`: branch, worktree, and subspawn assignment plans.
- `commands`: redacted command metadata with return codes.

## Scoring

Default score is impact plus fixability:

- impact: affected users, event count, and recent activity.
- urgency: Sentry priority, level, unhandled status, regression/review substatus.
- fixability: Seer fixability score, culprit/code-location confidence, and
  localized project ownership.

The score is explainable. Each issue includes component values and the fields
that drove them. LLM judgment can reorder only after explaining the override.

## Redaction

The operator uses two-tier output:

- local bundle: rich but redacted and truncated, suitable for agent reasoning.
- GitHub body: strict summary with Sentry links, issue IDs, impact, suspected
  root cause, proposed validation, and no raw event payloads.

Sensitive keys and patterns are redacted recursively. GitHub bodies must not
contain user objects, request bodies, headers, cookies, tokens, emails, IP
addresses, prompts, completions, or replay payloads.

## Native Sentry Capture

Use these command families before raw API calls:

```bash
sentry issue list TARGET --query "is:unresolved" --period 7d --limit 100 --json
sentry issue view ISSUE --spans 3 --json
sentry issue events ISSUE --full --period 7d --limit 5 --json
sentry issue explain ISSUE --json
sentry issue plan ISSUE --json
sentry trace logs TRACE_ID --json
sentry replay view REPLAY_ID --json
```

`capture --issue ISSUE` hydrates exact issues and does not run
`sentry issue list` unless a list target or query is provided. This keeps
single-issue runs narrow and avoids accidental broad Sentry reads.

Use `sentry api` only when the native commands do not expose the required data,
for example advanced issue sorts, external issue links, or tag value summaries.

## Stop Rules

Mark a group `needs_manual_triage` when:

- group members do not share project, culprit, ownership, or normalized title;
- events point at shared config, lockfiles, migrations, generated code, or
  release wiring;
- Sentry data is missing, stale, or too redacted to justify the group;
- Seer output conflicts with stack frames or repository code.
