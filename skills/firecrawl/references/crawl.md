# Crawl

Use `firecrawl crawl` when many pages from one site or section are needed.
Scope crawls tightly because credits are consumed per page.

## Quick Start

```bash
mkdir -p .firecrawl
firecrawl crawl "https://docs.example.com" --include-paths /docs --limit 50 --wait --pretty -o .firecrawl/crawl-docs.json
firecrawl crawl "https://docs.example.com" --max-depth 3 --wait --progress --pretty -o .firecrawl/crawl-depth3.json
```

Check or cancel a job:

```bash
firecrawl crawl <job-id> --status
firecrawl crawl <job-id> --cancel
```

## Key Flags

- `--wait`, `--poll-interval`, `--timeout`, `--progress`: async behavior.
- `--limit <number>` and `--max-depth <number>`: bound scope.
- `--include-paths <paths>` and `--exclude-paths <paths>`: path filters.
- `--sitemap <skip|include>` and `--ignore-query-parameters`: discovery shape.
- `--crawl-entire-domain`, `--allow-external-links`, `--allow-subdomains`:
  broaden scope only when explicitly needed.
- `--delay <ms>` and `--max-concurrency <number>`: crawl pacing.
- `--scrape-options` / `--scrape-options-file`: JSON scrape options for every
  crawled page.
- `--webhook <url-or-json>`: webhook config.

## Output Shape

Completed crawl output is JSON when saved with `--pretty` or JSON-oriented
options. Inspect root keys, then pull URLs and page text recursively because
async job and wait responses can differ:

```bash
jq 'keys' .firecrawl/crawl-docs.json
jq -r '.. | objects | .metadata?.sourceURL? // .url? // empty' .firecrawl/crawl-docs.json | head -100
jq -r '.. | objects | .markdown? // empty' .firecrawl/crawl-docs.json | head -120
```

Use page URLs from the crawl output as evidence anchors in summaries. If a crawl
returns a job ID instead of content, run `firecrawl crawl <job-id> --status`
and wait or fetch results according to local help.

## Scope Rule

Prefer `map --search` plus targeted scrapes when the user needs specific pages.
Use crawl when breadth is the requirement.
