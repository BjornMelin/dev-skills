# Audit + migration methodology (architect-grade)

Use this to perform deep reviews and produce actionable migration plans.

## Phase 1: Inventory

1. Identify entrypoints:
   - agent constructors, graph builders, tool registries, UI bindings.
2. Capture dependencies and versions.
3. Run automated scan:
   - `python scripts/audit_repo_agents.py --root . --out agent_audit_report.md --json agent_audit.json`

## Phase 2: Classify architecture

For each agent workflow, classify:

- topology: single-agent / supervisor-subagents / handoffs / orchestrator-worker
- memory: none / thread checkpointer / store long-term memory
- safety: guardrails / HITL / permissions
- observability: tracing + metrics + evaluation

## Phase 3: Define target architecture (keep it minimal)

Pick the simplest that meets requirements:

1) single `create_agent` + middleware (best default)
2) supervisor + subagents (tool-calling)
3) graph-native orchestration (LangGraph StateGraph)

## Phase 4: Migration plan (ship in slices)

Generate a draft plan automatically:

- `python scripts/generate_migration_plan.py --audit-json agent_audit.json --out migration_plan.md`

Then refine:

- add concrete file-level tasks
- add tests for each migration slice
- add rollout plan and monitoring

## Phase 5: Execute + harden

- migrate wiring with minimal behavior changes
- add middleware guardrails + HITL for side effects
- add evaluation/regression suite for drift

