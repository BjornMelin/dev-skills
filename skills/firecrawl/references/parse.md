# Parse

Use `firecrawl parse` for local documents, not URLs. Supported released 1.18.x
file types: `.html`, `.htm`, `.pdf`, `.docx`, `.doc`, `.odt`, `.rtf`, `.xlsx`,
`.xls`. Max upload size is 50 MB.

Do not use this command for generic local source-code reads or repo editing.

## Quick Start

```bash
mkdir -p .firecrawl
firecrawl parse "./report.pdf" -o .firecrawl/report.md
firecrawl parse "./report.pdf" -S -o .firecrawl/report-summary.md
firecrawl parse "./report.pdf" -Q "What are the main conclusions?" -o .firecrawl/report-qa.md
firecrawl parse "./report.pdf" -f markdown,links --json --pretty -o .firecrawl/report.json
```

## Key Flags

- `-f, --format <formats>`: `markdown`, `html`, `rawHtml`, `links`, `images`,
  `summary`, `json`, `attributes`.
- `-H, --html`: shortcut for HTML.
- `-S, --summary`: summary output.
- `-Q, --query <prompt>`: targeted question.
- `--only-main-content`, `--include-tags`, `--exclude-tags`: content filtering.
- `--timeout <ms>`, `--timing`: execution controls.
- `--json`, `--pretty`, `-o`: structured output handling.

## Output Shape

Single-format parse output is raw content. Multiple formats or `--json` produce
JSON similar to scrape output:

```bash
jq 'keys' .firecrawl/report.json
jq -r '.markdown? // .data?.markdown? // empty' .firecrawl/report.json | head -80
jq -r '.. | objects | .links? // empty | if type == "array" then .[] else . end' .firecrawl/report.json
```

Inspect parsed files incrementally because documents can be large.
