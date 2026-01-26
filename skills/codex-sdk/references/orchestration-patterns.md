# Orchestration patterns for agentic coding (battle-tested)

These patterns are designed for **correctness-first** coding workflows. Use them with:

- `codex exec` (automation + JSONL)
- `@openai/codex-sdk` (programmatic control)
- Codex MCP server + OpenAI Agents SDK (multi-agent orchestration)

For any work likely to exceed a single session, pair these patterns with an **ExecPlan** (`references/execplans.md`) so state survives compaction and restarts.

## 1) Planner → Executor → Verifier (default)

**Planner**
- produces a short plan with stop conditions
- produces structured “work spec” (files to touch, commands to run, success criteria)

**Executor**
- implements the smallest change set
- writes code + updates tests/docs as needed

**Verifier**
- runs the exact checks (tests/lint/build) and blocks merge if red
- outputs a structured “verdict” object

Key rule: **Only the verifier can say “done”.**

## 2) Orchestrator/Worker with gated handoffs (multi-agent)

Use when you have role specialization (design, backend, frontend, QA, security).

Orchestrator responsibilities:

- maintains a single source of truth state: `run_id`, `threadId`s, artifact list, gates
- validates gates (file existence, test pass)
- decides which worker runs next and with what context

Worker responsibilities:

- produce specific artifacts only (files, patches, structured report)
- do not self-expand scope

## 3) Evaluator–optimizer loop (hard problems)

Use when changes are subtle or regressions are likely.

- Optimizer proposes a fix.
- Evaluator checks against explicit rubric and rejects/accepts.

Keep evaluator strict; require citations, line ranges, and reproducible commands.

## 4) Parallelization (safe parallel work)

Parallelize only when tasks are **write-disjoint**:

- different directories or services
- no shared config files
- no shared migrations

Otherwise, parallelize analysis only, then merge plans into a single executor.

## 5) Idempotent, resumable runs

To make runs resumable:

- persist `threadId` and run metadata in SQLite
- record JSONL event stream for every run
- make every step safe to re-run (use `--dry-run`, `git diff`, `git apply --check`)

## 7) ExecPlan as the “durable state machine”

For long tasks, treat the ExecPlan as the durable source of truth:

- the planner writes/updates the plan (milestones, gates, acceptance)
- the executor implements only what the plan currently says
- the verifier updates Progress with evidence and blocks scope creep

Record “resume pointers” (IDs, artifacts, next command to run) so you can restart from disk with minimal context.

## 6) Context control (avoid token blowups)

- do not paste entire repos into prompts
- feed targeted diffs and file lists
- use RAG: store “index artifacts” in SQLite (file inventory, module graph) and retrieve on demand
