# Deployment: LangGraph CLI / Agent Server (langgraph.json)

Use this when packaging a LangGraph/LangChain agent as a deployable service with durable execution.

## Minimal application structure

A deployable app typically includes:

- one or more compiled graphs (LangChain `create_agent` returns a compiled graph)
- `langgraph.json` (configuration)
- dependency spec (`pyproject.toml` or `package.json`)
- optional `.env`

## Minimal `langgraph.json` (Python)

The smallest useful config maps graph IDs to Python module objects:

```json
{
  "dependencies": ["."],
  "graphs": {
    "agent": "./src/agent.py:agent"
  },
  "env": ".env"
}
```

Keep `graphs` stable: treat graph IDs as API contracts.

## HTTP headers → runtime config (multi-tenant safety)

Agent Server can map selected HTTP headers into the runtime `config` (for user/org IDs, feature flags, budgets). For safety, explicitly allowlist which headers are passed through:

```json
{
  "http": {
    "configurable_headers": {
      "includes": ["x-user-id", "x-organization-id", "my-prefix-*"],
      "excludes": ["authorization", "x-api-key"]
    }
  }
}
```

Related: you can also opt-in to logging specific headers for correlation/debugging (keep PII redacted by default).

## Streaming + resumability (runs/threads)

- Agent Server streams outputs via SSE for run endpoints.
- If a run is created with `stream_resumable=true`, clients can reconnect using `Last-Event-ID` to resume from the last seen event ID (see the Agent Server API `Join Run Stream` endpoint).
- For UX robustness (page refresh, flaky networks), prefer resumable streaming and store `run_id` while a run is active.

## Operational checklist

- Add auth + rate limiting at the service boundary.
- Use persistent checkpointing in production (not in-memory).
- Configure store/semantic search carefully for multi-tenant isolation.
- Redact logs/spans by default (PII).
- Configure cancellation behavior (`cancel_on_disconnect`) for long streams when appropriate.
- Review Agent Server scale guidance if you expect high read/write or many concurrent runs.

## Where to look next

- LangGraph “application structure” + config reference (schema evolves; always consult latest docs).
- MCP endpoint support and auth middleware if exposing tools/agents via MCP.
