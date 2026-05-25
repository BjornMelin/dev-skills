# Scrape

Use `firecrawl scrape` when the URL is known. It handles static pages and many
JavaScript-rendered pages.

## Quick Start

```bash
mkdir -p .firecrawl
firecrawl scrape "https://example.com/page" -o .firecrawl/example-page.md
firecrawl scrape "https://example.com/page" --only-main-content -o .firecrawl/main.md
firecrawl scrape "https://example.com/page" --wait-for 3000 -o .firecrawl/rendered.md
firecrawl scrape "https://example.com/page" --format markdown,links -o .firecrawl/page.json
```

Multiple URLs can be passed positionally and are scraped concurrently:

```bash
firecrawl scrape "https://example.com" "https://example.com/blog"
```

## Key Flags

- `-f, --format <formats>`: `markdown`, `html`, `rawHtml`, `links`, `images`,
  `screenshot`, `summary`, `changeTracking`, `json`, `attributes`, `branding`.
- `-H, --html`: shortcut for `--format html`.
- `-S, --summary`: shortcut for `--format summary`.
- `-Q, --query <prompt>`: ask a question about the page. Use sparingly because
  saving and inspecting content is usually more reusable.
- `--schema` / `--schema-file`: structured extraction from a page.
- `--actions` / `--actions-file`: run scrape-time action arrays.
- `--profile <name>` and `--no-save-changes`: persistent browser state.
- `--lockdown`: lockdown mode.
- `--proxy <proxy>`: proxy mode such as `auto` or `basic`.
- `--max-age`, `--country`, `--languages`: cache and locale controls.

## Output Shape

Single-format output is usually raw content in the chosen format. Multiple
formats, `--schema`, or `--json` produce JSON. Probe the shape before assuming
field names:

```bash
jq 'keys' .firecrawl/page.json
jq -r '.markdown? // .data?.markdown? // empty' .firecrawl/page.json | head -80
jq -r '.. | objects | .metadata?.sourceURL? // .url? // empty' .firecrawl/page.json
jq -r '.. | objects | .links? // empty | if type == "array" then .[] else . end' .firecrawl/page.json
```

For schema extraction, read the schema result first, then fall back to markdown
or metadata only if the schema result is incomplete.

## Escalation

Use `interact` only if scrape cannot reach content without user-like actions.
Use `agent` when the extraction needs a structured schema across complex pages.
Use `crawl` when many related pages must be extracted.
