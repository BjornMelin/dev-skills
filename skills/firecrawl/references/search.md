# Search

Use `firecrawl search` when the user does not already have the exact URL.

## Quick Start

```bash
mkdir -p .firecrawl
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --query "query" --intent search --json
firecrawl search "query" --json -o .firecrawl/search-query.json
firecrawl search "query" --scrape --json -o .firecrawl/search-query-scraped.json
firecrawl search "query" --sources news --tbs qdr:d --json -o .firecrawl/news.json
```

## Key Flags

- `--limit <number>`: result count, default 5, max 100.
- `--sources <web,images,news>`: source types.
- `--categories <github,research,pdf>`: category filters.
- `--tbs <qdr:h|qdr:d|qdr:w|qdr:m|qdr:y>`: recency.
- `--location <location>` and `--country <code>`: geo targeting.
- `--scrape`: scrape result pages immediately.
- `--scrape-formats <formats>`: formats for scraped result pages.
- `--ignore-invalid-urls`: exclude URLs not valid for other Firecrawl endpoints.
- `--timeout <ms>`: search timeout.

## Output Shape

With `--json`, inspect root keys first because source-specific arrays vary by
query options. Released 1.19.x search responses commonly include a search
`.id` and source arrays under `.data`.

```bash
jq 'keys' .firecrawl/search-query.json
jq -r '.id // empty' .firecrawl/search-query.json
jq -r '.data.web[]?.url' .firecrawl/search-query.json
jq -r '.data.web[]? | "\(.title): \(.url)"' .firecrawl/search-query.json
jq -r '.data.news[]? | "\(.title): \(.url)"' .firecrawl/search-query.json
jq -r '.. | objects | .markdown? // empty' .firecrawl/search-query-scraped.json | head -80
```

`search --scrape` already fetches full result content. Do not re-scrape those
URLs unless the embedded scraped content is incomplete for the task.

## Reuse

Search results are usually fresh for 6 hours. Use a 1-hour window when the
query says "latest", "today", "current", "news", "pricing", "status", or
"changelog". If a stale search artifact contains the right URLs, scrape those
known URLs before repeating search.

## Feedback

After using a search result, send feedback once per search ID unless
`FIRECRAWL_NO_SEARCH_FEEDBACK=1` or `FIRECRAWL_DISABLE_SEARCH_FEEDBACK=1` is
set. Use `--silent &` so feedback never blocks the task.

```bash
SEARCH_ID=$(jq -r '.id' .firecrawl/search-query.json)
firecrawl search-feedback "$SEARCH_ID" \
  --rating good \
  --valuable-sources '[{"url":"https://example.com","reason":"Authoritative"}]' \
  --missing-content '[{"topic":"missing official changelog"}]' \
  --silent &
```

Use `good`, `partial`, or `bad` based on actual usefulness. Make
`--missing-content` specific, one topic per entry.

For non-search endpoint jobs, use generic feedback:

```bash
firecrawl feedback scrape "$SCRAPE_ID" --rating partial --issues missing_content --note "Main pricing table was absent" --silent &
```
