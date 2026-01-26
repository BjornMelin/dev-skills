# ExecPlans (durable planning across sessions and compaction)

An **ExecPlan** is a versioned, living design+execution document that makes long work resumable even when:

- your conversation gets compacted,
- sessions restart,
- multiple agents are involved,
- tool traces are lost.

In practice: treat the ExecPlan as your **durable “state machine on disk”**.

## Core contract

An ExecPlan must be:

- **Self-contained**: a novice with only a fresh checkout + the plan can execute it.
- **Outcome-first**: acceptance is observable behavior, not internal attributes.
- **Living**: progress, surprises, and decisions stay aligned with reality.
- **Safe to re-run**: include recovery steps and idempotent commands.

The canonical standard lives in `.agent/PLANS.md` (inside the repo you’re working in).

## Where plans live

- `execplans/execplan-<short-name>.md`

Treat `execplans/` as a stable namespace: links, CI references, and long-running work should rely on it.

## What to record so you can resume

At minimum, record these “resume pointers” in **Context and Orientation** (and keep them current):

- repo root path (or “run from repo root”)
- branch + current commit hash
- the exact commands already run and their outcomes
- paths to artifacts:
  - JSONL logs from `codex exec --json` (use `tee` so you can replay/ingest)
  - SQLite DB path if you store events/state
  - any generated reports
- identifiers needed to resume:
  - `SESSION_ID` for `codex exec resume ...` (if applicable)
  - `threadId` for `@openai/codex-sdk` thread resume

If you don’t know an ID (because you’re mid-run), explicitly say what output/event contains it and how to find it.

## ExecPlans in multi-agent workflows

Use this division of responsibility:

- **Orchestrator**
  - owns the ExecPlan and keeps it correct
  - sets gates (“only proceed if tests pass”)
  - records decisions and scope changes
  - maintains the canonical list of artifacts
- **Workers**
  - produce scoped outputs only (patches, reports, or JSON)
  - do not invent new milestones or widen scope
  - add evidence snippets (test output, diffs) when asked

## Using ExecPlans with `codex exec`

Recommended operational pattern:

1. Put the plan on disk first (`execplans/...`).
2. Run Codex with JSONL logging:

   codex exec --json "<prompt>" | tee artifacts/codex.jsonl

3. Record in the plan:
   - the prompt (or a stable reference to it),
   - the JSONL path,
   - the session/thread identifiers (when known).
4. If you need to resume:

   codex exec resume <SESSION_ID> "<prompt>"

5. Ingest JSONL to SQLite for audits and querying (optional):

   cat artifacts/codex.jsonl | python3 scripts/codex_jsonl_to_sqlite.py --db codex.sqlite --run-label "..."

## Using ExecPlans with `@openai/codex-sdk`

When running SDK-driven threads, record:

- the `threadId` (so you can resume deterministically),
- the sandbox/approval policy used,
- the schema used for structured output (if any),
- any file-system side effects you expect (created/modified files).

For long work, store streamed events (JSONL) as artifacts and ingest them into SQLite in the same way as CLI JSONL.

