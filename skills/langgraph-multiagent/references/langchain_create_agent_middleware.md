# LangChain v1 `create_agent` + middleware (Python)

Use this as the canonical playbook for building production agents in LangChain v1 (built on LangGraph) while staying version-correct.

## Ground rules

- Treat all APIs as **versioned**: verify with `langchain-docs.SearchDocsByLangChain` + Context7 before coding.
- Prefer **middleware** for policy and cross-cutting concerns (context engineering, guardrails, retries, logging) instead of embedding that logic inside tools/nodes.

## The modern surface area (what matters)

### Agent creation

- Build agents with `langchain.agents.create_agent`.
- Prefer small, typed tool schemas and deterministic tool behavior.
- Add `checkpointer=` when you need durability features like **interrupts / human-in-the-loop** and thread persistence.

### Middleware hooks (Python)

LangChain supports both decorator-based middleware (simple, single-hook) and custom middleware implementations (complex, configurable). Common hooks:

- `@before_agent`: once, before agent starts
- `@before_model`: before each model call
- `@after_model`: after each model response
- `@after_agent`: once, after agent completes
- `@wrap_model_call`: wrap each model call (modify request/response, retries)
- `@wrap_tool_call`: wrap each tool call (retries, error shaping, auth gates)
- `@dynamic_prompt`: generate dynamic system prompts

Use `references/memory_and_context_engineering.md` for “what to inject” and how to keep context small.

## Built-in middleware to know (Python)

These are the “default toolkit” for production agents:

- **Human-in-the-loop**: `HumanInTheLoopMiddleware(interrupt_on={...})`
- **PII**: `PIIMiddleware(pii_type, strategy=..., apply_to_input=..., apply_to_output=..., apply_to_tool_results=...)`
- **Summarization**: `SummarizationMiddleware(model=..., trigger={...})`
- **Tool emulation** (useful for tests / dry-runs): `LLMToolEmulator(...)`

Names/params are versioned; confirm via docs for your pinned versions.

## Patterns (battle-tested)

### 1) Dynamic tool exposure (accuracy + cost)

Goal: reduce prompt/tool-choice entropy by presenting only a relevant subset of tools per turn.

- Implement as `@wrap_model_call` and override `request.tools` (or equivalent override API).
- Keep the global tool registry stable; select a subset dynamically.
- Apply permissions here too (user/org allowlists).

### 2) Tool error shaping (don’t crash the run, but don’t hide real bugs)

Goal: treat user-caused/runtime tool failures as tool feedback, while surfacing genuine implementation bugs.

- Implement as `@wrap_tool_call`.
- Catch expected “runtime input errors” (e.g., invalid SQL syntax) and return a `ToolMessage` with a helpful error.
- Do **not** swallow:
  - network/system outages (use retry middleware + circuit breakers)
  - programming errors (let them bubble up)
  - schema validation errors (the framework handles these)

### 3) Human-in-the-loop for side effects

Goal: force review for irreversible operations.

- Gate: outbound comms, destructive writes, payments, privileged ops.
- HITL requires a `checkpointer=...` and a `thread_id` in config so the run can pause/resume.

### 4) Context injection as policy (not prompt sprawl)

Goal: keep LLM context small and relevant.

- Load memory/preferences in `@before_model` and inject a short structured snippet.
- Use long-term store for *facts/preferences*, not transcripts.
- For large retrieved context, summarize or attach as artifacts rather than stuffing into the model prompt.

## Migration notes: `create_react_agent` → `create_agent`

When migrating legacy LangGraph prebuilt agents:

- `prompt` → `system_prompt` (dynamic prompts → middleware)
- pre/post hooks → middleware (`before_model` / `after_model`)
- tool error handling → middleware (`wrap_tool_call`)
- runtime context injection → `context=` argument (thread state still uses config/thread_id)
- custom state: TypedDict only (no Pydantic/dataclass state schemas in `create_agent`)

Use `references/upgrades_and_versioning.md` and the official migration guides to confirm details.

