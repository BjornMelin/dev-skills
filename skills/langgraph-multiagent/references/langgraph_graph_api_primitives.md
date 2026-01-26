# LangGraph Graph API primitives (Python)

Use this when you need to drop below `create_agent` and build custom workflows/agents directly in LangGraph.

## Mental model

- A graph is nodes (functions) + edges (control flow) over a shared **state**.
- Nodes read state, do work, and return **partial state updates** (dict).
- Conditional edges encode routing decisions explicitly.
- Durable execution comes from **checkpointing**.

## State schemas

### Messages-first state (chat apps)

- Use `MessagesState` for message history, because it includes the right reducer semantics for chat message lists.
- Prefer *structured fields* for important state (IDs, flags, plans), not only free-form text.

### Reducers (required for parallelism)

Parallel fanout requires deterministic merge semantics.

- For append-only lists: use a reducer like `operator.add` on a list field.
- For message lists: use LangGraph’s `add_messages` reducer (built in).

If you ignore this, you’ll hit `INVALID_CONCURRENT_GRAPH_UPDATE`.

## Common nodes/edges patterns

- **Single-pass workflow**: `START → step1 → step2 → END`
- **Agent loop**: conditional edge checks “has tool calls?” to continue
- **Router**: conditional edges based on state flag or last message content
- **Fanout**: orchestrator returns a list of `Send(...)` instructions to spawn workers

## Interrupts and human-in-the-loop

- Use `interrupt()` to pause a graph at a node boundary for human input.
- Interrupts require a checkpointer; without one you’ll hit `MISSING_CHECKPOINTER`.

## Subgraphs

Use subgraphs to:

- encapsulate complex agent behavior behind a stable interface
- isolate per-agent/private message histories

Two ways:

1) Invoke a compiled graph from inside a node (schemas can differ)
2) Add a compiled graph directly as a node (shares state keys)

Known gotcha:

- checkpointing + multiple subgraphs invoked in the same node can trigger `MULTIPLE_SUBGRAPHS` unless structured carefully.

## Troubleshooting: high-frequency error codes

See the common errors reference; the most common in multi-agent systems:

- `INVALID_CONCURRENT_GRAPH_UPDATE`: parallel updates without reducers
- `INVALID_GRAPH_NODE_RETURN_VALUE`: node returned non-dict
- `GRAPH_RECURSION_LIMIT`: unintended infinite loops
- `INVALID_CHAT_HISTORY`: malformed message sequence (often from bad handoff history)

## When to use LangChain vs LangGraph directly

- Use LangChain `create_agent` when the agent loop is standard and you want middleware.
- Use LangGraph directly when you need deterministic control-flow, bespoke multi-agent routing, or custom durability boundaries.

