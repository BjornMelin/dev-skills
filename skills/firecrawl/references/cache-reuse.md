# Cache Reuse

Use this reference before paid Firecrawl commands when `.firecrawl/` may already
contain useful results. Local reuse is the only credit-saving cache layer:
Firecrawl `--max-age` can make paid scrapes faster and more reliable, but cached
server hits still bill credits.

## Workflow

1. Check for a fresh local artifact.
2. If a fresh exact hit exists, inspect the artifact and cite it.
3. If only stale hits exist, use them to target the smallest refresh.
4. If no hit exists, run the narrowest Firecrawl command and save output under
   `.firecrawl/`.
5. Re-scan or record the new artifact so future runs can reuse it.

Set the skill directory once, but run the command from the task repository so
`.firecrawl/` points at the current project. Override `FIRECRAWL_SKILL_DIR` if
the skill is installed somewhere else.

```bash
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" scan --root .firecrawl --out .firecrawl/index.jsonl
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --url "https://docs.example.com/page" --intent docs --json
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --query "firecrawl parse docs" --intent search --json
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" find --file "./report.pdf" --intent parse --json
```

When a new artifact needs explicit source metadata, append a record:

```bash
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record \
  --artifact .firecrawl/scrape-example-page.md \
  --url "https://example.com/page" \
  --command 'firecrawl scrape "https://example.com/page" -o .firecrawl/scrape-example-page.md' \
  --intent docs
```

For parsed local documents, record the source file so hash-based reuse works:

```bash
FIRECRAWL_SKILL_DIR="${FIRECRAWL_SKILL_DIR:-$HOME/.agents/skills/firecrawl}"
node "$FIRECRAWL_SKILL_DIR/scripts/firecrawl-cache-index.mjs" record \
  --artifact .firecrawl/parse-report.md \
  --source-file ./report.pdf \
  --intent parse
```

## Freshness Defaults

Use these defaults unless the user gives a stricter freshness requirement:

| Intent | Fresh window |
| --- | --- |
| `current`, `latest`, `news`, `pricing`, `status`, `changelog` | 1 hour |
| `search` result sets | 6 hours |
| `product`, `release`, `package` | 24 hours |
| `docs`, `reference`, `api` | 7 days |
| `parse` | Until the source file hash changes |
| `monitor` output | Historical evidence only |

For user wording such as "today", "latest", "current", "breaking", "price",
"status", or "changelog", prefer the 1-hour window. If the user explicitly asks
for fresh data, refresh even if the local cache is fresh.

## Refresh Policy

- Known URL stale hit: refresh that URL only.
- Search stale hit: reuse known good URLs where possible; rerun search only when
  source discovery itself matters.
- Crawl/download stale hit: map or scrape the needed pages before repeating the
  whole crawl/download.
- Parse stale hit: rerun only when the source file hash changed.
- Monitor hit: use only as historical context; call monitor APIs or scrape fresh
  for current claims.

For paid refreshes, pass an intent-matched `--max-age` instead of inventing
flags such as `--fresh`:

```bash
firecrawl scrape "https://docs.example.com/page" --max-age 604800000 -o .firecrawl/scrape-docs-page.md
firecrawl scrape "https://example.com/pricing" --max-age 3600000 -o .firecrawl/scrape-example-pricing.md
```

Use `--lockdown` only when the user wants Firecrawl's server-side cache-only
behavior for compliance or replay. Lockdown can miss, can return old cached
content, and is still billed.

## Index Privacy

`.firecrawl/index.jsonl` is metadata-only and must stay gitignored. It may store
URLs, query hashes, artifact paths, Firecrawl job IDs, file hashes, timestamps,
formats, and command text. It must not duplicate full scraped content.
