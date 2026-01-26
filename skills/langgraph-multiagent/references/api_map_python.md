# Python API map (LangChain v1 + LangGraph v1+)

This is a **cheat sheet**; always verify with docs for your pinned versions.

## Agents + middleware

- `from langchain.agents import create_agent`
- `from langchain.agents import AgentState` (for typed state extension; confirm export in your version)
- `from langchain.agents.middleware import HumanInTheLoopMiddleware, PIIMiddleware, SummarizationMiddleware`
- `from langchain.agents.middleware import before_model, after_model, wrap_model_call, wrap_tool_call, dynamic_prompt`

## Tools + runtime access

- `from langchain.tools import tool, ToolRuntime`
- `from langchain.messages import ToolMessage, AIMessage` (handoffs and tool error shaping)

## LangGraph primitives

- `from langgraph.graph import StateGraph, START, END, MessagesState`
- `from langgraph.types import Command, Send, interrupt`
- `from langgraph.checkpoint.memory import InMemorySaver` (dev only; use persistent checkpointer in prod)
- `from langgraph.store.memory import InMemoryStore` (dev; use DB-backed store in prod)

## Common patterns to look up in docs

- reducers: `add_messages`, `operator.add`, per-field reducer annotations
- time travel: state history + checkpoint IDs
- persistence: thread_id config + checkpointer backends
- multi-agent handoffs: `Command.PARENT` and message pairing rules

