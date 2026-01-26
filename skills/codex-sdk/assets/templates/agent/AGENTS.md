# .agent/AGENTS.md

This folder defines how agentic work is planned, executed, and verified in this repository.

## Working agreements

- Prefer structured outputs for decisions that affect code.
- Default to read-only analysis unless edits are required.
- Keep changes small and verifiable; run checks before declaring done.

## ExecPlans (planning contract)

- Use `.agent/PLANS.md` as the canonical planning standard for multi-step work.
- For any task likely to outlive a single session (multi-hour work, migrations, large refactors, multi-agent workflows), create an **ExecPlan** under `execplans/` and keep it updated.
- When resuming after context compaction or a new session, **re-open the ExecPlan and continue from its Progress section** (do not rely on chat history).
- The orchestrator (human or agent) owns plan correctness; workers should only add scoped evidence and never expand scope unilaterally.
