# Schema Templates

Use these templates with `scrape --schema-file`, `agent --schema-file`, or
monitor `changeTracking` schemas. Save selected templates under
`.firecrawl/schema-<purpose>.json` and keep `evidenceUrl` fields when the output
will support user-facing claims.

## Pricing

```json
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
          "billingPeriod": { "type": "string" },
          "limits": { "type": "array", "items": { "type": "string" } },
          "features": { "type": "array", "items": { "type": "string" } },
          "evidenceUrl": { "type": "string" }
        },
        "required": ["name", "evidenceUrl"]
      }
    }
  },
  "required": ["plans"]
}
```

## Changelog

```json
{
  "type": "object",
  "properties": {
    "entries": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "title": { "type": "string" },
          "date": { "type": "string" },
          "summary": { "type": "string" },
          "impact": { "type": "string" },
          "evidenceUrl": { "type": "string" }
        },
        "required": ["title", "evidenceUrl"]
      }
    }
  },
  "required": ["entries"]
}
```

## Docs API Reference

```json
{
  "type": "object",
  "properties": {
    "endpoints": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "method": { "type": "string" },
          "path": { "type": "string" },
          "description": { "type": "string" },
          "requiredParameters": {
            "type": "array",
            "items": { "type": "string" }
          },
          "evidenceUrl": { "type": "string" }
        },
        "required": ["path", "evidenceUrl"]
      }
    }
  },
  "required": ["endpoints"]
}
```

## Products

```json
{
  "type": "object",
  "properties": {
    "products": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "category": { "type": "string" },
          "description": { "type": "string" },
          "price": { "type": "string" },
          "availability": { "type": "string" },
          "evidenceUrl": { "type": "string" }
        },
        "required": ["name", "evidenceUrl"]
      }
    }
  },
  "required": ["products"]
}
```

## Company Contacts

```json
{
  "type": "object",
  "properties": {
    "company": { "type": "string" },
    "contacts": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "role": { "type": "string" },
          "email": { "type": "string" },
          "profileUrl": { "type": "string" },
          "evidenceUrl": { "type": "string" }
        },
        "required": ["name", "evidenceUrl"]
      }
    }
  },
  "required": ["contacts"]
}
```

## Research Claims

```json
{
  "type": "object",
  "properties": {
    "claims": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "claim": { "type": "string" },
          "value": { "type": "string" },
          "date": { "type": "string" },
          "sourceTitle": { "type": "string" },
          "evidenceUrl": { "type": "string" },
          "confidence": { "type": "string" }
        },
        "required": ["claim", "evidenceUrl"]
      }
    }
  },
  "required": ["claims"]
}
```

## Usage

```bash
firecrawl scrape "https://example.com/pricing" \
  --schema-file .firecrawl/schema-pricing.json \
  --json \
  --pretty \
  -o .firecrawl/scrape-example-pricing.json

firecrawl agent "Extract product pricing with evidence URLs" \
  --urls "https://example.com/pricing" \
  --schema-file .firecrawl/schema-pricing.json \
  --max-credits 25 \
  --wait \
  --json \
  --pretty \
  -o .firecrawl/agent-pricing.json
```

Validate outputs before relying on them:

```bash
jq 'keys' .firecrawl/agent-pricing.json
jq -r '.. | objects | .evidenceUrl? // empty' .firecrawl/agent-pricing.json | sort -u
```
