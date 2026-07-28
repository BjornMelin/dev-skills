# Codex Artifact Model

Codex cleanup spans storage surfaces under `CODEX_HOME`, defaulting to
`~/.codex`. The scanner separates scope roots from artifact families so one
workflow can handle a single repo, explicit roots, cwd subrepos, Codex-home
artifacts, or all sessions.

## Primary Surfaces

- `sessions/**/*.jsonl`: active rollout/session logs.
- `archived_sessions/*.jsonl`: archived rollout/session logs.
- `history.jsonl`: prompt history keyed by `session_id`.
- `session_index.jsonl`: lightweight thread id to thread name index.
- `state_5.sqlite`: thread metadata, goals, spawn edges, dynamic tools, memory
  stage outputs, agent jobs, and related state.
- `logs_2.sqlite`: structured logs keyed by `thread_id`.
- `memories/MEMORY.md`: memory registry.
- `memories/rollout_summaries/*.md`: generated memory summaries linked to
  session ids and rollout paths.
- `prune-quarantine/`: timestamped cleanup quarantines and scan reports.

## Scope Resolution

- `current`: current Git repository root when available; otherwise the cwd.
- `root`: one or more explicit `--root` values.
- `roots`: explicit roots from repeated `--root` and/or `--roots-file`.
- `cwd-subrepos`: Git repositories discovered under `--cwd`.
- `codex-home`: Codex-home artifact families not primarily owned by a repo.
- `all`: all repo-bound sessions plus requested Codex-home artifact families.

For repo-bound sessions, matching uses `threads.cwd` from `state_5.sqlite`.
Nested repositories default to longest-root-wins ownership.

## Artifact Families

- `sessions`: session JSONL, history/index JSONL, state/log SQLite rows.
- `memory`: memory registry plus linked rollout summaries; copy-first.
- `quarantine`: previous quarantine bundles; purge is separate.
- `logs`: local Codex log files/directories.
- `cache`: cache and temp directories.
- `generated`: generated images or other generated Codex outputs.
- `skills-agents-config`: skills, agents, AGENTS/config/hook files.

`auth.json`, `internal_storage/**`, and secret-like env files are protected
unless the user asks for a separate security-reviewed workflow.

## Quarantine Layout

Each apply creates:

- `manifests/manifest.json`: exact selected sessions and planned operations.
- `manifests/result.json`: actual mutations and integrity outcomes.
- `SHA256SUMS`: checksums for quarantined copies and backups.
- `db_backups/`: SQLite and history/index backups.
- `session_files/`: selected session JSONL files.
- `memory_rollout_summaries/`: copied or explicitly moved memory snapshots.

Plain Codex logs and broad config surfaces are report-only by default.
