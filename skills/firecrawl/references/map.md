# Map

Use `firecrawl map` to discover URLs on a known site, especially when the user
knows the domain but not the exact page.

## Quick Start

```bash
mkdir -p .firecrawl
firecrawl map "https://docs.example.com" --search "authentication" -o .firecrawl/docs-auth-urls.txt
firecrawl map "https://docs.example.com" --limit 500 --json --pretty -o .firecrawl/docs-urls.json
```

## Key Flags

- `--wait`: wait for map completion.
- `--limit <number>`: maximum URLs to discover.
- `--search <query>`: filter URLs by search query.
- `--sitemap <only|include|skip>`: sitemap strategy.
- `--include-subdomains`: include subdomains.
- `--ignore-query-parameters`: normalize away query params.
- `--timeout <seconds>`: map timeout.

## Output Shape

Map output has changed across CLI/API versions, so extract URLs defensively
instead of assuming one array path:

```bash
jq 'keys' .firecrawl/docs-urls.json
jq -r '.. | strings | select(test("^https?://"))' .firecrawl/docs-urls.json | head -100
```

For text output, treat each non-empty line as a candidate URL and validate
before scraping:

```bash
sed '/^[[:space:]]*$/d' .firecrawl/docs-auth-urls.txt | head -100
```

## Pattern

Map plus scrape is usually cheaper and cleaner than crawling when only one or a
few pages are needed:

```bash
firecrawl map "https://docs.example.com" --search "webhooks" --json -o .firecrawl/map-webhooks.json
jq -r '.. | strings | select(test("^https?://"))' .firecrawl/map-webhooks.json | head
firecrawl scrape "https://docs.example.com/path/from-map" -o .firecrawl/webhooks.md
```
