# Firecrawl Command Selection

Use this decision tree before reading a command-specific reference.

## Reuse First

Before paid commands, check local `.firecrawl/` artifacts:

```bash
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --url "https://example.com/page" --intent docs --json
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --query "query text" --intent search --json
```

Reuse fresh exact hits. Use stale hits to target the smallest refresh. Read
[cache-reuse.md](cache-reuse.md) for TTLs, parse file hashes, and `--max-age`
defaults.

## Route By User Intent

| User intent | Default command | Load |
| --- | --- | --- |
| Find sources, articles, current info, or pages | `search` | `references/search.md` |
| Read/extract a known URL | `scrape` | `references/scrape.md` |
| Find a URL on a known site | `map` | `references/map.md` |
| Extract many pages under one site or section | `crawl` | `references/crawl.md` |
| Track future changes | `monitor` | `references/monitor.md` |
| Extract structured data from complex sites | `agent` | `references/agent.md` |
| Click, fill, login, paginate, or inspect a live session | `scrape` then `interact` | `references/interact.md` |
| Parse a local PDF/Office/HTML document | `parse` | `references/parse.md` |
| Save a local offline site copy | `x download` | `references/download.md` |
| Research arXiv papers or GitHub issue/PR history through Firecrawl | `research` | `references/research.md` |
| Diagnose CLI/account/job failure | `doctor` | `references/maintenance.md` |
| Send endpoint quality feedback | `feedback` or `search-feedback` | `references/search.md` |

## Escalation

1. Search if no URL is known.
2. Scrape known URLs before using heavier tools.
3. Map a site to locate a specific page before crawling.
4. Crawl only when the task needs many pages.
5. Use monitor for repeated future checks instead of repeated one-off scrapes.
6. Use agent for structured extraction when normal scrape/crawl would require
   substantial navigation or synthesis.
7. Use interact only after a scrape, and only when interaction is required.

## Scope Discipline

- Default to `--limit` and narrow path filters.
- Prefer `map --search` plus targeted `scrape` before `crawl`.
- Require a schema and `--max-credits` for `agent` unless the user explicitly
  wants exploratory extraction.
- Do not use whole-domain, subdomain, or external-link crawl flags unless the
  user asks for that breadth.

## Gotchas

- `firecrawl x download` is the preferred released 1.19.x site-download path.
  It aliases the expanded `firecrawl experimental download` form. Do not use
  top-level `firecrawl download` unless `firecrawl --help` shows it.
- `parse` is for local documents, not URLs.
- `search --scrape` already fetches result page content.
- `interact` needs a prior scrape ID, either implicit from the last scrape or
  explicit with `--scrape-id`.
- `firecrawl init`, `firecrawl setup skills`, `firecrawl setup mcp`,
  `firecrawl launch`, and `firecrawl make default` can modify workstation
  config or installed skills. Do not run them from this skill unless the user
  explicitly asks for Firecrawl maintenance.
- `firecrawl make default` can change native web-provider defaults; use
  `firecrawl make default --undo` only as an intentional maintenance action.
