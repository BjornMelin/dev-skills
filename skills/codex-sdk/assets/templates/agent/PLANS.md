# .agent/PLANS.md (ExecPlans)

This file defines the planning standard for **ExecPlans**: living, self-contained design + execution documents for work that can’t reliably fit in a single chat/session.

ExecPlans exist to survive:

- context window limits,
- conversation compaction,
- agent restarts,
- multi-agent handoffs.

## When to create an ExecPlan

Create an ExecPlan when the task is any of the following:

- multi-hour work or multi-PR work,
- migrations, refactors, or “touch many files” changes,
- high-risk changes (security, data loss, auth, destructive ops),
- anything where a future contributor must be able to resume from disk.

## Non-negotiables

1. **Self-contained**
   - Assume the reader has only a fresh repo checkout + this ExecPlan file.
   - Include definitions for any non-obvious term you use.
   - Include exact commands, file paths, and expected outputs.

2. **Living document**
   - Update **Progress**, **Surprises & Discoveries**, and **Decision Log** continuously.
   - Before stopping, record “what’s done” and “what’s next” in Progress.

3. **Outcome-first**
   - Acceptance is phrased as observable behavior (commands + expected output), not internal implementation attributes.

4. **Idempotent and safe**
   - Steps should be safe to re-run.
   - If a step can fail halfway, include how to recover.

## How to use ExecPlans

- **Authoring**: start from the template; fill in repo-specific context and make the plan executable by a novice.
- **Executing**: treat the ExecPlan as the single source of truth; proceed milestone-by-milestone without asking for “next steps”.
- **Resuming**: re-open the ExecPlan and continue from Progress; do not depend on chat history.

## Formatting rules (avoid broken plans)

- Prefer prose; use checklists only in **Progress**.
- Avoid nested triple-backtick fences inside an ExecPlan. Use indentation for:
  - commands
  - transcripts
  - code excerpts
  - diffs
- If an ExecPlan is embedded inside another document, wrap it in a single fenced block labeled `md`.
- If an ExecPlan is the entire contents of its own `.md` file, do not wrap it in triple backticks.

## Where ExecPlans live

Store ExecPlans under `execplans/`:

- `execplans/execplan-<short-name>.md`

## ExecPlan template

Copy from `assets/templates/execplan.md` (or generate one with the skill script `scripts/new_execplan.py`).
