# Docs index (always fetch the latest)

This skill is intentionally **docs-driven**: do not rely on memory for APIs. Use `langchain-docs.SearchDocsByLangChain` first, then Context7 for API-level details.

## Primary doc entrypoints (Python)

- Multi-agent overview: search `langchain multi-agent` (LangChain OSS Python).
- Subagents / supervisor pattern tutorial (migration target for supervisor libs): search `subagents-personal-assistant`.
- Handoffs (state-driven routing): search `handoffs` and `Command.PARENT`.
- Guardrails: search `langchain guardrails` (middleware, PII/prompt-injection checks, HITL).
- Middleware (core of create_agent): search `decorator-based middleware` and `custom middleware`.
- Context engineering: search `context overview` (runtime context, state, store).
- MCP integration: search `langchain mcp` (adapters, ToolRuntime context).
- Human-in-the-loop: search `human-in-the-loop` and `interrupt`.
- Retrieval + agentic RAG: search `langgraph agentic rag` and `langchain retrieval`.
- Long-term memory: search `langchain long-term memory` (stores, namespaces, search).
- LangGraph Graph API + reducers: search `MessagesState` and `Process state updates with reducers`.
- LangGraph patterns: search `thinking in langgraph` and `workflows and agents` and `orchestrator-worker`.
- LangGraph persistence: search `persistence` and `time travel`.
- LangGraph app config/deployment: search `langgraph.json` and `LangGraph CLI` and `application structure`.
- Releases & migrations:
  - search `langgraph v1` (what’s new)
  - search `langchain v1` (what’s new)
  - search `migrate langgraph v1` (e.g., `create_react_agent → create_agent`)
  - search `migrate langchain v1` (migration table + behavior changes)

## UI + frontend streaming (JS/TS + Next.js)

When integrating LangGraph/LangChain into Next.js/React UIs, prioritize these docs:

- `useStream` hook + frontend streaming guide:
  - search `streaming frontend useStream`
  - search `useStream return values` (interrupts, toolCalls, branching, resume)
- Thread management patterns:
  - search `Thread management` and `Optimistic thread creation`
  - search `Resume after page refresh reconnectOnMount`
- Tool call rendering:
  - search `Rendering tool calls getToolCalls`
- Generative UI (React UI components from the graph):
  - search `generative-ui-react`
  - search `LoadExternalComponent react-ui`
  - search `uiMessageReducer onCustomEvent`
- Agent Server + SDK streaming semantics:
  - search `Create Run, Stream Output`
  - search `Join Run Stream Last-Event-ID`
  - search `Join Thread Stream`
- Security/gov for multi-tenant UIs:
  - search `configurable headers`
  - search `logging headers`
  - search `agent server scale`

Alternative UI stack (Node/TS runtime):

- AI SDK v6 (`useChat`, `streamText`, UI stream protocol):
  - see `references/ui_nextjs_ai_sdk.md`
  - Context7: `/vercel/ai`
- Streamdown (streaming-safe markdown renderer):
  - skill: `/home/bjorn/.codex/skills/streamdown/SKILL.md`

## LangGraph doc sitemap (`llms.txt`)

Use `llms.txt` as the authoritative index of LangGraph documentation pages (agentic RAG friendly).

- Extract URLs: run `python scripts/fetch_llms_txt_urls.py --print --unique` (from the skill folder)
- Then crawl selectively (don’t blindly fetch everything):
  - pick relevant URLs based on the task
  - fetch content with `mcp__exa__crawling_exa` or `web.run`
  - synthesize a “current best practice” summary tied to the installed versions

## “Must use docs” triggers

Always consult docs before implementing if any of these are true:

- You’re touching agent creation APIs (`create_agent`, deprecated prebuilt agents, middleware).
- You’re implementing supervisor/subagent handoffs, parallel fanout, or reducers.
- You’re using `interrupt`, checkpointing, stores, or persistence.
- You’re dealing with runtime context injection (`ToolRuntime`, `context`, `store`).
- You’re implementing MCP integrations or tool adapters.
- You’re migrating from `langgraph-supervisor(-py)` or `create_react_agent`.
- You’re building a UI that depends on streaming (`useStream`, SSE, resume/branching, tool-call UIs).
