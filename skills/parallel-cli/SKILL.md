---
name: parallel-cli
description: "Use the Parallel CLI for web data work: URL extraction, deep research, data enrichment, entity discovery (findall), memory recall, and web monitoring. Web search goes to the parallel-search MCP server (free); escalate to `parallel-cli search` only when MCP is insufficient. Use for deep research, exhaustive investigation, extracting full content from URLs, enriching lists, finding entities, recalling prior runs, tracking web changes, and CLI setup/auth/balance."
license: MIT
allowed-tools: Bash(command:*), Bash(brew:*), Bash(uv:*), Bash(npm:*), Bash(pipx:*), Bash(curl:*), Bash(rm:*), Bash(parallel-cli:*)
metadata:
  author: parallel
---

# Parallel CLI

One interface for Parallel web data work: extract, deep research, enrichment, findall, memory, and monitor. Requires `parallel-cli` installed, authenticated, and funded.

## Routing: MCP first, CLI for depth

- **General web search, quick lookups, fact-checks**: use the `parallel-search` MCP server (`web_search`) when it is configured. It is free. Do not call `parallel-cli search` for these. When the MCP server is absent, use the harness's native web search, or fall back to `parallel-cli search`.
- Escalate to `parallel-cli search` only when MCP is insufficient: `--mode advanced` for hard multi-step questions, `--after-date` freshness, `--include-domains` / `--exclude-domains`, `--location` geo-targeting, or high-volume batch queries.
- **URL content extraction**: `parallel-cli extract` (cleaner than MCP `web_fetch` for PDFs, JS-heavy pages, and long articles).
- **Deep/exhaustive research** (user says "deep", "exhaustive", "comprehensive report", "thorough investigation"): `research run` + `poll`.
- **Structured entity lists** ("find all X", "list every Y"): `findall`. **Add fields to a list you already have**: `enrich`.
- **Recall prior runs**: `memory`. **Continuous change tracking**: `monitor`.
- Do not use the parallel-task MCP for research or enrichment; the CLI is the tool of record.

## Setup

Check and install, requiring `parallel-cli >= 0.8.1`:

```bash
command -v parallel-cli && parallel-cli --version
# install: brew install parallel-web/tap/parallel-cli (macOS)
#          uv tool install "parallel-web-tools[cli]"   (pipx: pipx install "parallel-web-tools[cli]" && pipx ensurepath)
#          npm install -g parallel-web-cli
# upgrade by install method: parallel-cli update | uv tool upgrade "parallel-web-tools[cli]" | pipx upgrade "parallel-web-tools[cli]" | npm update -g parallel-web-cli | brew update && brew upgrade parallel-web/tap/parallel-cli
```

When the CLI is present but old, detect the install method before upgrading:
`command -v parallel-cli`, then `readlink "$(command -v parallel-cli)"` if it is a symlink. Paths under `~/.local/share/uv/tools/` mean `uv tool install`; paths under `~/.local/share/parallel-cli/` mean the standalone installer (use `parallel-cli update` only for standalone installs).

Authenticate if `parallel-cli auth --json` shows `"authenticated": false` or `"selected_org_id": "legacy"`:

```bash
parallel-cli login --json   # add --no-browser in headless sessions
```

This triggers device OAuth. Wait for `{"event": "auth_success"}`. Then check balance: `parallel-cli balance --json get`. If zero, ask the user before running `parallel-cli balance add <AMOUNT_IN_CENTS>`. A `403` on any command usually means balance is required; offer `balance --json get` and ask before topping up.

## Common execution rules

- Prefer writing results to disk with `-o "/tmp/<descriptive>.json"` and reading that file over parsing stdout. Large JSON floods or truncates tool output.
- Do NOT pass `--json` to poll commands; it floods context. `-o` saves results to disk.
- Use `--timeout 540` (9 minutes) on polls to stay within tool execution limits.
- Cite only URLs that appear in command output. Never invent or guess URLs.
- After any run, tell the user the output file path so they can ask follow-ups.

## Workflows

### Extract: URL content

```bash
parallel-cli extract "https://example.com/article" --json -o "/tmp/<name>.json"
```

Options: `--objective "focus area"`, `-q "keyword"` (repeatable), `--full-content` (complete page body; also retry with this when excerpts come back empty), `--no-excerpts`, `--session-id` to group related search/extract calls. On failure (errors field, empty results, 404/timeout), do NOT fabricate content: report the failure, suggest verifying the URL, retrying with `--full-content`, or locating the current URL via search.

Present the result as **Page Title** with its URL, followed by the content verbatim: preserve every numbered/bulleted item, keep all facts, names, numbers, dates, and quotes; strip only obvious noise (nav menus, footers, ads).

### Deep research

```bash
parallel-cli research run "<topic>" --processor pro-fast --text --no-wait --json
parallel-cli research poll "<RUN_ID>" -o "/tmp/<name>" --timeout 540
parallel-cli research status "<RUN_ID>" --json   # quick status check
```

- `--text` returns a markdown report with inline citations; `--text-description` steers length/focus. Drop `--text` for structured JSON.
- `--no-wait` is required on `run`; the call returns instantly with `{run_id, interaction_id, result_url, processor, status, output_schema}`. Share the `result_url` for monitoring and the expected latency, and delegate the poll (see Delegation).
- Default processor `pro-fast`; follow-ups reuse the prior `interaction_id` via `--previous-interaction-id` with a lighter tier (`lite-fast`). `-fast` tiers use cached data; non-`-fast` tiers re-fetch fresher data for very recent events. See the reference for the full processor table.
- Poll writes `<name>.json` always, plus `<name>.md` when `run` used `--text`. On timeout, re-run the same poll. After completion: share the executive summary printed to stdout, the file paths, and the `interaction_id`. Do NOT re-share the monitoring URL. Do NOT read the result files into context unless asked.
- Pass `--memory-scope-key` to scope research to a workspace.

### Enrichment

```bash
parallel-cli enrich suggest "Find CEO and recent funding info" --json   # optional: columns + processor recommendation
parallel-cli enrich run --data '[{"company": "Google"}, {"company": "Microsoft"}]' --intent "CEO name and founding year" --target "/tmp/enrichment-<name>.csv" --no-wait --json
parallel-cli enrich poll "<TASKGROUP_ID>" --timeout 540 --output "/tmp/enrichment-<name>.json"
```

- CSV source: `--source-type csv --source "input.csv" --target "/tmp/enrichment-<name>.csv" --source-columns '[{"name": "company", "description": "Company name"}]'`.
- `--target` is syntactically required on `enrich run` even though `--no-wait` polling writes the real results through `poll --output`.
- Use `--enriched-columns` (array from `suggest`, in place of `--intent`) when the user gave a vague intent.
- Output envelope: `{taskgroup_id, url, num_runs}`; there is no `interaction_id`. Tell the user enrichment is running and share the `url`, then delegate the poll.
- Output is a JSON array of `{input, output}` objects regardless of file extension. Report row count, preview a few rows, give the path.

### FindAll: entity discovery

```bash
parallel-cli findall run "<objective>" --no-wait --json        # comprehensive; {findall_id}
parallel-cli findall poll "<FINDALL_ID>" -o "/tmp/<name>.json" --timeout 540
parallel-cli findall entity-search "<objective>" -t companies -n 100 -o "/tmp/<name>.json"   # fast, throwaway lists only
parallel-cli findall enrich "<FINDALL_ID>" '{"properties":{"ceo":{"type":"string"}}}'         # add fields
parallel-cli findall extend "<FINDALL_ID>" 50                                                 # more matches
parallel-cli findall ingest "<objective>" --json   # preview inferred schema before paying for a run
```

- `findall run` defaults: generator `core`, match limit `10`. `-g pro` for comprehensive coverage, `-g base` only when the user accepts noise. `-n 5..1000`. Exclusions: `--exclude '[{"name":"Google","url":"google.com"}]'`.
- `entity-search` is synchronous, companies/people only, no exclusions/enrichment, and its `entity_set_id` cannot be used with `enrich`/`extend`. Use only when the user explicitly wants a fast, rough list.
- Filter noise before presenting: drop entries with empty/missing `url`, names that echo the query, and third-party directory/profile URLs (entity-search's directory links are expected; keep them). Sanity-check `-g base` matches against source URLs for falsifiable criteria.
- Present matches as a markdown table, lead with the count, cite each entity's URL.

### Memory

```bash
parallel-cli memory retrieve --query "serverless inference vendors"   # topic recall
parallel-cli memory retrieve --limit 5                                # recent runs
parallel-cli memory evict --kind task --id "trun_example"
parallel-cli memory clear --confirm-clear
```

- Recall prior Task, Monitor, or FindAll runs before launching overlapping work; fetch the source run for full records. Empty `results` is a successful miss, not an error.
- On a memory-eligibility error, report the returned reason: it distinguishes rollout, organization settings, account opt-in, and key eligibility. On a key-eligibility error, tell the user to reauthenticate.
- Fields vary by kind: `task` uses `id/updated_at/input_excerpt/output_excerpt`; `monitor` uses event excerpts; `findall` uses objective excerpt and `matched_count`.
- Evicting/clearing only affects Memory, not the underlying runs. Confirm before `clear` unless already asked. Ingestion is asynchronous; a just-completed run may not be retrievable yet.

### Monitor

```bash
parallel-cli monitor create "<query>" --frequency 1d --json        # 1h|1d|1w or daily/weekly; --webhook URL optional
parallel-cli monitor list -n 10 --json
parallel-cli monitor events "<MONITOR_ID>" --json                  # --cursor pages; --event-group-id for full payload
parallel-cli monitor get "<MONITOR_ID>" --json
parallel-cli monitor update "<MONITOR_ID>" --frequency 1w --json
parallel-cli monitor trigger "<MONITOR_ID>" --json                 # one real off-schedule run
parallel-cli monitor cancel "<MONITOR_ID>" --json                  # always confirm; irreversible
```

- Monitors are server-side, persist until cancelled, and can deliver events via `--webhook`. The query cannot be updated; create a new monitor instead.
- Match frequency to change rate: hourly for prices/news, weekly for filings/staffing.

## Delegation

Long-running work (research poll, enrich poll, findall poll) should run in a background subagent with a self-contained prompt: the exact command, the output file path, and the instruction to return only the executive summary and file paths, never inventing URLs. Spawn per harness:

- **opencode**: `task` subagent on `opencode-go/deepseek-v4-flash` or luna `high`/`max`, with Bash access only.
- **Codex**: spawn a fresh thread via `codex exec` (the native multi_agent_v2 tool does not support luna), launched in the background, writing results to a writable path:
  ```bash
  codex exec -C "<repo>" -m gpt-5.6-luna -c model_reasoning_effort="max" --sandbox workspace-write --output-last-message "/tmp/codex-out.md" "<self-contained prompt>" &
  # follow-ups: codex exec resume --last "<instruction>"
  ```
  The prompt must name the exact `parallel-cli ... -o "/tmp/<name>.json"` command; the thread needs `workspace-write` so its `-o` writes succeed.
- **Claude Code**: subagent with pinned `model` and `effort`, `tools: Bash`, bounded `maxTurns`, dispatched via Task. Prefer a `model: inherit`-free pin per the repo routing matrix.
- **Kimi Code**: `Agent` tool spawn with `subagent_type: coder` (or `explore` for read-only), with a model override where the harness router supports one. UNVERIFIED: luna wiring depends on the local Kimi provider/alias configuration.

## Reference

Workflow reference, processor table, JSON envelopes, and version gates: `references/parallel-cli-reference.md`.
