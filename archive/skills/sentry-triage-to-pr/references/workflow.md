# Sentry Triage Workflow

Use this workflow from the target repository, not from the skill directory.
Keep generated state under `.codex/sentry/<timestamp>-<slug>/`.

## 1. Preflight

- Confirm `sentry --version`, `sentry auth whoami` when credentials are
  uncertain, `gh auth status` before GitHub writes, and `git status --short`.
- Resolve repo/project mapping. If multiple Sentry projects map to one repo,
  prefer an explicit `ORG/` or `ORG/PROJECT` target.
- Use the operator doctor when local tooling is uncertain:

  ```bash
  python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py doctor --json
  ```

## 2. Capture And Rank

Run the operator from the target repo:

```bash
python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py capture \
  --target ORG/ --query "is:unresolved" --period 7d --limit 100 \
  --hydrate-top 5 --include-seer --include-plan --out .codex/sentry/run.json

python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py triage \
  .codex/sentry/run.json --out .codex/sentry/triaged.json
```

For a known issue, use `--issue ISSUE`; this hydrates that issue directly
without running a broad issue list unless `--target` or `--query` is also
provided.

Score by impact plus fixability: Sentry priority, affected users, event count,
unhandled status, severity, recency/regression, project ownership confidence,
and `seerFixabilityScore` when present.

## 3. Group Into Fix Units

Create conservative evidence clusters:

```bash
python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py group \
  .codex/sentry/triaged.json --out .codex/sentry/groups.json
```

Group only issues that plausibly share a root cause and implementation surface.
Do not merge Sentry issues just because titles look similar. Block parallel
execution for groups that share lockfiles, migrations, build config, release
wiring, or unknown ownership.

## 4. Create GitHub Issue Plans

Render strict GitHub-safe issue bodies and idempotent commands:

```bash
python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py render-github \
  .codex/sentry/groups.json --repo OWNER/REPO --out-dir .codex/sentry/github
```

Search for existing GitHub issues before applying generated create/update
commands. Use the hidden marker in the body for dedupe.

## 5. Plan Branches And Subspawn Worktrees

Generate branch/worktree plans:

```bash
python3 skills/sentry-triage-to-pr/scripts/sentry_triage_operator.py plan-worktrees \
  .codex/sentry/groups.json --repo-root "$PWD" --out .codex/sentry/worktrees.json
```

Branches use `fix/sentry-IDS-slug`. Spawn at most 2-3 non-overlapping
implementation workers. Follow `$subspawn` strict rendezvous behavior: the
parent waits for the batch, reviews results, then synthesizes.

## 6. Implement And Ship

For each selected group, use `sentry-cli-fix-issues` for the per-issue
investigation/fix loop. Create a conventional branch, patch the repo, run
focused and repo-native checks, then use existing branch/PR workflows to commit,
push, and open a PR into `main`.

PR closeout requires no unresolved review comments, no failing required checks,
and an approving review when the repo requires review approval.

## 7. Post-Ship Sentry Handling

After merge/deploy evidence exists, generate Sentry commands such as:

```bash
sentry issue resolve ISSUE --in @commit --json
sentry issue archive ISSUE --until auto --json
```

Re-read each issue with `--fresh` before applying any state change.
