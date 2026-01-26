# Retrieval + Agentic RAG

## Baseline

Start with a retrieval tool and let the agent decide when to call it (agentic RAG). Keep tool outputs structured:

- “content” for model consumption
- “artifacts” for your application (raw docs/metadata)

## Advanced controls

- Query rewriting and relevance checks as separate nodes.
- Reranking with timeouts (fail open).
- Source attribution: keep doc IDs/URLs outside the LLM context when possible, and attach them as artifacts.

## Docs-driven RAG for LangGraph

Use `llms.txt` as the doc URL index, and implement an allowlist-based fetch tool that only loads those URLs.

