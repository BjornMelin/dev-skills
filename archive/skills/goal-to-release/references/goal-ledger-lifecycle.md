# Goal Ledger Lifecycle

Use this when creating durable ledgers, writing evidence, handing off, auditing,
or distilling completed goals.

## Storage Defaults

Prefer repo-local untracked storage:

```text
.agents/goals/<YYYY-MM-DD>-<slug>-<id>/
  ledger.md
  summary.md
  summary.json
  evidence/
  preflight/
  archive/
```

Before writing under `.agents/goals`, verify it is ignored by git. If it would
be tracked, do not modify `.gitignore`; fall back to:

```text
~/.codex/goals/<repo-slug>/<YYYY-MM-DD>-<slug>-<id>/
```

Use one directory per goal. Never mix unrelated goals in one ledger.

Create a durable ledger by default for multi-issue work, multi-PR work, stacked
or parallel branches, repo-wide modernization, dependency upgrade campaigns,
release orchestration, cross-session handoff, or any goal whose evidence should
be reusable later. Use working notes only for small single-PR tasks that will
finish in one session and do not need durable retrieval.

## Evidence Policy

Store redacted, concise evidence:
- command names and outcomes
- hosted check names and URLs
- review-thread counts, review decisions, and approval state
- screenshot paths or hashes
- deploy URLs or identifiers when safe
- explicit "not applicable" deploy rationale when no deploy evidence is needed
- short log excerpts after redaction

Do not store raw secrets, full noisy logs, private tokens, or unnecessary large
artifacts.

## Handoff

If work pauses, write or update handoff content with:
- current state
- merged and open PRs
- linked issues
- validation and deploy evidence
- next exact action
- blockers and what not to redo

## Closeout Distillation

When a durable goal is achieved:
1. Run the strict ledger audit.
2. Distill the workbench into `summary.md` and `summary.json`.
3. Keep raw ledger/evidence available through archive pointers.
4. Treat the summary files as the first future retrieval surface.

The summary must include:
- final status and objective
- issues, branches, PRs, and merge outcomes
- major decisions
- validation and deploy evidence
- docs and release impact
- residual risks and follow-ups
- reusable workflow lessons
