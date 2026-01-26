# Pattern library (multi-agent in LangGraph/LangChain)

## 0) Default recommendation

Prefer **supervisor + subagents via tool-calling** unless you have a specific need for:

- strict deterministic control-flow (workflow-heavy)
- parallel fanout (orchestrator-worker)
- deeply customized state transitions or checkpoints

## 1) Supervisor + subagents (tool-calling)

Core idea:

- Each subagent is a focused `create_agent(...)` with its own tools + prompt.
- The supervisor calls subagents as tools to get context isolation and prevent context bloat.
- Human-in-the-loop and guardrails are implemented via middleware (preferred) or graph interrupts.

Checklist:

- Keep subagents stateless; the supervisor owns memory/state.
- Use strict tool schemas; validate inputs before tool side effects.
- For sensitive tools, require approval (HITL middleware).
- Make supervisor prompts explicit about when to delegate.

## 2) Orchestrator-worker (fanout + aggregation)

Core idea:

- Orchestrator emits a dynamic set of worker tasks.
- Workers run independently and write results into a dedicated state key.
- Orchestrator aggregates results and decides whether to continue.

Checklist:

- Never let multiple nodes write to the same state key without a reducer.
- Design aggregation state so it can be merged deterministically.
- Bound concurrency; handle timeouts and partial results.

## 3) Hybrid workflow + agents

Core idea:

- Keep control-flow deterministic where possible.
- Use LLM decisions only at “routing” seams.

Checklist:

- Separate *policy* (routing) from *mechanics* (tools, IO).
- Make errors part of the flow: retries, loop-backs with context, pause for human, or bubble up.

## 4) Context + memory (three layers)

Use the correct layer for the correct job:

- **Static runtime context**: per-run dependencies (user ID, DB client, API clients).
- **Dynamic runtime context (state)**: mutable per-run state transitions.
- **Dynamic cross-conversation context (store)**: long-term memory / preferences / user profile.

## 5) Safety primitives

- Guardrails: deterministic checks (regex, allowlists) + model-based evaluations when needed.
- HITL: approvals for irreversible actions.
- Tool design: idempotency, timeouts, explicit side-effects, least-privilege credentials.

