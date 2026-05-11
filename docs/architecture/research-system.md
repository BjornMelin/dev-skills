# Research System Architecture

The research stack is designed for high-confidence engineering decisions, not
generic web summarization.

## Goals

- Prefer current primary sources.
- Avoid duplicate paid or broad provider calls.
- Make evidence auditable after the Codex session ends.
- Separate source records from claims.
- Route dynamic or blocked pages deliberately instead of blindly retrying.
- Keep private content out of external providers unless explicitly allowed.

## Provider Routing

Default route order for broad research:

1. Codex-native web for official/current checks.
2. Context7 REST for library/API docs.
3. GitHub app or REST/`gh` for repository evidence.
4. Direct fetch for static text.
5. Exa for broad semantic expansion.
6. `agent-browser` for local rendered extraction.
7. Firecrawl for public rendered/blocked/crawl-heavy pages.
8. `opensrc` for installed package source and version diffs.

The route order changes by topic. For example, dependency research puts
Context7, opensrc, and GitHub ahead of broad web; OpenAI research starts with
official OpenAI docs via native web.

## Research Profiles

`codex-research plan` supports four profiles:

| Profile | Use For | Behavior |
| --- | --- | --- |
| `quick` | low-risk lookup | tiny provider budget, no Firecrawl by default |
| `standard` | normal research | balanced web/docs/GitHub/direct fetch budget |
| `deep` | high-stakes or ambiguous work | larger budgets, broader GitHub/Exa/rendered exploration |
| `exhaustive` | rare audits | high budget, explicit cost/latency expectation |

Profiles are planning aids. Codex still decides whether a provider is useful.
For replayable runs, initialize run state and pass `--run` to provider
commands:

```bash
codex-research run init "question" --profile deep --topic github --out .codex/research/run.json
codex-research github search-issues 'repo:owner/repo behavior is:issue' --run .codex/research/run.json
codex-research run debit --run .codex/research/run.json --provider codex-web --count 1 --note "native web search"
codex-research run status --run .codex/research/run.json
```

When a provider budget is exhausted, commands fail before making the provider
call unless `--no-budget` is passed.

## Predictive Fetch Router

`codex-research fetch probe <url>` performs a cheap classification:

- GitHub URLs route to GitHub APIs.
- Text, JSON, markdown, and raw-like content route to direct fetch.
- HTML with low text density and app-shell markers routes to `agent-browser`.
- likely blocks, PDFs, or hard rendered pages route to Firecrawl if policy
  allows it.

Signals include content type, content length, byte-limited response text, script
count, app-shell markers, text density, and known hosts.

Route memory records successful provider outcomes by domain. Future probes show
the route-memory hit and can prefer the previously successful route when it does
not violate GitHub or privacy rules.

## Evidence Ledger

The ledger is JSONL and usually lives at:

```text
.codex/research/ledger.jsonl
```

Record types:

- source record: provider, URL, title, route, fetched time, source ID;
- claim record: text, confidence, source IDs, note, created time.

The Markdown report is rendered from the ledger:

```bash
codex-research report --ledger .codex/research/ledger.jsonl --out .codex/research/report.md
```

Use source IDs in final analysis so claim support can be checked later.

Close out a run with a replayable evidence bundle:

```bash
codex-research --json bundle \
  --run .codex/research/run.json \
  --ledger .codex/research/ledger.jsonl \
  --report .codex/research/report.md \
  --out .codex/research/evidence-bundle.json \
  --markdown-out .codex/research/evidence-bundle.md \
  --strict
```

The bundle is the handoff artifact for task capsules and PR descriptions. It
contains run budget status, redacted provider errors, ledger source and claim
IDs, citation coverage, source freshness counts, report path status, and
generated artifact paths. It intentionally sanitizes free-form query/debit/error
text and excludes raw provider payloads and cached page bodies.

## Cache

Global cache state defaults to:

```text
~/.cache/codex-research/
  research.sqlite
  blobs/
```

The first version initializes:

- `schema_migrations`
- `sources`
- `route_memory`
- `claims`
- content-addressed blob storage for direct fetches stored with `--store`

The source cache stores normalized source metadata for direct fetch, Context7,
GitHub, and Firecrawl responses. Raw bodies are not stored for external or
private provider responses by default.

Override the cache root with:

```bash
CODEX_RESEARCH_HOME=/tmp/codex-research codex-research cache stats
```

## Context7 Policy

Use direct Context7 API through:

```bash
codex-research context7 search --library "Library" --query "question"
codex-research context7 context --library-id "/org/project" --query "question"
codex-research context7 refresh --library-name "/org/project"
```

Rules:

- Prefer version-pinned library IDs when the target repo pins a version.
- Treat refresh-triggered docs as potentially stale until confirmed by a second
  source for latest-critical claims.
- Report 202, redirects, missing libraries, auth failures, and rate limits.
- Do not use removed Context7 research-mode behavior as a core dependency.

## GitHub Policy

Use hybrid GitHub evidence:

- Codex GitHub app/plugin for session-private PR/repo data.
- `codex-research github` for standalone REST calls.
- `gh` token fallback before environment tokens.

Search strategy:

1. Generate targeted query shards.
2. Keep `--per-page` small.
3. Hydrate promising hits into full files, issues, PRs, releases, tags, compare
   ranges, manifests, or changelogs.
4. Treat `incomplete_results`, rate limits, and code-search scope limits as
   evidence quality issues.
5. Clone or sparse checkout only when API hydration cannot prove the claim.

Standalone hydration commands now include:

```bash
codex-research github compare owner/repo main feature --per-page 100
codex-research github tags owner/repo
codex-research github release owner/repo --latest
codex-research github release owner/repo --tag v1.2.3
codex-research github issue owner/repo 123 --comments
codex-research github pr owner/repo 456 --files --comments --reviews
```

## Firecrawl Policy

Firecrawl is a paid-capacity lane under a classified policy:

- public docs may use cache;
- latest-critical pages use `--fresh`;
- sensitive public pages use `--no-store-in-cache`;
- private/confidential content is never sent unless the user explicitly allows
  external scraping for that material.

The CLI enforces this with `--privacy` and `--allow-private-external`.
Ambiguous or private/authenticated URLs are refused before the Firecrawl call by
default.

Use Firecrawl after route prediction, not as the default for every URL.

## Subagent Research Pattern

When research is split across agents:

1. The parent defines independent lanes.
2. The parent spawns the batch through `$subspawn`.
3. The parent immediately waits for every spawned agent.
4. Subagents return evidence, files/commands, findings, and risks.
5. The parent synthesizes, resolves conflicts, and writes final claims.

Subagents do not recursively spawn subagents.
