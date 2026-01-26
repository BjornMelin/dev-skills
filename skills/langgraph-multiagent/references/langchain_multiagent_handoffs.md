# Multi-agent patterns: supervisor/subagents vs handoffs vs subgraphs

This reference is about **multi-agent orchestration patterns** in the LangChain/LangGraph ecosystem and how to choose the simplest one that meets requirements.

## Default recommendation

Start with **supervisor + subagents via tool-calling** (LangChain multi-agent “subagents” pattern) unless you have a strong reason to do more.

Why:

- Subagents are stateless and run in a clean context window → less context bloat.
- The supervisor keeps the conversation state and can combine results deterministically.
- Fits well with middleware guardrails and HITL.

## Pattern A: Supervisor + subagents (tool-calling)

Structure:

1. Define low-level tools (API calls).
2. Build specialized subagents with domain toolsets.
3. Wrap each subagent behind a single “high-level” tool for the supervisor.
4. Supervisor uses those wrapper tools to route work.

Best for:

- specialization across domains
- clean context isolation
- simple routing via prompt/tool descriptions

Key best practices:

- Make wrapper tools accept **one explicit request** string or a small structured schema.
- Pass only relevant context to subagents; do not dump the full message history by default.
- Prefer sequential tool calls for dependent operations; parallelize only when safe.

## Pattern B: Handoffs (state-driven behavior changes)

Core idea:

- A tool updates a persistent state variable (e.g., `active_agent` / `current_step`).
- A router (or middleware) uses that state to change behavior: prompts, tools, or next node.

Two main implementations:

1) **Single agent + middleware (recommended for most handoff use cases)**
- One agent; middleware switches prompt/tools based on state.
- Simpler than multiple graph nodes.

2) **Multiple agent nodes/subgraphs (only when you need truly distinct agent implementations)**
- Each agent is its own node/subgraph.
- Handoff tools return `Command(goto=..., update=..., graph=Command.PARENT)`.

### Handoff correctness invariants (multi-node/subgraph approach)

If you hand off via a tool call, the receiving agent must see a valid “tool call → tool result” pair in history:

- The `AIMessage` containing the tool call that triggered the handoff
- A synthetic `ToolMessage` acknowledging the handoff (the tool “result”)

Do not forward entire subagent transcripts by default. Prefer:

- (a) a short summary in the ToolMessage, plus
- (b) structured artifacts stored in state/store for retrieval if needed

## Pattern C: Orchestrator-worker (fanout + aggregation)

Core idea:

- Orchestrator generates tasks.
- Workers run in parallel.
- Results aggregate into a shared state key with reducers.

Use when:

- tasks are independent (parallelizable)
- you need explicit aggregation logic

Key requirement:

- never let parallel nodes update the same state key unless it has a reducer (see `references/langgraph_graph_api_primitives.md`).

