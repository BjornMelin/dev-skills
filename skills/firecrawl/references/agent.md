# Agent

Use `firecrawl agent` for AI-powered structured extraction from complex sites.
It is heavier than scrape/crawl, so cap credits and prefer schemas.
For reusable schema templates, read `references/schemas.md`.

## Quick Start

```bash
mkdir -p .firecrawl
firecrawl agent "extract all pricing tiers" --wait --max-credits 25 --json --pretty -o .firecrawl/pricing.json
firecrawl agent "extract products" --urls "https://example.com/products" --schema-file schema.json --wait --json --pretty -o .firecrawl/products.json
```

## Key Flags

- `--urls <urls>`: comma-separated starting URLs.
- `--model <model>`: `spark-1-mini` default or `spark-1-pro`.
- `--schema <json>` / `--schema-file <path>`: structured output contract.
- `--max-credits <number>`: spending cap.
- `--wait`, `--poll-interval`, `--timeout`: completion behavior.
- `--status` and `--cancel`: job management.
- `--webhook <url-or-json>`: completion webhook.

## Output Shape

Agent output depends on whether a schema was provided. Inspect the root, then
prefer schema-constrained data over free-form text:

```bash
jq 'keys' .firecrawl/agent-pricing.json
jq '.data // .result // .' .firecrawl/agent-pricing.json
jq -r '.. | strings | select(test("^https?://"))' .firecrawl/agent-pricing.json | sort -u
```

Require evidence URLs in requested schemas when claims matter:

```json
{
  "type": "object",
  "properties": {
    "items": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "claim": { "type": "string" },
          "value": { "type": "string" },
          "evidenceUrl": { "type": "string" }
        },
        "required": ["claim", "evidenceUrl"]
      }
    }
  }
}
```

## Use Agent When

- The user asks for structured JSON from a complex website.
- The data spans several pages and navigation is non-obvious.
- A schema or field list is available.

For simple single-page extraction, use `scrape` with `--schema` or save markdown
and inspect it yourself.
