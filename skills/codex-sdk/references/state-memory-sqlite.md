# State, caching, and memory with SQLite

SQLite is the simplest “shared brain” for multi-step and multi-agent runs:

- durable storage of run inputs/outputs
- indexing runs by repo/branch/purpose
- caching expensive computations
- storing `threadId` for resuming Codex sessions

## What to store

At minimum:

- `runs`: one record per workflow run (label, created_at, metadata)
- `events`: append-only JSONL events with extracted columns for filtering

Optionally:

- `artifacts`: paths/hashes of important output files
- `kv_cache`: key/value cache for derived summaries

## How to ingest JSONL

Use `scripts/codex_jsonl_to_sqlite.py`:

- safe for repeated ingestion
- stores raw JSON for audit
- extracts `type`, `thread_id`, item types, and usage where available

## Use cases

- **Resume**: retrieve the latest `threadId` by run label, then call `codex-reply`.
- **Audit**: answer “what commands did the agent run?” by filtering `command_execution` items.
- **Cost control**: compute total token usage per run (`turn.completed` events).
- **Caching**: memoize expensive steps (diff summaries, dependency graphs).

