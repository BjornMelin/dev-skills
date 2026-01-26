# Migration: `langgraph-supervisor(-py)` → modern LangChain/LangGraph

## Why migrate

Current guidance generally recommends building the supervisor pattern **directly via tools/subagents**, using:

- LangChain v1 `create_agent` (middleware, runtime context)
- LangGraph v1+ primitives for durable execution and checkpoints

Keep `langgraph-supervisor` only when it is clearly net-positive for your codebase and matches your pinned versions.

## Migration outline (safe, staged)

1. **Inventory current behavior**
   - Identify supervisor creation (`create_supervisor`, handoff tools, routing rules).
   - Add tests around the externally observable behavior (tool calls, outputs, error handling).
2. **Extract tool layer**
   - Move business operations to standalone LangChain tools with strict schemas.
   - Ensure side-effectful tools are idempotent or protected by HITL.
3. **Port subagents**
   - For each worker agent: create `create_agent(model, tools=..., system_prompt=..., middleware=...)`.
   - Wrap subagents as tools callable by the supervisor (tool-calling pattern).
4. **Port supervisor**
   - Implement supervisor as `create_agent` with the subagent-wrapper tools.
   - If you need strict ordering: encode it in the supervisor system prompt and/or deterministic graph edges.
5. **Add durability + HITL**
   - Add checkpointer for pause/resume and thread memory.
   - Add HITL middleware or graph-level `interrupt` for sensitive tools.
6. **Remove legacy wiring**
   - Delete supervisor-lib-specific adapters only after parity tests pass.

## What to validate explicitly

- Message/state handoff semantics (what context is passed to subagents).
- Streaming behavior (node naming, step boundaries).
- Error handling: tool exceptions, retry behavior, and “fail open” decisions.
- Any hook replacements:
  - pre/post model hooks → middleware (`before_model` / `after_model`)
  - tool error handling → middleware wrappers

