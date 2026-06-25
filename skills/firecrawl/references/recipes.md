# Recipes

Use these when the inline `SKILL.md` recipes are too small. Keep outputs under
`.firecrawl/`, inspect incrementally, quote URLs and paths, and leave a small
evidence file for claims the final answer depends on.

## Local Reuse Check

Run the bundled cache helper before paid commands when prior `.firecrawl`
artifacts may answer the task:

```bash
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --url "https://docs.example.com/page" --intent docs --json
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --query "official react useTransition docs" --intent search --json
```

Reuse fresh exact hits. Use stale hits to target the smallest refresh.

## Search, Scrape, Feedback

Use when no URL is known and you need result content.

```bash
mkdir -p .firecrawl
firecrawl search "official react useTransition docs" \
  --scrape \
  --limit 5 \
  --json \
  -o .firecrawl/search-react-use-transition.json

jq -r '.data.web[] | "\(.title): \(.url)"' \
  .firecrawl/search-react-use-transition.json

SEARCH_ID=$(jq -r '.id' .firecrawl/search-react-use-transition.json)
firecrawl search-feedback "$SEARCH_ID" \
  --rating good \
  --valuable-sources '[{"url":"https://react.dev/reference/react/useTransition","reason":"Official React API reference"}]' \
  --missing-content '[{"topic":"routing examples","description":"Wanted examples for route transitions"}]' \
  --silent &
```

Use `partial` or `bad` when appropriate. If
`FIRECRAWL_NO_SEARCH_FEEDBACK=1` or `FIRECRAWL_DISABLE_SEARCH_FEEDBACK=1` is
set, do not try to work around it.

## Map To Scrape

Use when the site is known but the exact page is not.

```bash
firecrawl map "https://docs.example.com" \
  --search "webhook signature verification" \
  --json \
  --pretty \
  -o .firecrawl/map-webhook-signatures.json

jq -r '.. | strings | select(test("^https?://"))' \
  .firecrawl/map-webhook-signatures.json | head

firecrawl scrape "https://docs.example.com/webhooks/signatures" \
  --only-main-content \
  -o .firecrawl/webhook-signatures.md
```

## Scoped Docs Crawl

Use when many pages under a section are needed.

```bash
firecrawl crawl "https://docs.example.com" \
  --include-paths /docs \
  --exclude-paths "/docs/archive,/docs/translations" \
  --limit 100 \
  --max-depth 3 \
  --wait \
  --progress \
  --pretty \
  -o .firecrawl/crawl-docs.json
```

Useful inspection:

```bash
jq 'keys' .firecrawl/crawl-docs.json
jq -r '.. | objects | .metadata?.sourceURL? // empty' .firecrawl/crawl-docs.json | head
```

## Structured Scrape With Schema

Use for one known page when a schema is enough and `agent` would be overkill.
For reusable templates, see `references/schemas.md`.

```bash
cat > .firecrawl/pricing-schema.json <<'JSON'
{
  "type": "object",
  "properties": {
    "plans": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "price": { "type": "string" },
          "features": { "type": "array", "items": { "type": "string" } }
        },
        "required": ["name"]
      }
    }
  }
}
JSON

firecrawl scrape "https://example.com/pricing" \
  --schema-file .firecrawl/pricing-schema.json \
  --json \
  --pretty \
  -o .firecrawl/pricing.json
```

## Evidence Closeout

Use after search, scrape, crawl, parse, or agent output supports a final answer.

```bash
{
  printf 'Source: %s\n' 'https://example.com/pricing'
  printf 'Command: %s\n' 'firecrawl scrape "https://example.com/pricing" --only-main-content -o .firecrawl/scrape-example-pricing.md'
  printf 'Artifact: %s\n' '.firecrawl/scrape-example-pricing.md'
  printf '\nKey lines:\n'
  rg -n "price|plan|limit|included" .firecrawl/scrape-example-pricing.md
} > .firecrawl/scrape-example-pricing.evidence.txt
```

In final answers, include the source URL and local artifact path when it helps
the user verify or rerun the work.

## Agent With Schema And Credit Cap

Use when data spans complex pages or navigation and a simple scrape is not
enough.

```bash
firecrawl agent "Extract pricing tiers and limits from this product site" \
  --urls "https://example.com/pricing,https://example.com/docs/limits" \
  --schema-file .firecrawl/pricing-schema.json \
  --max-credits 25 \
  --wait \
  --json \
  --pretty \
  -o .firecrawl/agent-pricing.json
```

## Scrape To Interact With Profile

Use for content that requires clicks, toggles, login state, pagination, or
infinite scroll.

```bash
firecrawl scrape "https://app.example.com/login" --profile app-example
firecrawl interact "Fill in the email field with the test account and click continue"
firecrawl interact "Extract the account table as JSON"
firecrawl interact stop
```

Read-only reconnect:

```bash
firecrawl scrape "https://app.example.com/dashboard" \
  --profile app-example \
  --no-save-changes
firecrawl interact "Summarize the dashboard notifications"
firecrawl interact stop
```

Do not put secrets in prompts. Stop if login requires credentials or approvals
the user has not explicitly provided.

## Monitor JSON Change Tracking

Use JSON payloads for field-level diffs such as prices, inventory, headlines,
or items in a list.

```bash
cat > .firecrawl/pricing-monitor.json <<'JSON'
{
  "name": "Pricing watch",
  "schedule": { "text": "hourly", "timezone": "UTC" },
  "targets": [
    {
      "type": "scrape",
      "urls": ["https://example.com/pricing"],
      "scrapeOptions": {
        "formats": [
          {
            "type": "changeTracking",
            "modes": ["json", "git-diff"],
            "prompt": "Extract pricing tiers and headline features for each plan.",
            "schema": {
              "type": "object",
              "properties": {
                "plans": {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "name": { "type": "string" },
                      "price": { "type": "string" },
                      "features": { "type": "array", "items": { "type": "string" } }
                    }
                  }
                }
              }
            }
          }
        ]
      }
    }
  ]
}
JSON

firecrawl monitor create .firecrawl/pricing-monitor.json
firecrawl monitor checks <monitorId> --limit 10
firecrawl monitor check <monitorId> <checkId> --page-status changed
```

Use `--state paused` to pause. Use `--page-status`, not `--status`, to filter
page results.

## Parse Local Document

Use only for local documents, not generic source files.

```bash
firecrawl parse "./contract.pdf" -o .firecrawl/contract.md
rg -n "termination|renewal|liability" .firecrawl/contract.md

firecrawl parse "./financials.xlsx" \
  -Q "What are the top three expense categories?" \
  -o .firecrawl/financials-qa.md
```

## X Download

Use for an offline local site copy. In released 1.19.x the command lives under
`x`; the expanded `experimental download` spelling is an alias.

```bash
firecrawl x download "https://docs.example.com" \
  --include-paths "/docs,/reference" \
  --exclude-paths "/zh,/ja,/fr,/archive" \
  --format markdown,links \
  --limit 100 \
  --only-main-content \
  -y
```

Do not use top-level `firecrawl download` unless local `firecrawl --help`
shows it.
