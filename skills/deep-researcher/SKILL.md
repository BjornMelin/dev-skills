---
name: deep-researcher
description: Deep, cited research across official docs, Codex web tools, Context7 API, GitHub, package source, rendered web pages, and Firecrawl with evidence ledgers.
---

# Deep Researcher

Use this skill when a task needs defensible, current, multi-source research:
library/API decisions, dependency investigations, release or changelog analysis,
GitHub issue/source archaeology, agent-prompt research, standards checks, or
high-stakes citations.

## Operating Model

Use a dual-plane design:

- Codex-native plane: use `web.search_query`, `web.open`, `web.find`, GitHub
  app tools, Context7 MCP when explicitly useful, Exa MCP, and `$opensrc` from
  the session. These tools are available to Codex, not to the Rust CLI.
- CLI plane: use `codex-research` for provider routing, Context7 REST API,
  GitHub REST/`gh` fallback, Firecrawl calls, direct fetch probes, SQLite/CAS
  cache state, JSONL ledgers, Markdown reports, doctor output, and evals.

Always treat search results as leads until hydrated into source records. A
claim is not ready to rely on until it is tied to source IDs, source freshness,
and a confidence score.

## Source Routing

Default order for broad research:

1. Native Codex web for narrow current facts, official docs, and quick source
   confirmation.
2. Context7 REST API through `codex-research context7` for version-aware
   library/API docs.
3. GitHub app or `codex-research github` for repository, code, issue, PR,
   release, tag, compare, manifest, and changelog evidence.
4. Direct fetch through `codex-research fetch probe|get` for text/static pages.
5. Exa for broad semantic discovery, repository inspiration, filtered web/GitHub
   exploration, and source expansion when native web is too narrow.
6. `agent-browser` or Firecrawl only when route prediction says direct fetch is
   likely an app shell, blocked, rendered, or crawl-heavy page.
7. `$opensrc` when package implementation source is required.

Use `codex-research plan "<query>" --profile quick|standard|deep|exhaustive`
to inspect call budgets before broad research. For replayable runs, initialize
run state and pass `--run` to provider commands:

```bash
codex-research run init "<query>" --profile deep --topic github --out .codex/research/run.json
codex-research run status --run .codex/research/run.json
```

Native Codex web calls are not visible to the CLI. Debit them manually with
`codex-research run debit --run .codex/research/run.json --provider codex-web`
when they are part of the same budgeted run.

## Firecrawl Policy

Firecrawl is a paid-capacity fallback lane, not the first source.

- Public docs: cache is allowed unless the task is latest-critical.
- Latest-critical pages: use `--fresh` so `maxAge=0`.
- Sensitive public pages: set `--no-store-in-cache`.
- Private/confidential/repo-proprietary content: do not send to Firecrawl unless
  the user explicitly allows external scraping for that material.
- If a page can be hydrated through GitHub, Context7, official docs, or direct
  fetch, prefer that before Firecrawl.

## GitHub Policy

Use hybrid GitHub access:

- In Codex sessions, prefer the GitHub app/plugin for PRs, private repos, review
  threads, workflow logs, and authenticated repository metadata.
- Use `codex-research github` for standalone, replayable, and reportable REST
  calls. It falls back through `gh auth token`, `GITHUB_TOKEN`, `GH_TOKEN`, then
  public unauthenticated mode.
- Target and hydrate. Generate narrow query shards, respect search result and
  rate limits, then fetch full files, issue threads, releases, or compare ranges
  before citing.
- Escalate to clone/sparse checkout and local `rg` only when API search cannot
  prove the source-level claim.

## Subagent Orchestration

When using subagents, follow `$subspawn` strict rendezvous behavior. The main
Codex session spawns specialized research subagents, immediately waits for all
spawned agents in the batch, then synthesizes. Research subagents must not spawn
nested subagents.

Initial focused pack:

- `deep_researcher`: lead multi-source researcher and synthesis owner.
- `github_researcher`: GitHub repository/code/issues/releases specialist.
- `context7_researcher`: direct Context7 API docs specialist.
- `openai_docs_researcher`: official OpenAI docs specialist.
- `source_validator`: package/source/release implementation validator.
- `citation_auditor`: claim-to-source and freshness auditor.

Install templates with:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target project
python3 skills/deep-researcher/scripts/install_agents.py --target global
```

## Evidence Bundles

For meaningful research, produce both machine and human outputs:

- JSONL ledger: `.codex/research/ledger.jsonl`
- source records JSON or cached source metadata
- route stats or cache stats when routing mattered
- Markdown report with concise claims and citations

Useful commands:

```bash
codex-research doctor
codex-research cache init
codex-research plan "research question" --profile deep
codex-research run init "research question" --profile deep --topic general --out .codex/research/run.json
codex-research fetch probe "https://example.com/docs"
codex-research context7 search --library "Next.js" --query "middleware auth"
codex-research github search-code 'repo:owner/repo symbol in:file'
codex-research ledger init
codex-research ledger add-source --from-cache <source-id>
codex-research report --ledger .codex/research/ledger.jsonl
codex-research eval
```

## Stop Rules

Stop and mark `UNVERIFIED` when:

- sources disagree and you cannot identify the current authority;
- a required provider is rate-limited or missing credentials;
- only stale secondary sources are available;
- private material would need to be sent to an external provider without
  explicit permission;
- GitHub search is incomplete and hydration cannot validate the claim.
