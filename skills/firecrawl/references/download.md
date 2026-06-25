# X Download

Use `firecrawl x download` to save a site as nested local files under
`.firecrawl/`. In released 1.19.x it aliases the expanded
`firecrawl experimental download` form. The command maps first, then scrapes
pages.

## Quick Start

```bash
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --url "https://docs.example.com" --intent docs --json
firecrawl x download "https://docs.example.com" --format markdown,links --screenshot --limit 20 -y
firecrawl x download "https://docs.example.com" \
  --include-paths "/features,/sdks" \
  --exclude-paths "/zh,/ja,/fr,/es" \
  --only-main-content \
  -y
```

Always use `-y` / `--yes` in automated agent runs to avoid confirmation prompts.

## Output Shape

The command writes a directory tree rather than one primary stdout payload.
Expect page files beneath `.firecrawl/` using the requested formats.

```bash
find .firecrawl -maxdepth 3 -type f | sort | head -50
rg -n "title|pricing|authentication" .firecrawl
```

Use `--format markdown,links` for agent-friendly text plus discovered links.
Add `json` only when downstream tooling expects per-page structured files.

## Key Flags

- `--limit <number>`: max pages.
- `--search <query>`: filter mapped pages.
- `--include-paths <paths>` and `--exclude-paths <paths>`: scope.
- `--allow-subdomains`: include subdomains.
- `-f, --format <formats>`: `markdown`, `html`, `rawHtml`, `links`, `images`,
  `summary`, `json`.
- `--screenshot`, `--full-page-screenshot`: visual capture.
- `--wait-for`, `--max-age`, `--country`, `--languages`, `--lockdown`: scrape
  options.

If a prior offline copy is stale, inspect and scrape only the needed pages
before repeating a full download.

Do not use top-level `firecrawl download` unless `firecrawl --help` shows it.
