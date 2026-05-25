# Install And Auth

Use this reference only when setup is missing or authentication is failing.

## Diagnose

```bash
firecrawl --version
firecrawl --status
firecrawl <command> --help
```

Healthy status shows an authenticated account, concurrency, credits, and
`.firecrawl`/`.gitignore` posture when applicable.

## Auth

Use auth commands only when the user explicitly asks to authenticate or repair
auth:

```bash
firecrawl login --browser
firecrawl login --api-key "$FIRECRAWL_API_KEY"
firecrawl logout
firecrawl view-config
```

Never print API keys. Prefer environment variables or the CLI credential store.

## Failure Recovery

Use local command output as authority. Do not guess at account state.

| Symptom | Recovery |
| --- | --- |
| Binary missing | Report the missing `firecrawl` binary and ask before installing global tooling. |
| 401 or unauthenticated | Run `firecrawl --status`; if the user asks to repair auth, use `firecrawl login --browser` or `firecrawl login --api-key "$FIRECRAWL_API_KEY"`. |
| 402, credit exhausted, or quota exceeded | Reduce scope only if the user asked for a smaller job; otherwise stop and report the account limitation. |
| Rate limited or concurrency blocked | Reduce `--limit`, add crawl pacing, wait for active jobs, or retry later. |
| Timeout | Narrow scope, add command-specific `--timeout`, or use `--wait-for` for rendered scrape content. |
| Blocked, empty, or JS-heavy page | Retry `scrape` with `--wait-for`; use `interact` only after a scrape establishes a browser session. |
| Monitor unavailable | Some teams cannot use monitoring with zero-data-retention settings; fall back to one-off scrape plus local diff. |
| Missing interact session | Run `scrape` first and then `interact`, or pass `--scrape-id`. |
| Malformed JSON | Preserve raw output, rerun with `--json --pretty -o`, and validate with `jq`. |
| Skill docs disagree with help | Trust `firecrawl <command> --help` for the current run and update this skill later. |

## Install

If the binary is missing, explain the missing prerequisite and ask before
installing global tooling. If the user approves, install the current released
CLI package through Bun:

```bash
bun add -g firecrawl-cli@latest
```

For one-off checks without a global install, use `bunx firecrawl-cli@latest`.
Do not run Firecrawl skill setup commands:

```bash
# Do not run from this skill:
firecrawl init
firecrawl setup skills
```
