# Deployment notes (LangGraph/LangChain)

## Deployment options (typical)

- Local dev: run graphs directly in-process.
- Service deployment: expose graphs behind an API service; add auth, rate limits, and auditing.
- Durable execution: enable checkpointing/persistence and thread IDs.

## What to decide early

- Where checkpoints live (in-memory vs DB-backed).
- Multi-tenant boundaries (namespace design for long-term memory stores).
- Secret management for tools (never ship secrets to the model; inject at runtime context).

## “Don’t ship without”

- tests around core tool calls
- traceability (run IDs, thread IDs)
- safe timeouts + retries
- least-privilege tool credentials

