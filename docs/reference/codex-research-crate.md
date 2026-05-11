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
| `toml` | profile and provider config parsing/rendering |
| `url` | URL classification |

## Main Types

### CLI Command Types

`Cli` is the root parser:

- global `--json`
- global `--config <path>`
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
- `Bundle`
- `Cache`
- `Config`
- `Run`
- `Eval`

Nested command enums:

- `FetchCommand`
- `Context7Command`
- `GithubCommand`
- `LedgerCommand`
- `CacheCommand`
- `ConfigCommand`
- `RunCommand`

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

Budgets are emitted by `plan`, materialized by `run init`, and enforced by
provider commands when `--run <path>` is supplied.

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
- `route_memory`

### ResearchConfig

Loaded from `--config`, `CODEX_RESEARCH_CONFIG`, nearest
`.codex/research/config.toml`, `$XDG_CONFIG_HOME/codex-research/config.toml`,
or built-in defaults.

Config sections:

- `profiles`: per-profile provider budgets;
- `privacy`: external-provider default posture;
- `providers.github`: per-page and retry defaults;
- `providers.context7`: metadata TTL and version-pin policy;
- `providers.firecrawl`: cache and max-age policy;
- `cache`: source metadata and raw-body storage defaults.

### ResearchRunState

JSON run state created by `run init`:

- `query`
- `profile`
- `topic`
- `status`
- `budgets`
- `spent`
- `debits`
- `provider_errors`
- `source_ids`

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
- `build_evidence_bundle_command`
- `handle_cache`
- `handle_config`
- `handle_run`
- `run_eval`

Async provider commands run under `tokio`.

`run_eval` uses a default suite embedded at build time from
`crates/codex-research/evals/research/core.json`; only `--suite` reads a
caller-supplied suite file at runtime. The eval harness is deliberately
offline-first so it can run in PR validation without provider credentials. It
supports task filtering, listing, strict warning handling, and JSON output.

### EvidenceBundle

`bundle` composes existing research contracts into
`codex-research.evidence-bundle.v1`:

- run metadata, status, cache source IDs, provider budgets, and sanitized debit
  history;
- redacted unresolved provider errors;
- ledger source/claim IDs;
- citation coverage, uncited claims, and missing claim source references;
- source freshness counts resolved through the cache when possible;
- report path status, artifact paths, warnings, and closeout failures.

The bundle status is `failed` when closeout evidence is incomplete: uncited
claims, missing source references, unresolved provider errors, missing
ledger/report artifacts, or missing source freshness records in strict mode.
Non-strict command execution still exits zero after writing output for
inspection; strict mode exits nonzero for recorded failures. Bundle generation
records metadata only, sanitizes free-form handoff text, and does not embed raw
provider payloads.

## Provider Implementations

### HTTP Client

`http_client()` creates a `reqwest::Client` with:

- `User-Agent: codex-research/0.2`
- `Accept: application/json, text/plain, text/html`
- redirect limit of 8
- 30-second timeout

### Context7

`handle_context7` requires `CONTEXT7_API_KEY` and calls:

- `GET https://context7.com/api/v2/libs/search`
- `GET https://context7.com/api/v2/context`
- `POST https://context7.com/api/v1/refresh`

Provider responses are wrapped with a cached `source_id`. The raw provider
payload stays under `data`; normalized source metadata is stored in SQLite.
Context7 202, redirects, 429, 503, and 504 are surfaced as explicit failures.

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
- latest release and release-by-tag
- compare refs
- tags
- issue hydration and comments
- PR hydration, files, comments, and reviews
- contents API file fetch

All GitHub commands store normalized source metadata and return a top-level
`source_id`. Search commands attach limitations metadata. Compare responses are
augmented with a `file_summary` array.

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
- `--privacy`;
- `--allow-private-external`.

429 responses report `Retry-After` when present.

## Cache

`research_paths` resolves:

- `CODEX_RESEARCH_HOME`, when set;
- otherwise `~/.cache/codex-research`.

`init_db` creates:

- `schema_migrations`
- `sources`
- `route_memory`
- `claims`

`store_blob` writes content-addressed blobs under:

```text
<cache>/blobs/<first-two-hash-chars>/<full-sha256>
```

`record_source_cache` stores normalized source metadata for direct fetches,
Context7, GitHub, and Firecrawl. Raw bodies are stored only for direct fetches
saved with `--store`.

Cache commands can list sources, inspect one source ID, inspect route memory,
and prune old source rows.

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

- add direct CLI-owned Exa calls if MCP-side Exa proves insufficient;
- add richer live eval cost caps if optional live checks expand;
- add more manifest task kinds only when they can stay deterministic in
  provider-disabled CI;
- split provider implementations into modules once the single-binary shape
  stops being reviewable.

## Validation

Required checks after crate changes:

```bash
cargo fmt --all --check
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
codex-research --json doctor
codex-research --json eval
codex-research --json eval --task evidence-claims-cited --strict
codex-research --json eval --task evidence-bundle-closeout-shape --strict
git diff --check
```

Optional live readiness check:

```bash
codex-research --json eval --live
```
