---
name: firecrawl
description: |
  Use this skill for Firecrawl CLI web work: web search, URL scraping, site mapping, crawling, structured extraction, page interaction, monitoring changes, offline site download via x download, and parsing local documents such as PDF, DOCX, XLSX, HTML, DOC, ODT, or RTF. Trigger for requests to search the web, look up current info, fetch/read/scrape a URL, extract website data, crawl docs, click/fill/login/paginate a page, monitor page changes, save a site offline, or parse a document. Do not trigger for generic local file reads/edits, git/deploy/code tasks, or Firecrawl app integration work.
license: ISC
allowed-tools:
  - Bash(firecrawl *)
  - Bash(npx firecrawl *)
---

# Firecrawl CLI

Use the Firecrawl CLI for live web search, URL extraction, site discovery,
bulk crawls, browser-backed interaction, recurring monitors, offline site
download, and local document parsing.

Use the installed CLI as command truth. Run `firecrawl --help` or
`firecrawl <command> --help` before relying on version-sensitive flags. This
skill is written for released `firecrawl-cli` 1.18.x behavior; do not teach or
use unreleased GitHub-main flags unless local help confirms them.

Do not run `firecrawl init`, `firecrawl setup skills`, or any Firecrawl
skill-install command from this skill. Those commands are user-manual only in
this environment so custom skills are not overwritten.

## First Checks

1. Check setup with `firecrawl --status`.
2. If a command or flag matters, confirm it with `firecrawl <command> --help`.
3. Write large outputs to `.firecrawl/` with `-o`; do not stream large page
   content into the agent context.
4. Quote URLs and paths. Shells treat `?`, `&`, spaces, and brackets specially.
5. Use deterministic artifact names so follow-up commands can find evidence.
6. Do not send private, confidential, repo-proprietary, or secret-bearing
   material to Firecrawl unless the user explicitly permits external
   processing.

For setup/auth troubleshooting, read [references/install-auth.md](references/install-auth.md).
For output safety, read [references/output-security.md](references/output-security.md).
For local drift checks, run [scripts/firecrawl-doctor.mjs](scripts/firecrawl-doctor.mjs).

## Command Selection

Read [references/command-selection.md](references/command-selection.md) for the
full decision tree. Default order:

1. `search` when no exact URL is known.
2. `scrape` when a URL is known.
3. `map` when a site is known but the exact page is not.
4. `crawl` when many pages from a site/section are needed.
5. `monitor` when the user needs ongoing change tracking.
6. `agent` when the user wants structured data from complex sites and provides
   a schema, or schema-like target fields.
7. `interact` only after `scrape` when content requires clicks, forms, login,
   pagination, session state, or browser actions.
8. `parse` for local documents, not URLs.
9. `x download` when the user wants a local offline site copy.

## Scope And Cost Defaults

- Use `--limit` on `search`, `map`, `crawl`, `agent`, and `x download`.
- Prefer `map --search` plus targeted `scrape` before broad crawls.
- Keep `crawl` scoped with `--include-paths`, `--exclude-paths`, `--max-depth`,
  and `--wait`.
- Use `agent --max-credits` and a schema for complex structured extraction.
- Do not crawl whole domains, allow external links, or allow subdomains unless
  the user explicitly needs that breadth.

## Default Recipes

Prefer these short chains before opening a detailed reference. Read
[references/recipes.md](references/recipes.md) for schema, monitor JSON, jq,
output-shape probes, profile, feedback, and download variants.

Search with page content, inspect, then send feedback:

```bash
firecrawl search "query" --scrape --json -o .firecrawl/search-query.json
jq -r '.data.web[] | "\(.title): \(.url)"' .firecrawl/search-query.json
firecrawl search-feedback "$(jq -r '.id' .firecrawl/search-query.json)" --rating good --valuable-sources '[{"url":"https://example.com","reason":"Useful result"}]' --silent &
```

Find a page on a known site, then scrape it:

```bash
firecrawl map "https://docs.example.com" --search "authentication" --json -o .firecrawl/map-auth.json
firecrawl scrape "https://docs.example.com/auth-page" -o .firecrawl/auth-page.md
```

Scoped docs crawl:

```bash
firecrawl crawl "https://docs.example.com" --include-paths /docs --limit 50 --wait --pretty -o .firecrawl/crawl-docs.json
```

Scrape, interact, then stop:

```bash
firecrawl scrape "https://example.com" --profile example-site
firecrawl interact "Click the pricing tab and extract the visible plans"
firecrawl interact stop
```

Parse a local document:

```bash
firecrawl parse "./report.pdf" -o .firecrawl/report.md
```

Offline docs copy:

```bash
firecrawl x download "https://docs.example.com" --include-paths /docs --format markdown,links --limit 50 -y
```

Basic recurring monitor:

```bash
firecrawl monitor create --name "Changelog" --schedule "every 30 minutes" --scrape-urls "https://example.com/changelog"
```

## Artifact Names

Create `.firecrawl/` first and use names that encode command plus subject:

```text
.firecrawl/search-<slug>.json
.firecrawl/search-<slug>-scraped.json
.firecrawl/map-<site>-<topic>.json
.firecrawl/scrape-<site>-<page>.md
.firecrawl/scrape-<site>-<page>.json
.firecrawl/crawl-<site>-<scope>.json
.firecrawl/agent-<task>.json
.firecrawl/monitor-<name>.json
.firecrawl/parse-<document>.md
.firecrawl/schema-<purpose>.json
```

## Evidence Closeout

Before finalizing web-data work, preserve enough evidence to audit the answer:

```bash
printf 'Command: %s\nArtifact: %s\n' \
  'firecrawl scrape "https://example.com/page" -o .firecrawl/scrape-example-page.md' \
  '.firecrawl/scrape-example-page.md' \
  > .firecrawl/scrape-example-page.evidence.txt
rg -n "pricing|limit|changed|released" .firecrawl/scrape-example-page.md \
  >> .firecrawl/scrape-example-page.evidence.txt
```

Final answers should cite source URLs and local artifact paths when Firecrawl
evidence materially supports the claim.

## Failure Recovery

- Auth/401: run `firecrawl --status`; see `references/install-auth.md`.
- Credits/402 or rate limit: reduce `--limit`, narrow scope, or stop and report.
- Timeout: add `--timeout`, reduce scope, or use `--wait-for` for rendering.
- Blocked or JS-heavy page: retry `scrape` with `--wait-for`; escalate to
  `interact` only after a successful scrape.
- Monitor unavailable: report the account/retention limitation and use one-off
  scrape plus local diff instead.
- Missing interact session: run `scrape` first and pass `--profile` or
  `--scrape-id`.
- Malformed JSON: save raw output, validate with `jq`, and rerun with `--json`
  or `--pretty`.
- CLI help differs from this skill: trust local `firecrawl <command> --help`
  and update the skill later.

## Reference Loading

- Rich command chains: [references/recipes.md](references/recipes.md)
- Reusable schemas: [references/schemas.md](references/schemas.md)
- Search/discovery first: [references/search.md](references/search.md)
- Known URL extraction: [references/scrape.md](references/scrape.md)
- Site URL discovery: [references/map.md](references/map.md)
- Bulk site extraction: [references/crawl.md](references/crawl.md)
- AI structured extraction: [references/agent.md](references/agent.md)
- Browser interaction: [references/interact.md](references/interact.md)
- Local document parsing: [references/parse.md](references/parse.md)
- Recurring change tracking: [references/monitor.md](references/monitor.md)
- Offline site copy: [references/download.md](references/download.md)
- Local maintenance/drift: [references/maintenance.md](references/maintenance.md)

For Firecrawl SDK/API integration into an application, adding
`FIRECRAWL_API_KEY` to a project, or choosing product endpoints, do not use
this CLI skill as the implementation authority. Use the Firecrawl build skills
if installed. For outcome deliverables such as research briefs, SEO audits,
lead lists, QA reports, or design extraction, use the dedicated Firecrawl
workflow skills if installed.

## Output Defaults

Use `.firecrawl/` for fetched or parsed output unless the user explicitly wants
inline content:

```bash
mkdir -p .firecrawl
firecrawl search "query" --json -o .firecrawl/search-query.json
firecrawl scrape "https://example.com/page" -o .firecrawl/example-page.md
```

Inspect output incrementally:

```bash
wc -l .firecrawl/example-page.md
head -80 .firecrawl/example-page.md
rg -n "pricing|authentication" .firecrawl/example-page.md
```

Single-format scrape/parse output is raw content. Multiple formats usually
return JSON. When using `search --scrape`, do not re-scrape those result URLs
unless the scraped payload is missing what the task needs.

For command-specific output shapes and resilient `jq` probes, open the matching
reference file rather than a separate shape reference.

## Validation

When maintaining this skill:

```bash
node scripts/firecrawl-doctor.mjs --json
node scripts/firecrawl-help-snapshot.mjs --output /tmp/firecrawl-help.json
```

The scripts are diagnostics only. They do not wrap Firecrawl operations and
they do not install Firecrawl skills.
