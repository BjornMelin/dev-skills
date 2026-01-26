# RAG + shared memory (SQLite-first)

This is a practical way to add “memory” shared between runs and agents without introducing infra.

## What to use RAG for in coding agents

- recalling prior decisions, constraints, and architecture notes
- retrieving module summaries and API surfaces
- avoiding repeated expensive scans (dependency graphs, file inventories)

## SQLite schema (minimal)

Use a simple retrieval store:

- `documents(id, source, uri, title, text, updated_at)`
- `chunks(id, document_id, chunk_index, text, token_count)`
- `notes(id, run_id, scope, key, value_json, created_at)` for structured memory

Template SQL (copy into your project as needed):

- `assets/templates/sqlite/rag-schema.sql`

If you want vector search:

- store embeddings in `chunks(embedding_json)` and do approximate search in-app
- or use an external vector DB (only if scale requires it)

## Retrieval pattern

1. Ingest/update documents (docs, ADRs, key code entry points).
2. On each agent step, retrieve top-K relevant chunks for the current task.
3. Provide retrieved chunks as *context* (not instructions) and keep prompts explicit.

## Guardrails for memory

- Treat retrieved content as untrusted input (prompt injection risk).
- Keep a strict system/developer instruction layer that cannot be overridden by retrieved text.
- Require structured outputs for decisions that affect code.

## Relationship to personalization

RAG is a good fit for retrieving *documents and past decisions*.

If you want durable user personalization (preferences/constraints), prefer a small structured state + memory notes pipeline (`references/context-personalization.md`) and inject only the relevant slices into the model each run.
