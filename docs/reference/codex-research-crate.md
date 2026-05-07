# codex-research Crate Reference

This is a handwritten reference for `crates/codex-research`. Do not commit
generated Rustdoc or `target/`.

Generate local Rustdoc when needed:

```bash
cargo doc -p codex-research --no-deps --open
```

## Package

```text
crates/codex-research/
  Cargo.toml
  src/main.rs
```

The crate currently ships one binary:

```toml
[[bin]]
name = "codex-research"
path = "src/main.rs"
```

## Dependencies

| Crate | Use |
| --- | --- |
| `anyhow` | error context and early bail-outs |
| `chrono` | timestamped ledger/cache records |
| `clap` | CLI parser and command surface |
| `directories` | platform-aware cache root |
| `reqwest` | async HTTP client |
| `rusqlite` | SQLite cache and route memory |
| `serde` / `serde_json` | JSON output, ledgers, provider responses |
| `sha2` | content-addressed blob IDs and short IDs |
| `tokio` | async runtime |
| `url` | URL classification |

## Main Types

### CLI Command Types

`Cli` is the root parser:

- global `--json`
- subcommand enum `Commands`

`Commands` variants:

- `Doctor`
- `Plan`
- `Search`
- `Fetch`
- `Context7`
- `Github`
- `Ledger`
- `Report`
- `Cache`
- `Eval`

Nested command enums:

- `FetchCommand`
- `Context7Command`
- `GithubCommand`
- `LedgerCommand`
- `CacheCommand`

### ResearchProfile

Profiles control planning budgets:

- `Quick`
- `Standard`
- `Deep`
- `Exhaustive`

Serialized as kebab-case strings.

### TopicKind

Search topic hints:

- `General`
- `Docs`
- `Github`
- `Dependency`
- `Openai`
- `Rendered`

### Route

Provider route labels:

- `CodexWeb`
- `Context7`
- `Github`
- `Direct`
- `AgentBrowser`
- `Firecrawl`
- `Exa`
- `Opensrc`

These are serialized as kebab-case strings such as `codex-web` and
`agent-browser`.

### ResearchPlan

Returned by `plan` and `search`:

- `query`
- `profile`
- `budgets`
- `route_order`
- `rules`

### ProviderBudgets

Budget fields:

- `codex_web_queries`
- `context7_calls`
- `github_calls`
- `exa_calls`
- `direct_fetches`
- `browser_fetches`
- `firecrawl_calls`

Budgets are planning guidance, not enforced hard limits yet.

### ProbeReport

Returned by `fetch probe`:

- `url`
- `status`
- `content_type`
- `content_length`
- `text_chars`
- `script_markers`
- `app_shell_markers`
- `route`
- `reason`

### LedgerRecord

Tagged enum:

- `source`
- `claim`

`SourceRecord`:

- `id`
- `provider`
- `url`
- `title`
- `route`
- `fetched_at`

`ClaimRecord`:

- `id`
- `text`
- `confidence`
- `sources`
- `note`
- `created_at`

## Execution Flow

`main` parses CLI arguments and dispatches to:

- `doctor`
- `output_plan`
- `output_search_plan`
- `handle_fetch`
- `handle_context7`
- `handle_github`
- `handle_ledger`
- `render_report`
- `handle_cache`
- `run_eval`

Async provider commands run under `tokio`.

## Provider Implementations

### HTTP Client

`http_client()` creates a `reqwest::Client` with:

- `User-Agent: codex-research/0.1`
- `Accept: application/json, text/plain, text/html`
- redirect limit of 8
- 30-second timeout

### Context7

`handle_context7` requires `CONTEXT7_API_KEY` and calls:

- `GET https://context7.com/api/v2/libs/search`
- `GET https://context7.com/api/v2/context`
- `POST https://context7.com/api/v1/refresh`

Provider responses are returned as JSON without lossy remapping. This preserves
Context7 metadata such as library IDs, trust, benchmark score, snippets, and
refresh status.

### GitHub

`handle_github` uses `github_get`.

Auth resolution:

1. `GITHUB_TOKEN`
2. `GH_TOKEN`
3. `gh auth token`
4. unauthenticated public request

Requests send `X-GitHub-Api-Version`.

Supported endpoints:

- repository search
- code search
- issue/PR search
- releases
- contents API file fetch

### Direct Fetch

`direct_fetch` sends a ranged GET using `Range: bytes=0-<max>`, decodes bytes as
UTF-8 lossily, and returns status, content type, byte count, and body.

### Predictive Probe

`probe_url` tries:

1. HEAD request for status/content metadata.
2. Direct fetch with byte cap.
3. `classify_body` for route recommendation.

`classify_body` detects:

- GitHub URLs;
- text-like content;
- PDF-like content;
- Cloudflare/block hints;
- JavaScript app shells;
- high script count plus low text density.

### Firecrawl

`firecrawl_scrape` requires `FIRECRAWL_API_KEY` and posts to:

```text
https://api.firecrawl.dev/v2/scrape
```

The command supports:

- `--fresh` mapping to `maxAge=0`;
- `--no-store-in-cache`;
- `--timeout-ms`.

429 responses report `Retry-After` when present.

## Cache

`research_paths` resolves:

- `CODEX_RESEARCH_HOME`, when set;
- otherwise `~/.cache/codex-research`.

`init_db` creates:

- `sources`
- `route_memory`
- `claims`

`store_blob` writes content-addressed blobs under:

```text
<cache>/blobs/<first-two-hash-chars>/<full-sha256>
```

`record_source_cache` stores source metadata for direct fetches saved with
`--store`.

## Ledger and Report

Ledger helpers:

- `append_ledger_record`
- `read_ledger_records`
- `ensure_parent`

Reports are intentionally simple Markdown:

- claims first;
- sources second;
- source IDs preserved.

This keeps the report readable while the JSONL ledger remains the audit source.

## Extension Points

Good next additions:

- enforce provider budgets during `plan`-driven runs;
- persist route-memory outcomes from probes and provider failures;
- add structured source-cache metadata for Context7/GitHub/Firecrawl results;
- add more offline eval fixtures for app shells, PDFs, redirects, and blocked
  pages;
- add subcommands for GitHub compare/tags/issues hydration;
- add an Exa REST provider if direct CLI-owned Exa calls become necessary.

## Validation

Required checks after crate changes:

```bash
cargo fmt --all --check
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
codex-research --json doctor
codex-research --json eval
git diff --check
```

Optional live readiness check:

```bash
codex-research --json eval --live
```
