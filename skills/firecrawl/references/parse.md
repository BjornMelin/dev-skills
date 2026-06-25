# Parse

Use `firecrawl parse` for local documents, not URLs. Supported released 1.19.x
file types: `.html`, `.htm`, `.pdf`, `.docx`, `.doc`, `.odt`, `.rtf`, `.xlsx`,
`.xls`. Max upload size is 50 MB.

Do not use this command for generic local source-code reads or repo editing.

## Quick Start

```bash
mkdir -p .firecrawl
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --file "./report.pdf" --command 'firecrawl parse "./report.pdf" -o .firecrawl/report.md' --intent parse --json
firecrawl parse "./report.pdf" -o .firecrawl/report.md
firecrawl parse "./report.pdf" -S -o .firecrawl/report-summary.md
firecrawl parse "./report.pdf" -Q "What are the main conclusions?" -o .firecrawl/report-qa.md
firecrawl parse "./report.pdf" -f markdown,links --json --pretty -o .firecrawl/report.json
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record --artifact .firecrawl/report.md --source-file ./report.pdf --command 'firecrawl parse "./report.pdf" -o .firecrawl/report.md' --intent parse
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

## Reuse

Parsed local documents are reusable when the source file hash and parse command
metadata match. After a parse, record the source file and exact command with
`firecrawl-cache-index.mjs record` so future `find --file --command ...` checks
can skip duplicate uploads without reusing a summary or targeted query for a
different parse request.
