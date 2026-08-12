# Parallel CLI Reference

Workflow reference for the commands the `parallel-cli` skill routes to.
Verified against `parallel-cli 0.8.2` (`parallel-cli <sub> --help`). Treat
this as the command baseline; newer CLI releases may add flags.

Intentionally out of scope: `enrich deploy` (cloud integration deploy for
BigQuery/Snowflake/etc.) and `skills install` (installs the official split
skill set; this consolidated skill replaces those).

## Version gates

| Capability | Minimum version |
| --- | --- |
| `research` / `enrich` (incl. `enrich suggest`) | 0.3.0 |
| `findall` command | 0.3.0 |
| `monitor` GA commands | 0.4.0 |
| `findall entity-search` | 0.6.0 |
| `memory` | 0.8.1 |

When a command errors with `no such command` / `no such option`, update the
CLI by its install method (`parallel-cli update` for standalone,
`uv tool upgrade "parallel-web-tools[cli]"`, `pipx upgrade
parallel-web-tools`, `npm update -g parallel-web-cli`, or
`brew update && brew upgrade parallel-web/tap/parallel-cli`) and retry.

## Top-level

```text
auth        Check authentication status (--json)
balance     Inspect or top up the org's prepaid credit balance
completion  Shell completion for bash, zsh, fish
config      View or set CLI configuration (standalone CLI only)
enrich      Data enrichment commands
extract     Extract content from URLs as clean markdown (alias: fetch)
findall     Discover entities from the web using natural language
login       Authenticate via device OAuth flow (--json, --no-browser)
logout      Remove stored credentials
memory      Search and manage saved Task, Monitor, FindAll entries
monitor     Continuously track the web for changes
research    Deep research commands for open-ended questions
search      Search the web using Parallel's AI-powered search
skills      Install and manage Parallel agent skills
update      Update to the latest version (standalone CLI only)
```

## search

```bash
parallel-cli search [OBJECTIVE] -q "keyword" -q "keyword2" --json \
  --max-results 10 --excerpt-max-chars-total 27000 -o "/tmp/<name>.json"
```

- `OBJECTIVE` is a natural-language goal; at least one of `OBJECTIVE` or
  `-q/--query` is required. `-` reads from stdin.
- `--mode` `turbo|basic|advanced` (default `basic`); aliases `fast`→basic,
  `one-shot`/`agentic`→advanced. `turbo`: p50 ~200ms, English/Japanese only.
- `--include-domains` / `--exclude-domains`: comma-separated or repeated.
- `--after-date YYYY-MM-DD`, `--location <ISO-3166-1-alpha-2>`.
- `--max-results` (server default 10), `--excerpt-max-chars-per-result`
  (min 1000), `--excerpt-max-chars-total` (default 60000),
  `--max-age-seconds` (min 600), `--timeout-seconds`.
- Do not set `max_output_tokens` on the harness call; it truncates the JSON.
  Read the `-o` file, not stdout. Response must cite every claim from the
  output as a markdown link to the source URL and end with a Sources section.

## extract

```bash
parallel-cli extract "https://example.com/page" --json -o "/tmp/<name>.json"
```

- `URLS...` is one or more URLs. `-o` always writes JSON; extension must be
  `.json`. Max 20 URLs per request and `--objective` max 5000 characters;
  batch larger URL sets across multiple calls.
- `--objective "focus area"` (also silences the no-objective warning),
  `-q "keyword"` repeatable, `--full-content` (complete page body),
  `--full-content-max-chars N`, `--no-excerpts`, `--session-id` to group
  related search/extract calls, `--client-model` (e.g. claude-opus-4-7),
  `--disable-cache-fallback` (error instead of stale cached content),
  `--excerpt-max-chars-*`, `--max-age-seconds`, `--timeout-seconds`.
- Failure: `errors` field, empty `results`, or 404/timeout means the
  extraction failed. Never fabricate content; surface the upstream status.

## research

```bash
parallel-cli research run "<topic>" --processor pro-fast --text --no-wait --json
parallel-cli research poll "<RUN_ID>" -o "/tmp/<name>" --timeout 540
parallel-cli research status "<RUN_ID>" --json
parallel-cli research processors
```

- `run`: query via positional arg, `-f/--input-file`, or `-` (stdin).
  `--processor` tier, `--text` (markdown report; `--text-description` steers
  length/focus), `--no-wait` (return immediately), `--json`,
  `--previous-interaction-id` (context chaining), `--memory-scope-key`.
- `poll`: `-o` writes `<name>.json` always, `<name>.md` only when `run` used
  `--text`. `--timeout` (use 540), `--force` to overwrite files. Prints an
  executive summary to stdout on completion. Do not pass `--json`.
- `run` envelope: `{run_id, interaction_id, result_url, processor, status,
  output_schema}`. Share `result_url` for monitoring, keep `interaction_id`
  for follow-ups, do not re-share the URL after completion.

### Processors

| Processor | Latency | Use when |
| --- | --- | --- |
| `lite-fast` | 10-20s | Quick lookups, follow-ups |
| `base-fast` | 15-50s | Simple questions |
| `core-fast` | 15s-100s | Moderate research |
| `core2x-fast` | 15s-3min | Extended research |
| `pro-fast` | 30s-5min | **Default**, exploratory research |
| `ultra-fast` | 1-10min | Multi-source deep research (~2x cost) |
| `ultra2x-fast` | 1-20min | Difficult deep research |
| `ultra4x-fast` | 1-40min | Very difficult research |
| `ultra8x-fast` | 1min-1hr | Most challenging research |
| `lite` | 10-60s | Quick lookups, fresher data |
| `base` | 15-100s | Simple questions, fresher data |
| `core` | 1-5min | Moderate research, fresher data |
| `core2x` | 1-10min | Extended research, fresher data |
| `pro` | 2-10min | Exploratory research, fresher data |
| `ultra` | 5-25min | Advanced deep research, fresher data |
| `ultra2x` | 5-50min | Difficult deep research, fresher data |
| `ultra4x` | 5-90min | Very difficult research, fresher data |
| `ultra8x` | 5min-2hr | Most challenging research, fresher data |

Run `parallel-cli research processors` for the live list. `-fast` tiers use
cached web data; the non-`-fast` tiers re-fetch fresher data, slower but
better for events from the last day or two. Use a lighter tier with
`--previous-interaction-id` for follow-ups.

## enrich

```bash
parallel-cli enrich suggest "Find CEO and recent funding info" --json
parallel-cli enrich run --data '[{"company": "Google"}]' \
  --intent "CEO name and founding year" --target "/tmp/enrichment-<name>.csv" --no-wait --json
parallel-cli enrich poll "<TASKGROUP_ID>" --timeout 540 \
  --output "/tmp/enrichment-<name>.json"
parallel-cli enrich status "<TASKGROUP_ID>" --json
```

- CSV input: `--source-type csv --source "input.csv" --target
  "/tmp/enrichment-<name>.csv" --source-columns
  '[{"name": "company", "description": "Company name"}]'`.
- `--target` is syntactically required on `run` (with `--source-type` +
  `--source` + `--source-columns`, or with `--data`) even though
  `--no-wait` polling writes the real results through `poll --output`.
- `--intent` and `--enriched-columns` are alternatives; `suggest` returns
  `{title, processor, enriched_columns, warnings}`. Pass `enriched_columns`
  and the suggested `--processor` to `run`. Skip `suggest` when the user
  already specified fields.
- `run` envelope: `{taskgroup_id, url, num_runs}`. There is no
  `interaction_id`; enrichment does not produce one, so it cannot chain a
  further follow-up. `--previous-interaction-id` chains a prior research task
  into the enrichment.
- `poll` output is a JSON array of `{input, output}` regardless of the
  `--output` extension; name it `.json`.

## findall

```bash
parallel-cli findall run "<objective>" --no-wait --json
parallel-cli findall poll "<FINDALL_ID>" -o "/tmp/<name>.json" --timeout 540
parallel-cli findall entity-search "<objective>" -t companies -n 100 -o "/tmp/<name>.json"
parallel-cli findall enrich "<FINDALL_ID>" '{"properties":{"ceo":{"type":"string"}}}'
parallel-cli findall extend "<FINDALL_ID>" 50
parallel-cli findall ingest "<objective>" --json
parallel-cli findall status "<FINDALL_ID>" --json
parallel-cli findall result "<FINDALL_ID>" --json
parallel-cli findall schema "<FINDALL_ID>" --json
parallel-cli findall cancel "<FINDALL_ID>" --json
```

- `run`: `-g base|core|pro` (default `core`), `-n` 5-1000 (default 10),
  `--exclude '[{"name":"Google","url":"google.com"}]'`, `--metadata`,
  `--memory-scope-key`, `--dry-run` (ingest preview without creating a run),
  `--timeout`/`--poll-interval` (when waiting in-line).
- `entity-search`: `-t companies|people` (required), `-n` (default 10; ask
  for more than needed, e.g. 100). Synchronous, no `findall_id`. Returns
  `{entity_set_id, entities: [{name, url, description}]}`. Directory/profile
  URLs are expected here; drop only empty URLs and query-echo names. The
  `entity_set_id` is not usable with `enrich`/`extend`.
- Noise filter for `run` results: drop empty URLs, names that echo the query,
  and third-party directory URLs. Spot-check `-g base` matches against source
  URLs; if many fail, re-run with `-g core` or higher.
- `enrich` schema: JSON Schema-style object mapping field names to
  `{type, description?}`.

## monitor

```bash
parallel-cli monitor create "<query>" --frequency 1d --json   # --webhook URL, --metadata JSON, --output-schema JSON
parallel-cli monitor create --type snapshot --task-run-id "trun_abc" --frequency 1d --json
parallel-cli monitor list -n 10 --json                        # --status active|cancelled (repeatable); --type filter
parallel-cli monitor events "<MONITOR_ID>" --json             # newest first; --cursor pages; --event-group-id
parallel-cli monitor get "<MONITOR_ID>" --json
parallel-cli monitor update "<MONITOR_ID>" --frequency 1w --json
parallel-cli monitor trigger "<MONITOR_ID>" --json
parallel-cli monitor cancel "<MONITOR_ID>" --json             # irreversible; confirm first
```

- Frequency: `<n><h|d|w>` (e.g. `1h`, `1d`, `1w`) or aliases `hourly`,
  `daily`, `weekly`, `every_two_weeks`.
- Two monitor types: `event_stream` (default) tracks a search query;
  `snapshot` tracks a Task Run's output, so omit QUERY and pass
  `--task-run-id` instead. Both accept `--processor lite|base` (default
  `lite`; `base` is more thorough at higher cost), `--webhook`,
  `--metadata`, and `--memory-scope-key`. `--include-backfill` (event_stream
  only) includes a sample of historical events on the first run.
- The query cannot be updated; create a new monitor. `trigger` enqueues a
  real off-schedule run and emits an event only on material change.

## memory

```bash
parallel-cli memory retrieve --query "serverless inference vendors"
parallel-cli memory retrieve --limit 5                          # recent memories
parallel-cli memory retrieve --kind task --since 2026-08-01T00:00:00Z
parallel-cli memory evict --kind task --id "trun_example"
parallel-cli memory clear --confirm-clear
```

- `kind` filter: `task`, `monitor`, `findall`. `since`: RFC 3339. Empty
  `results` is a miss, not an error. Evict/clear do not delete underlying
  runs. Ingestion is asynchronous.

## skills / auth / balance / login

```bash
parallel-cli skills list
parallel-cli skills install      # intentionally excluded: installs the official split skill set
parallel-cli skills reinstall    # this consolidated skill replaces those
parallel-cli skills uninstall    # PARALLEL_SKILLS_INDEX_URL overrides the index
parallel-cli auth --json
parallel-cli login --json
parallel-cli login --json --no-browser                      # headless sessions; streams auth_start/device_code/auth_waiting/auth_success
parallel-cli balance --json get
parallel-cli balance add <AMOUNT_IN_CENTS>
```

`auth --json` fields: `authenticated`, `method`, `env_var_set`,
`has_stored_credentials`, `stored_overridden_by_env`, `token_file`,
`version`, `selected_org_id`, `selected_org_name`,
`has_control_api_tokens`. `selected_org_id: "legacy"` means re-login.

## Conventions

- `-o` output is always JSON. `.md` files are written only by `research poll`
  when `run` used `--text`.
- Polls: `--timeout 540`; re-run the same poll to keep waiting. Never pass
  `--json` to poll commands for large result sets.
- Balance: `403` responses indicate a likely balance problem; check
  `balance --json get` and ask before `balance add`.
- Headless auth: `login --no-browser`; block until `{"event":
  "auth_success"}`.
