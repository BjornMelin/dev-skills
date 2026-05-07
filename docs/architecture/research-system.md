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

## Predictive Fetch Router

`codex-research fetch probe <url>` performs a cheap classification:

- GitHub URLs route to GitHub APIs.
- Text, JSON, markdown, and raw-like content route to direct fetch.
- HTML with low text density and app-shell markers routes to `agent-browser`.
- likely blocks, PDFs, or hard rendered pages route to Firecrawl if policy
  allows it.

Signals include content type, content length, byte-limited response text, script
count, app-shell markers, text density, and known hosts.

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

## Cache

Global cache state defaults to:

```text
~/.cache/codex-research/
  research.sqlite
  blobs/
```

The first version initializes:

- `sources`
- `route_memory`
- `claims`
- content-addressed blob storage for direct fetches stored with `--store`

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

## Firecrawl Policy

Firecrawl is a paid-capacity lane under a classified policy:

- public docs may use cache;
- latest-critical pages use `--fresh`;
- sensitive public pages use `--no-store-in-cache`;
- private/confidential content is never sent unless the user explicitly allows
  external scraping for that material.

Use Firecrawl after route prediction, not as the default for every URL.

## Subagent Research Pattern

When research is split across agents:

1. The parent defines independent lanes.
2. The parent spawns the batch through `$subspawn`.
3. The parent immediately waits for every spawned agent.
4. Subagents return evidence, files/commands, findings, and risks.
5. The parent synthesizes, resolves conflicts, and writes final claims.

Subagents do not recursively spawn subagents.
