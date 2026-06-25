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
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record --artifact .firecrawl/report.md --source-file ./report.pdf --command 'firecrawl parse "./report.pdf" -o .firecrawl/report.md' --intent parse
```

Choose one parse shape per artifact. For summary, targeted questions, or
multi-format output, use a matching `find --command ...` before upload and
record the same command with the artifact you created:

```bash
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --file "./report.pdf" --command 'firecrawl parse "./report.pdf" -S -o .firecrawl/report-summary.md' --intent parse --json
firecrawl parse "./report.pdf" -S -o .firecrawl/report-summary.md
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record --artifact .firecrawl/report-summary.md --source-file ./report.pdf --command 'firecrawl parse "./report.pdf" -S -o .firecrawl/report-summary.md' --intent parse

node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --file "./report.pdf" --command 'firecrawl parse "./report.pdf" -Q "What are the main conclusions?" -o .firecrawl/report-qa.md' --intent parse --json
firecrawl parse "./report.pdf" -Q "What are the main conclusions?" -o .firecrawl/report-qa.md
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record --artifact .firecrawl/report-qa.md --source-file ./report.pdf --command 'firecrawl parse "./report.pdf" -Q "What are the main conclusions?" -o .firecrawl/report-qa.md' --intent parse

node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --file "./report.pdf" --command 'firecrawl parse "./report.pdf" -f markdown,links --json --pretty -o .firecrawl/report.json' --intent parse --json
firecrawl parse "./report.pdf" -f markdown,links --json --pretty -o .firecrawl/report.json
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record --artifact .firecrawl/report.json --source-file ./report.pdf --command 'firecrawl parse "./report.pdf" -f markdown,links --json --pretty -o .firecrawl/report.json' --intent parse
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
