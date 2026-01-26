# UI integration backend: FastAPI for LangGraph/LangChain agents

This guide focuses on running the agent runtime behind **FastAPI** and integrating with web UIs.

## Choose your backend mode

### Mode 1 (recommended when you want the full LangGraph stack): Agent Server

Run LangGraph Agent Server (locally via `langgraph dev`, or deployed) and have your UI call it directly using the JS SDK (`useStream`) or REST API.

Use FastAPI only as a “BFF” if you need:

- custom auth/session integration
- request shaping / logging / governance
- cross-service orchestration
- a single domain/origin for the browser (avoid CORS issues)

### Mode 2: In-process LangChain/LangGraph inside FastAPI

You own:

- persistence (thread state, checkpoint storage)
- streaming protocol to the browser
- HITL resume endpoints

Use when:

- you need full control and won’t use Agent Server

## Streaming transport

For browser UIs, prefer **SSE** for token streaming:

- works over plain HTTP
- proxies/CDNs generally handle it well
- client reconnection behavior is simpler than WS for “server → client” streams

Use WebSockets only if you need **bi-directional** real-time channels beyond request/response.

## FastAPI endpoint design (recommended)

Keep APIs minimal and explicit:

- `POST /chat/stream` → SSE stream of events (token/update/custom/interrupt/done)
- `POST /chat/resume` → resume a paused thread with HITL decisions (or fold into the stream endpoint)
- `POST /threads` → create a new thread ID (optional; the stream endpoint can also create one)

Always require a `thread_id` for:

- long-running conversations
- HITL pause/resume
- time travel/branching (if implemented)

## In-process runtime essentials (LangGraph/LangChain)

If you run the agent inside FastAPI:

- Use `create_agent(...)` (LangChain v1) or a compiled LangGraph `StateGraph`.
- Provide a **checkpointer** (required for interrupts / resume and for durable threads).
- Use `config = {"configurable": {"thread_id": thread_id}}` for thread scoping.
- Stream with `stream_mode=["messages", "updates", "custom"]` so the UI can show:
  - tokens (`messages`)
  - step/state deltas + interrupts (`updates`)
  - progress bars + tool traces (`custom`)

For HITL resumption, accept a resume payload and invoke with `Command(resume=...)` (exact schema is versioned and comes from the interrupt itself).

## Security + governance

- Never accept tool args from the client as “trusted”. Tool args must come from the model + schema validation.
- Use auth to gate tool access (per user/org).
- Add rate limits and timeouts per tool call.
- Redact PII from logs/spans by default.
- Prefer least-privilege credentials injected at runtime (server-side), not stored in graph state.

## Template shipped with this skill

- `assets/templates/fastapi/fastapi_sse_multiagent.py`:
  - shows SSE streaming from `agent.astream(...)`
  - emits typed JSON events compatible with a Next.js client

Pair it with `references/ui_streaming_protocol.md` for the event schema.
