# codex-research CLI Reference

`codex-research` is a Rust CLI for evidence-first research workflows. It is
used by the `deep-researcher` skill and can also be run directly.

## Installation

From the repository root:

```bash
cargo build -p codex-research
cargo install --path crates/codex-research --force
codex-research --help
```

The binary supports `--json` globally for machine-readable output where
supported.

## Environment

Optional environment variables:

| Variable | Purpose |
| --- | --- |
| `CODEX_RESEARCH_HOME` | Override cache database and blob root |
| `CONTEXT7_API_KEY` | Enable direct Context7 REST calls |
| `FIRECRAWL_API_KEY` | Enable Firecrawl scrape fallback |
| `GITHUB_TOKEN` | GitHub REST fallback token |
| `GH_TOKEN` | GitHub REST fallback token |
| `EXA_API_KEY` | Records Exa readiness in doctor output; Exa calls are Codex MCP-side |

Tool fallback:

- `gh auth token` is used before `GITHUB_TOKEN`/`GH_TOKEN` for GitHub.
- `agent-browser`, `ctx7`, and `opensrc` are detected by `doctor`.

## Commands

```text
codex-research [--json] <command>
```

Commands:

- `doctor`
- `plan`
- `search`
- `fetch`
- `context7`
- `github`
- `ledger`
- `report`
- `cache`
- `eval`

## doctor

Inspect local provider auth, external tools, and cache paths.

```bash
codex-research doctor
codex-research --json doctor
```

Use this before a deep research run or when a provider command fails.

## plan

Produce a provider-aware research plan.

```bash
codex-research plan "question" --profile standard
codex-research --json plan "question" --profile deep
```

Profiles:

- `quick`
- `standard`
- `deep`
- `exhaustive`

The output includes provider budgets, route order, and research rules.

## search

Produce a source-routing plan. This command does not call native Codex web
tools; it tells the Codex session which routes to use.

```bash
codex-research search "latest Next.js routing guidance" --topic docs
codex-research search "openai/codex subagent docs" --topic github --profile deep
```

Topics:

- `general`
- `docs`
- `github`
- `dependency`
- `openai`
- `rendered`

## fetch

### fetch probe

Classify a URL and recommend the best route.

```bash
codex-research fetch probe "https://docs.github.com/en/rest/search/search"
codex-research --json fetch probe "https://github.com/openai/codex"
```

Options:

- `--max-bytes <n>`: byte cap for the probe GET, default `65536`.

Routes:

- `direct`
- `github`
- `agent-browser`
- `firecrawl`

### fetch get

Fetch a URL through direct HTTP.

```bash
codex-research fetch get "https://example.com/page.md"
codex-research fetch get "https://example.com/page.md" --store
```

Options:

- `--max-bytes <n>`: byte cap, default `512000`.
- `--store`: store response bytes in the content-addressed cache and insert a
  source-cache row.

### fetch firecrawl

Scrape a public URL through Firecrawl v2.

```bash
codex-research fetch firecrawl "https://example.com/docs"
codex-research fetch firecrawl "https://example.com/docs" --fresh --no-store-in-cache
```

Options:

- `--fresh`: request `maxAge=0`.
- `--no-store-in-cache`: disable Firecrawl server-side cache storage for
  sensitive public pages.
- `--timeout-ms <n>`: default `60000`.

Requires `FIRECRAWL_API_KEY`.

## context7

Direct Context7 REST API commands.

### context7 search

Find a Context7 library ID.

```bash
codex-research context7 search --library "Turborepo" --query "package configurations and task graph"
```

Requires `CONTEXT7_API_KEY`.

### context7 context

Retrieve documentation context.

```bash
codex-research context7 context --library-id "/vercel/turborepo" --query "package configurations"
codex-research context7 context --library-id "/vercel/next.js@v16.0.0" --query "cache components" --fast
```

Use version-pinned IDs when the target repo pins versions.

### context7 refresh

Trigger a library refresh.

```bash
codex-research context7 refresh --library-name "/vercel/turborepo"
codex-research context7 refresh --library-name "/owner/repo" --branch main
```

For latest-critical claims, verify refreshed docs against another primary
source because existing docs may be returned while refresh runs.

## github

GitHub REST calls with `gh`/token fallback.

Authentication order:

1. `GITHUB_TOKEN`
2. `GH_TOKEN`
3. `gh auth token`
4. public unauthenticated mode

### github search-repos

```bash
codex-research github search-repos "openai codex in:name" --per-page 3
```

### github search-code

```bash
codex-research github search-code 'repo:openai/codex spawn_agent' --per-page 5
```

Use narrow queries. GitHub code search has strict rate and index limitations.

### github search-issues

```bash
codex-research github search-issues 'repo:openai/codex subagent is:issue' --per-page 5
```

Search issues and pull requests. Hydrate threads before citing.

### github releases

```bash
codex-research github releases openai/codex --per-page 5
```

### github file

Fetch one file through the contents API.

```bash
codex-research github file openai/codex README.md --ref main
```

## ledger

Manage JSONL evidence ledgers.

### ledger init

```bash
codex-research ledger init
codex-research ledger init --path .codex/research/ledger.jsonl
```

### ledger add-source

```bash
codex-research ledger add-source \
  --provider github \
  --url https://github.com/openai/codex \
  --title "openai/codex" \
  --route github
```

The command prints a source ID.

### ledger add-claim

```bash
codex-research ledger add-claim \
  --text "Hydrated GitHub APIs are preferred over scraping GitHub HTML." \
  --confidence 0.9 \
  --source <source-id> \
  --note "Confirmed by local routing policy."
```

### ledger inspect

```bash
codex-research ledger inspect
codex-research --json ledger inspect --path .codex/research/ledger.jsonl
```

## report

Render a Markdown report from a ledger.

```bash
codex-research report
codex-research report --ledger .codex/research/ledger.jsonl --out .codex/research/report.md
```

## cache

Initialize or inspect global cache state.

```bash
codex-research cache init
codex-research cache stats
codex-research --json cache stats
```

Default cache root:

```text
~/.cache/codex-research/
```

## eval

Run deterministic offline checks and optional provider-readiness smoke checks.

```bash
codex-research eval
codex-research --json eval --live
```

`--live` reports configured providers; it does not perform expensive scrape or
search operations.

## Exit and Failure Behavior

- Missing required API keys produce an error from provider commands.
- Firecrawl 429 reports `Retry-After` when present.
- GitHub 403 reports rate-limit headers when present.
- `fetch probe` tries to classify even when direct fetch fails by using
  whatever metadata the HEAD request returned.
