# Observability (LangSmith + OpenTelemetry)

## Minimum viable observability

Record these per run:

- node-level latency + errors
- tool call count + latency + error taxonomy
- token usage (prompt/completion) and estimated cost
- cache hit rate (retrieval/tool/memory)
- stop reasons (max steps, recursion limit, human interrupt, tool failure)

## Tracing strategy

1. Use LangSmith traces for rapid debugging and UX iteration (when available).
2. Use OpenTelemetry for org-standard metrics/logs/traces pipelines.
3. Ensure propagation of:
   - thread/run identifiers
   - user/org identifiers (redacted or hashed)
   - model/provider metadata

## Production hardening checklist

- Add sampling for high-throughput endpoints.
- Redact PII from spans/logs by default.
- Add “golden path” regression traces to detect behavior drift after upgrades.

