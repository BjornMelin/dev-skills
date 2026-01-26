# Context engineering + memory (state, store, runtime context)

This is the “operating manual” for keeping multi-agent systems scalable and reliable.

## The 3-layer context model

### 1) Static runtime context (per run)

Immutable dependencies injected at invocation time (dependency injection):

- user/org IDs
- DB clients
- API credentials (never show to model)
- feature flags / permissions

In LangChain v1 this is typically provided via the `context=` argument (shaped by `context_schema=`).

### 2) Dynamic runtime context (state)

Mutable data during a run:

- messages
- intermediate results
- routing flags (`active_agent`, `next_step`)
- extracted entities (IDs, dates, etc.)

### 3) Dynamic cross-conversation context (store)

Long-term memory, shared across threads/sessions:

- user preferences (tone, verbosity)
- stable facts (timezone, role)
- task-specific durable artifacts (when safe)

Use namespaces (often `(user_id, app_context)` or `(org_id, user_id, domain)`).

## Short-term vs long-term memory

- Short-term memory is state persisted via **checkpointer** (thread-scoped).
- Long-term memory is a **store** (cross-thread) with optional semantic search.

## Practical playbook

### Keep messages small

- Use summarization middleware or explicit summary fields.
- Delete stale messages when safe (requires message reducers).
- Store raw artifacts outside the LLM prompt; only reference them via IDs + short summaries.

### Store the right things

Good long-term memories:

- preferences (“user likes short answers”)
- stable identifiers (“account_id”, “timezone”)
- rules/instructions (“always confirm before sending email”)

Bad long-term memories:

- full transcripts
- secrets / API keys
- large raw documents (store pointers/IDs instead)

### Multi-agent memory strategy

- Keep subagents stateless; the supervisor owns memory.
- If agents must hand off, pass only the minimum validated message/tool pair plus a compact summary.

## ToolRuntime (why it matters)

Modern LangChain/LangGraph stacks provide a runtime object to tools/middleware/interceptors that can expose:

- `state` (conversation state)
- `config` (thread IDs, run config)
- `store` (long-term memory)
- `tool_call_id` (for ToolMessage correctness)

Use this to:

- enforce auth/permissions
- implement per-user namespaces
- write progress events / telemetry

