# Upgrades + versioning playbook (LangGraph/LangChain)

This is the repeatable process for staying “latest best practice” without breaking production.

## 0) Establish the truth

1. Identify pinned versions (lockfiles / constraints).
2. Confirm installed versions (runtime truth).
3. Snapshot dependency sources for edge cases:
   - `python scripts/opensrc_snapshot.py --packages langgraph langchain langchain-core`

## 1) Read before you code

- Search release notes + migration guides with `langchain-docs.SearchDocsByLangChain`.
- Use Context7 for API-level confirmations (signatures, import paths).

## 2) Run the repo audit

- `python scripts/audit_repo_agents.py --root . --json agent_audit.json`
- Generate a migration plan:
  - `python scripts/generate_migration_plan.py --audit-json agent_audit.json --out migration_plan.md`

## 3) Migrate in slices

Recommended order:

1. tool layer (schemas, retries, idempotency)
2. orchestration wiring (agent loop / graph)
3. memory (checkpointer + store)
4. guardrails + HITL (middleware)
5. observability + evaluation

## 4) Known modernization themes

- `langgraph.prebuilt.create_react_agent` → `langchain.agents.create_agent`
- hooks → middleware (`before_model`, `after_model`, `wrap_tool_call`, `dynamic_prompt`)
- custom state → TypedDict only in modern agent stacks

Always confirm details against the docs for the exact versions you ship.

