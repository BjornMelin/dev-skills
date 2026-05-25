# Firecrawl Command Selection

Use this decision tree before reading a command-specific reference.

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

- `firecrawl x download` is the preferred released 1.18.x site-download path.
  It aliases the expanded `firecrawl experimental download` form. Do not use
  top-level `firecrawl download` unless `firecrawl --help` shows it.
- `parse` is for local documents, not URLs.
- `search --scrape` already fetches result page content.
- `interact` needs a prior scrape ID, either implicit from the last scrape or
  explicit with `--scrape-id`.
- `firecrawl init` and `firecrawl setup skills` can modify installed skills in
  this environment. Do not run them from this skill.
