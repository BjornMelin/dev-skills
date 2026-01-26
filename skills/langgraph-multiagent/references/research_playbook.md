# Research playbook (LangGraph/LangChain multi-agent)

Use this when you need *high confidence* on behavior, APIs, or migrations. The goal is to always use the **latest** docs *for the repo’s versions*.

## 1) Establish version ground truth

1. Read dependency constraints (preferred order):
   - `pyproject.toml`, `uv.lock`, `poetry.lock`, `requirements*.txt`, `pip-tools` files.
2. Confirm what is actually installed (runtime truth):
   - `python -c "from importlib import metadata; print(metadata.version('langgraph'))"` (repeat for `langchain`, `langchain-core`)

If installed versions don’t match constraints, treat all behavior as `UNVERIFIED` until reconciled.

## 2) Find the *right* docs pages (search, don’t guess)

Use `langchain-docs.SearchDocsByLangChain` as the first stop for:

- release notes + migrations
- “how-to” guides and patterns
- official examples

Technique:

1. Search by concept, not just API:
   - `create_agent middleware before_model after_model`
   - `subagents supervisor tool calling`
   - `interrupt checkpointer InMemorySaver`
   - `store long-term memory namespace search`
2. Open 3–5 top pages and extract the relevant section(s) with Exa crawling or `web.run`.

## 3) Lock down API references (Context7)

After you identify the concept/page, use Context7 for API-level details and code snippets:

- Resolve library IDs: `mcp__context7__resolve-library-id`
- Query docs with the exact API names you’re implementing:
  - “`create_agent` middleware `HumanInTheLoopMiddleware`”
  - “`StateGraph` `MessagesState` reducers `Send`”

## 4) When docs disagree or edge cases appear: use `opensrc/`

Use `opensrc/` for under-the-hood truth (read-only):

1. Snapshot exact versions:
   - `python scripts/opensrc_snapshot.py --packages langgraph langchain langchain-core` (from the skill folder)
2. Inspect:
   - `opensrc/sources.json` (source-of-truth for versions)
    - internal implementations for the relevant APIs
3. In writeups (ADRs/specs/PRs): cite exact `opensrc/...` paths + version strings.

## 5) Crawl strategy for “all relevant docs” without context bloat

Do **not** dump entire docs into the prompt. Instead:

1. Use `llms.txt` as sitemap for LangGraph:
   - `python scripts/fetch_llms_txt_urls.py --print --unique` (from the skill folder)
2. Filter URLs by task keywords (e.g. `multi_agent`, `checkpoint`, `store`, `interrupt`, `mcp`, `deployment`).
3. Crawl only those URLs; summarize into a short, version-tagged memo:
   - “What changed”, “Recommended pattern”, “Migration steps”, “Gotchas”.

## 6) Evidence checklist (what “research backed” means)

- Every non-trivial API usage is supported by either:
  - official docs snippet (LangChain docs + Context7), or
  - `opensrc/` inspection for the installed version
- For migrations: include exact “from → to” mappings and identify behavior changes (middleware, streaming, state schema).
- For production changes: include tests and observability hooks.
