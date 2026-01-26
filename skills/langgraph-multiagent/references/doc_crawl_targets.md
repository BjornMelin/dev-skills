# Doc crawl targets (seeds)

This file exists to make “crawl everything relevant” repeatable without guessing where to start.

## LangGraph sitemap

- `https://langchain-ai.github.io/langgraph/llms.txt`
- (optional, larger) `https://langchain-ai.github.io/langgraph/llms-full.txt`

## LangChain OSS Python (high-signal entrypoints)

- `https://docs.langchain.com/oss/python/langchain/multi-agent`
- `https://docs.langchain.com/oss/python/langchain/multi-agent/subagents-personal-assistant`
- `https://docs.langchain.com/oss/python/langchain/guardrails`
- `https://docs.langchain.com/oss/python/langchain/runtime`
- `https://docs.langchain.com/oss/python/langchain/context-engineering`
- `https://docs.langchain.com/oss/python/langchain/mcp`
- `https://docs.langchain.com/oss/python/langchain/human-in-the-loop`
- `https://docs.langchain.com/oss/python/langchain/retrieval`
- `https://docs.langchain.com/oss/python/langchain/long-term-memory`

## LangChain OSS JavaScript (UI + streaming)

- `https://docs.langchain.com/oss/javascript/langchain/streaming/frontend`
- `https://docs.langchain.com/oss/javascript/langchain/ui`
- `https://docs.langchain.com/oss/javascript/langchain/human-in-the-loop`

## LangGraph OSS Python (high-signal entrypoints)

- `https://docs.langchain.com/oss/python/langgraph/agentic-rag`
- `https://docs.langchain.com/oss/python/langgraph/sql-agent`
- `https://docs.langchain.com/oss/python/langgraph/workflows-agents`
- `https://docs.langchain.com/oss/python/langgraph/thinking-in-langgraph`

## LangSmith / Agent Server (UI + API)

- `https://docs.langchain.com/langsmith/agent-server`
- `https://docs.langchain.com/langsmith/server-api-ref`
- `https://docs.langchain.com/langsmith/configurable-headers`
- `https://docs.langchain.com/langsmith/configurable-logs`
- `https://docs.langchain.com/langsmith/agent-server-scale`
- `https://docs.langchain.com/langsmith/generative-ui-react`
- `https://docs.langchain.com/langsmith/agent-server-api/thread-runs/create-run-stream-output`
- `https://docs.langchain.com/langsmith/agent-server-api/thread-runs/join-run-stream`
- `https://docs.langchain.com/langsmith/agent-server-api/threads/join-thread-stream`

## How to crawl (bounded)

Use the deterministic crawler:

- `python scripts/crawl_docs.py --llms-txt https://langchain-ai.github.io/langgraph/llms.txt --allow-prefixes https://langchain-ai.github.io/langgraph/ --max-pages 500 --out-dir docs_cache_langgraph` (from the skill folder)
- For LangChain pages, pass as explicit seeds and constrain prefixes:
  - `python scripts/crawl_docs.py --seeds https://docs.langchain.com/oss/python/langchain/multi-agent https://docs.langchain.com/oss/python/langchain/guardrails --allow-prefixes https://docs.langchain.com/oss/python/langchain/ --max-pages 300 --out-dir docs_cache_langchain` (from the skill folder)

If you need richer extraction (markdown/plaintext), prefer MCP tools (`langchain-docs.SearchDocsByLangChain` + Exa crawling) over raw HTML crawling.
