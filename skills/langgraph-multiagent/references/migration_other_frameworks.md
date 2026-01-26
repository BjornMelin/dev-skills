# Migration playbook: other agent frameworks → LangGraph/LangChain

Use this when a repo uses mixed stacks (LlamaIndex agents, CrewAI, Agno, OpenAI Agents). The strategy is always:

1) **stabilize behavior with tests**, 2) **extract tools**, 3) **rebuild orchestration**, 4) **add observability**, 5) **delete legacy**.

## LlamaIndex agents → LangGraph/LangChain

Typical mappings:

- LI “agent” orchestration → LangGraph graph + LangChain `create_agent`
- LI tools → LangChain tools (`@tool` / BaseTool)
- LI retrieval/indexing → either:
  - keep LI indexing where it’s uniquely valuable, but call it from LangChain tools, or
  - migrate to LangChain retrievers/vector stores if alignment/simplicity is preferred

Key risk: duplicated memory / state / tracing layers.

## CrewAI → supervisor/subagents

CrewAI concepts:

- “crew/roles” → subagents
- “tasks” → tool calls or deterministic workflow nodes
- “manager” → supervisor agent

Migration steps:

1. Define per-role toolsets.
2. Convert each role to a subagent.
3. Implement a supervisor that delegates via tool calls.
4. Add HITL for side effects; add thread-level memory/checkpointing.

## Agno → graph-native orchestration

Treat Agno’s orchestration as “workflow + tools”. Rebuild the orchestration as a graph:

- explicit nodes for state changes
- conditional edges for routing decisions
- middleware for policy enforcement

## OpenAI Agents / Responses API → LangChain `create_agent`

Strategy:

1. Extract tools into provider-agnostic LangChain tools.
2. Replace agent loop with `create_agent`.
3. Implement middleware guardrails for:
   - prompt injection checks
   - PII filtering
   - tool approvals
4. Keep the model provider swappable; validate tool-call JSON schemas end-to-end.

