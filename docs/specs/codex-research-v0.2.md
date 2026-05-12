# codex-research v0.2 Follow-up Specification

Status: implemented in `feat/codex-research-v0.2-spec`

Branch: `feat/codex-research-v0.2-spec`

Target release type: pre-1.0 SemVer patch via Conventional Commit scope `feat(codex-research)`

## Purpose

This spec defines the next implementation wave for the research and subagent
stack after the initial merge. The goal is not to add the widest possible
provider surface. The goal is to make `codex-research`, `deep-researcher`,
`subagent-creator`, and `subspawn` reliable enough that a Codex session can use
them repeatedly for high-stakes research without duplicated work, accidental
external disclosure, unbounded provider usage, or weak evidence.

The selected direction is a v0.2 CLI hardening release:

- enforce research profiles and provider budgets through reusable config and
  run state;
- add GitHub hydration endpoints that turn search hits into citable evidence;
- persist normalized source metadata for all providers without default raw
  private-content archiving;
- make privacy policy enforceable, especially for Exa, Firecrawl, rendered
  fetches, and other external providers;
- add hermetic tests and optional live-provider smokes;
- tighten existing research subagent templates around evidence contracts
  instead of expanding the agent catalog.

## Research Basis

This spec is grounded in current repo state and current upstream documentation.

Primary upstream facts used:

- OpenAI Codex subagents are explicit opt-in workflows. Good prompts should
  describe how to divide work, whether to wait for agents, and what output to
  return. Codex docs also say subagents are useful for read-heavy exploration,
  tests, triage, and summarization, while write-heavy parallelism needs care.
  Source: <https://developers.openai.com/codex/concepts/subagents>
- Codex custom agents are TOML files under `~/.codex/agents/` or
  `.codex/agents/` and require `name`, `description`, and
  `developer_instructions`; optional fields include model, reasoning effort,
  sandbox, MCP servers, and skill config. Source:
  <https://developers.openai.com/codex/subagents>
- GitHub REST search has search-specific rate limits, query limitations,
  default-branch and file-size constraints for code search, and
  `incomplete_results` on timed-out searches. Search hits must be hydrated
  before they are used as evidence. Source:
  <https://docs.github.com/en/rest/search/search>
- GitHub compare returns changed-file details, statuses, and patches and
  supports `BASE...HEAD` path semantics with pagination. Source:
  <https://docs.github.com/en/rest/commits/commits#compare-two-commits>
- GitHub contents API supports file and directory reads at a ref and requires
  contents read permission for private resources, while public resources can be
  fetched without authentication. Source:
  <https://docs.github.com/en/rest/repos/contents>
- GitHub REST authenticated requests have much higher primary limits than
  unauthenticated requests, and secondary limits require backoff rather than
  retry loops. Source:
  <https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api>
- Context7 provides search, context, and refresh APIs; recommends detailed
  natural-language queries, response caching for hours or days, version-pinned
  library IDs, and `Retry-After` handling on 429. Source:
  <https://context7.com/docs/api-guide>
- Firecrawl `scrape` supports `maxAge` and cache controls, while Firecrawl
  rate limits are team-scoped and measured in requests per minute. Sources:
  <https://docs.firecrawl.dev/api-reference/endpoint/scrape> and
  <https://docs.firecrawl.dev/rate-limits>

Operational risk notes:

- A current `openai/codex` issue reports that tool-backed sessions may not
  expose named custom agents even when `.codex/agents/*.toml` files exist.
  Source: <https://github.com/openai/codex/issues/15250>
- A recent `openai/codex` subagent issue described spawn slot leakage unless
  completed agents are explicitly closed. Even though the issue is currently
  closed upstream, the local `subspawn` policy should continue to require
  wait-and-close discipline. Source:
  <https://github.com/openai/codex/issues/18335>
- Another current `openai/codex` issue reports long-lived multi-agent sessions
  accumulating stdio MCP launcher processes. This reinforces bounded fanout,
  strict close behavior, and no recursive subagent spawning. Source:
  <https://github.com/openai/codex/issues/14233>

## Pre-implementation Baseline Snapshot

The merged v0.1 baseline already provided:

- `codex-research` Rust CLI with `doctor`, `plan`, `search`, `fetch`,
  `context7`, `github`, `ledger`, `report`, `cache`, and `eval` commands.
- Provider readiness detection for `gh`, `agent-browser`, `ctx7`, `opensrc`,
  `CONTEXT7_API_KEY`, `FIRECRAWL_API_KEY`, `EXA_API_KEY`, `GITHUB_TOKEN`,
  `GH_TOKEN`, and `CODEX_RESEARCH_HOME`.
- Research profiles that emit budget recommendations, but do not yet enforce
  them across provider calls.
- A predictive fetch probe that classifies URLs into direct, GitHub,
  agent-browser, or Firecrawl routes.
- Direct HTTP fetch with optional content-addressed blob storage.
- Context7 search, context, and refresh commands.
- GitHub search-repos, search-code, search-issues, releases, and file commands.
- JSONL ledgers for source and claim records plus Markdown report rendering.
- SQLite cache tables for `sources`, `route_memory`, and `claims`.
- `deep-researcher` skill and Focused Six custom-agent templates.
- `subagent-creator` validation/install/smoke helpers.
- `subspawn` strict wait-before-next-work delegation policy.
- Docs portal, CLI reference, crate reference, architecture, cookbooks,
  runbooks, prompt library, and validation docs.

Gaps addressed by this implementation:

- `ProviderBudgets` are advisory plan output only.
- `route_memory` exists but is not yet a learned input from provider outcomes.
- Source cache rows are mostly direct-fetch centric; Context7, GitHub, and
  Firecrawl do not consistently emit durable source metadata.
- GitHub search commands do not yet hydrate issues, PRs, changed files,
  compare ranges, release-by-tag records, or tags.
- External provider privacy is mostly policy text, not enforced by the CLI.
- `eval --live` is readiness oriented and does not yet exercise provider
  parsers against controlled fixtures or optional live calls.
- Subagent templates define useful return sections, but the format is not yet
  validated as a stable evidence contract.

## Decision Record

Decision scoring uses the repo's decision framework:

| Criterion | Weight |
| --- | ---: |
| Solution leverage | 35% |
| Application value | 30% |
| Maintenance and cognitive load | 25% |
| Architectural adaptability | 10% |

| Decision | Selected Option | Score | Rejected Alternatives |
| --- | --- | ---: | --- |
| v0.2 scope | CLI Core | 9.6 | Agent UX 8.7, Provider Expansion 8.5 |
| GitHub depth | Hydration Pack | 9.7 | Search Plus Compare 8.6, Full Ops 8.4 |
| Cache model | Provider Source Cache | 9.6 | Route Memory Only 8.4, Full Archive 7.8 |
| Config and budgets | Profile Config | 9.4 | Flags Only 8.2, Docs Policy 7.9 |
| Privacy default | Deny Private | 9.7 | Domain Allowlist 8.8, Warn Proceed 8.0 |
| Eval strategy | Hermetic Plus Live | 9.5 | Hermetic Only 8.8, Live CI 7.6 |
| Spec artifact | Docs Spec | 9.5 | Agents Plan 8.6, GitHub Issues 8.2 |
| Template scope | Template Contracts | 9.3 | Catalog Expansion 8.4, No Template Work 7.8 |
| Delivery | Stacked Phases | 9.4 | Single PR 8.7, Mega Release 7.3 |

## Non-goals

- Do not add a broad new catalog of custom agents in v0.2.
- Do not make Firecrawl or Exa the default route for ordinary docs lookup.
- Do not send private repository content, local files, pasted proprietary
  snippets, or confidential URLs to external scraping/search providers unless
  the operator passes an explicit private-external override.
- Do not implement recursive subagent spawning.
- Do not require live provider credentials in default CI.
- Do not turn `codex-research` into a general browser automation tool. It can
  route to `agent-browser`, but it should not duplicate that tool.

## Target Architecture

### 1. Config and Research Run State

Add reusable config and per-run state.

New global option:

```bash
codex-research --config <path> <command>
```

Config precedence:

1. `--config <path>`
2. `CODEX_RESEARCH_CONFIG`
3. nearest `.codex/research/config.toml` from the current directory upward
4. `$XDG_CONFIG_HOME/codex-research/config.toml`
5. built-in defaults

New commands:

```bash
codex-research config init [--path <path>] [--force]
codex-research config show [--json]
codex-research run init "query" --profile deep --topic github --out .codex/research/run.json
codex-research run status --run .codex/research/run.json
codex-research run debit --run .codex/research/run.json --provider github --count 1 --note "search issues"
codex-research run close --run .codex/research/run.json
```

Provider commands should accept:

```bash
--run .codex/research/run.json
--no-budget
```

Behavior:

- `run init` materializes the effective profile, provider order, policy, and
  starting budgets.
- Provider commands with `--run` serialize run-state writes with a lockfile and
  atomic rename, then debit the matching provider budget before network calls.
- If a budget would go negative, fail before calling the provider unless
  `--no-budget` is set.
- Codex-native web calls cannot be automatically intercepted; the skill docs
  must instruct the parent agent to call
  `run debit --run .codex/research/run.json --provider codex-web` whenever
  native web searches are used as part of the same run.
- `run status --json` must expose remaining calls, spent calls, last provider
  errors, and source counts.
- `run close` marks the run final and refuses further debits unless
  `--reopen` is introduced in a later release.

Initial config shape:

```toml
[profiles.standard]
codex_web_queries = 4
context7_calls = 3
github_calls = 4
exa_calls = 2
direct_fetches = 8
browser_fetches = 2
firecrawl_calls = 1

[profiles.deep]
codex_web_queries = 8
context7_calls = 4
github_calls = 8
exa_calls = 4
direct_fetches = 12
browser_fetches = 4
firecrawl_calls = 6

[privacy]
private_external_default = "deny"
ambiguous_external_default = "deny"
allow_private_external = false
redact_query_secrets = true

[providers.github]
per_page_default = 10
per_page_max = 100
backoff_retries = 2

[providers.context7]
cache_ttl_hours = 168
prefer_version_pinned_ids = true

[providers.firecrawl]
default_max_age_ms = 172800000
latest_critical_max_age_ms = 0
store_in_cache_default = true

[cache]
source_metadata_ttl_hours = 168
store_raw_external_default = false
```

### 2. Enforced Privacy Policy

Add a small privacy classifier used before Exa, Firecrawl, and rendered
external routes.

Classifications:

- `public`: safe to use public external providers.
- `sensitive-public`: public URL, but use cache/storage minimization.
- `private-or-authenticated`: private repo, session URL, localhost, signed URL,
  intranet host, local file, user-provided secret-bearing URL, or content that
  requires authenticated browser/session state.
- `ambiguous`: insufficient evidence.

Default behavior:

- Direct fetch and GitHub authenticated APIs may handle private resources
  because they are first-party or user-authenticated routes.
- Context7 is for documentation/library IDs and should not receive private
  code snippets.
- Firecrawl, Exa, and external rendered providers refuse
  `private-or-authenticated` and `ambiguous` inputs unless
  `--allow-private-external` is passed.
- Firecrawl defaults to normal cache for public docs, `--fresh` for
  latest-critical pages, and `--no-store-in-cache` for sensitive-public pages.
- The CLI should redact common token query parameters in logs and source
  metadata.

Required flags:

```bash
--allow-private-external
--privacy public|sensitive-public|private-or-authenticated|ambiguous
```

The explicit `--privacy` flag is for operator override when automated
classification is too conservative.

### 3. Provider Source Cache

Extend the SQLite cache from direct-fetch-only behavior to normalized source
records for every provider.

Source record fields:

- `id`
- `provider`
- `route`
- `url`
- `canonical_url`
- `title`
- `fetched_at`
- `freshness_status`: `current`, `stale`, `redirected`, `unavailable`,
  `unverified`
- `privacy_classification`
- `status`
- `content_hash`
- `raw_body_stored`: boolean
- `metadata_json`

Behavior:

- Provider commands should emit a `source_id` in JSON output when source
  metadata is stored.
- Raw bodies are stored only when explicitly requested or when the route is
  direct fetch with `--store`.
- GitHub, Context7, and Firecrawl store normalized metadata by default but do
  not archive raw private response bodies by default.
- `ledger add-source` should accept `--from-cache <source-id>`.
- `report` should include cache-backed source metadata when the ledger points
  at cached source IDs.

New cache commands:

```bash
codex-research cache sources [--provider github] [--limit 20] [--json]
codex-research cache source <source-id> [--json]
codex-research cache route-memory [--domain example.com] [--json]
codex-research cache prune --older-than-days 30 [--dry-run]
```

Migration rule:

- Add schema migrations with a `schema_migrations` table rather than relying on
  one monolithic `create table if not exists` batch forever.

### 4. Learned Route Memory

Use route outcomes to influence future `fetch probe` and `search` advice.

Record:

- domain
- path pattern or host-level fallback
- route attempted
- success/failure
- status class
- reason
- updated_at

Update rules:

- Direct text/markdown success increments direct route confidence.
- App-shell or low-text-density failure increments browser/Firecrawl
  preference only for public or explicitly allowed external URLs.
- Firecrawl success should not override privacy constraints.
- GitHub HTML URLs should continue to prefer GitHub API hydration.

`fetch probe --json` should include:

- route-memory hits used;
- current route recommendation;
- why a more expensive route was or was not selected.

### 5. GitHub Hydration Pack

Add GitHub commands that hydrate search leads into citable source material.

Required commands:

```bash
codex-research github compare <repo> <base> <head> [--per-page 100] [--page 1]
codex-research github tags <repo> [--per-page 30]
codex-research github release <repo> --tag <tag>
codex-research github release <repo> --latest
codex-research github issue <repo> <number> [--comments]
codex-research github pr <repo> <number> [--files] [--comments] [--reviews]
codex-research github file <repo> <path> --ref <ref>
```

Output requirements:

- All commands support `--json`.
- JSON output includes `source_id` when source metadata is stored.
- Search commands surface `incomplete_results`, rate-limit headers when
  present, and search constraints in a `limitations` field.
- `compare` includes changed filenames, statuses, additions/deletions, patch
  presence, and pagination metadata.
- `pr --files` includes filename, status, previous filename, additions,
  deletions, changes, and patch presence.
- `issue --comments` and `pr --comments` preserve author, created/updated time,
  body excerpt metadata, and API URLs, without over-quoting in rendered reports.

Auth order stays:

1. `GITHUB_TOKEN`
2. `GH_TOKEN`
3. `gh auth token`
4. unauthenticated public mode

### 6. Context7 Improvements

Keep Context7 focused and bounded.

Required changes:

- Add source-cache metadata for `context7 search`, `context7 context`, and
  `context7 refresh`.
- Surface 202, 301 redirects, 429 `Retry-After`, 503, and 504 in structured
  error output.
- Add config-driven `cache_ttl_hours` for Context7 metadata.
- Add `--version <version>` helper on `context7 search` that suggests
  `/owner/repo/<version>` and `/owner/repo@<version>` forms when applicable.
- Preserve the existing decision not to depend on removed Context7 CLI research
  mode as a core behavior.

### 7. Firecrawl and Rendered Route Improvements

Required changes:

- Enforce the privacy classifier before `fetch firecrawl`.
- Accept `--privacy` and `--allow-private-external`.
- Store Firecrawl source metadata with cache policy, `maxAge`, and
  `store_in_cache` fields.
- Add structured handling for 429 with `Retry-After`.
- Add `--formats markdown,json` only if implementation remains simple and the
  docs can keep the command clear. Otherwise defer multi-format scrape to v0.3.

### 8. Subagent Template Contracts

Tighten existing templates. Do not expand the catalog in v0.2.

Templates to update:

- `skills/deep-researcher/templates/agents/deep_researcher.toml`
- `skills/deep-researcher/templates/agents/github_researcher.toml`
- `skills/deep-researcher/templates/agents/context7_researcher.toml`
- `skills/deep-researcher/templates/agents/openai_docs_researcher.toml`
- `skills/deep-researcher/templates/agents/source_validator.toml`
- `skills/deep-researcher/templates/agents/citation_auditor.toml`
- mirrored templates under `skills/subagent-creator/templates/agents/` when
  roles overlap

Contract additions:

- Return `Status` as one of: `complete`, `partial`, `blocked`.
- Include `Sources hydrated` with source IDs when available.
- Include `Claims` with confidence and source IDs.
- Include `Provider limits` when searches are incomplete, rate-limited,
  unauthenticated, default-branch-only, stale, redirected, or otherwise weak.
- Include `Privacy notes` when content was intentionally not sent to external
  providers.
- Include `Recommended next verification`.
- Preserve `Do not edit files`, `Do not spawn nested subagents`, and parent
  prompt authority.

Validation:

- Extend `subagent_creator.py validate` or add a focused check so the research
  templates must include the required return headings.
- Keep smoke tests offline and deterministic.

### 9. Skill and Docs Updates

Required docs updates in the implementation wave:

- `docs/reference/codex-research-cli.md`: new commands, flags, examples, exit
  behavior.
- `docs/reference/codex-research-crate.md`: config, run state, cache schema,
  migrations, provider modules.
- `docs/architecture/research-system.md`: enforced budgets, privacy
  classifier, route memory, GitHub hydration, source cache.
- `docs/cookbooks/deep-research-workflow.md`: run init, budget debit, ledger,
  report.
- `docs/cookbooks/github-archaeology.md`: compare/tags/release/PR hydration.
- `docs/cookbooks/context7-source-validation.md`: version-pinned Context7 and
  source validation pattern.
- `docs/cookbooks/evidence-ledgers.md`: cache-backed source IDs and claim
  confidence.
- `docs/runbooks/validation.md`: hermetic plus live evals.
- `docs/runbooks/troubleshooting.md`: provider budgets, privacy refusals,
  GitHub 403/429/secondary limits, Context7 redirects/429, Firecrawl 429.
- `docs/prompts/codex-scenario-prompts.md`: updated operator prompts using
  `run init`, `run debit`, GitHub hydration, and strict subspawn synthesis.
- `skills/deep-researcher/SKILL.md`: explain run-state budget use and source
  cache expectations.
- `skills/deep-researcher/references/runbook.md`: step-by-step updated deep
  research workflow.
- `skills/subspawn/SKILL.md`: keep strict immediate wait and close discipline;
  add a reminder to close completed agent threads when runtime tools expose
  explicit close.

## Delivery Plan

### P0: Spec and Planning

Commit scope:

```text
docs(research): specify codex-research v0.2 follow-up plan
```

Files:

- `docs/specs/codex-research-v0.2.md`
- `docs/index.md`
- `README.md`

Acceptance:

- Spec linked from docs portal and README.
- No feature implementation in this branch.
- Link checker passes.
- Git diff check passes.

### P1: Config, Run State, Privacy, and Cache Migrations

Commit scopes:

```text
feat(codex-research): add profile config and run budgets
feat(codex-research): enforce external provider privacy policy
feat(codex-research): add source cache migrations
```

Implementation:

- Add config loader and `config` commands.
- Add run state file and `run` commands.
- Add budget debit support to provider commands.
- Add privacy classifier and external-provider refusals.
- Add schema migrations.
- Add source cache metadata model.

Acceptance:

- Budgets can be initialized, debited, exhausted, and inspected.
- Provider commands refuse exhausted budgets when `--run` is active.
- Firecrawl/Exa/rendered external routes refuse ambiguous/private inputs by
  default.
- Migration from existing cache works without deleting user data.
- Hermetic tests cover config precedence, budget exhaustion, privacy
  classification, and migration.

### P2: GitHub Hydration and Source Cache Integration

Commit scope:

```text
feat(codex-research): add github hydration commands
```

Implementation:

- Add compare, tags, release-by-tag/latest, issue, PR, PR files/comments, and
  review hydration.
- Normalize GitHub source metadata and limitations.
- Preserve existing search and file commands.
- Surface rate-limit and incomplete-result evidence.

Acceptance:

- Fixture-backed tests cover search limitations, compare file parsing, tags,
  release-by-tag, issue comments, PR files, and PR comments.
- JSON output includes source IDs and limitations.
- CLI reference examples match actual help.

### P3: Context7, Firecrawl, Route Memory, and Reporting

Commit scopes:

```text
feat(codex-research): persist provider source metadata
feat(codex-research): learn route memory from provider outcomes
```

Implementation:

- Store source metadata for Context7 and Firecrawl commands.
- Record route-memory outcomes from probe/get/firecrawl.
- Make route-memory influence future probe output.
- Let `ledger add-source --from-cache` and reports use cached source metadata.

Acceptance:

- Route memory records successes and failures in hermetic tests.
- Probe JSON explains route-memory influence.
- Reports render cache-backed source metadata.
- Context7 structured errors include redirect and retry details in fixtures.
- Firecrawl structured errors include retry details in fixtures.

### P4: Template Contracts, Docs, and Evals

Commit scopes:

```text
feat(research-agents): enforce evidence return contracts
docs(research): update v0.2 operations guides
test(codex-research): add hermetic provider fixtures
```

Implementation:

- Update research agent templates and overlapping subagent-creator templates.
- Extend template validation for required return headings.
- Update skills and docs listed above.
- Add `eval --live` optional provider smokes gated by environment.

Acceptance:

- Template validation fails if required evidence headings are missing.
- Default tests do not require network or secrets.
- `codex-research eval` remains hermetic.
- `codex-research eval --live` runs only configured low-cost checks and reports
  skipped providers clearly.

## Verification Matrix

Required before opening the implementation PR:

```bash
cargo fmt --all --check
cargo clippy -p codex-research --all-targets -- -D warnings
cargo check -p codex-research
cargo test -p codex-research
codex-research --json doctor
codex-research --json eval
python3 -m compileall -q skills tools subagents/hardened-codex/scripts
python3 tools/docs/check_links.py docs README.md AGENTS.md
python3 tools/eval/skill_subagent_eval.py --json
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents skills/subspawn/templates/agents subagents/hardened-codex/agents
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
git diff --check
```

Optional release smoke when credentials are configured:

```bash
codex-research --json eval --live
codex-research --json run init "GitHub hydration smoke" --profile quick --topic github --out .codex/research/run.json
codex-research --json github release openai/codex --latest --run .codex/research/run.json
codex-research --json run status --run .codex/research/run.json
```

The optional smoke should be skipped in CI unless secrets and rate-limit policy
are explicitly configured.

## Implementation Guardrails

- Keep diffs narrow by phase. Do not mix feature behavior, docs, and template
  contract changes in one commit unless the diff is too small to split.
- Preserve current command names and JSON shape unless the spec explicitly adds
  fields.
- Treat existing docs examples as tests: update docs when help/output changes.
- Do not store raw private provider bodies by default.
- Do not add dependencies unless they materially simplify the implementation or
  test harness.
- Prefer structured parsers and typed structs over ad hoc string extraction.
- Continue using `gh auth token` fallback for local GitHub auth.
- Keep subagents read-only for research roles.
- Require parent-agent synthesis after all subagents finish.

## Open Questions Deferred Until Implementation

These are deliberately deferred because they do not change the approved design:

- Whether route-memory path patterns should be exact path prefixes or a small
  host-level classifier in v0.2. Start host-level unless tests prove path-level
  value.
- Whether Firecrawl multi-format output belongs in v0.2 or v0.3. Add only if
  it does not complicate privacy or report rendering.
- Whether live evals should support a paid-provider cost cap. Start with
  skipped-by-default smokes and add cost accounting only if live checks become
  broader.
- Whether a future v0.3 should add a portable MCP agent-definition bridge.
  Current Codex runtime mismatch makes this risky for v0.2.

## Approval Criteria

This spec is ready for implementation when the user approves:

- CLI Core as the v0.2 scope;
- GitHub Hydration Pack;
- Provider Source Cache;
- Profile Config and run-state budgets;
- Deny Private external-provider defaults;
- Hermetic Plus Live eval strategy;
- Template Contracts rather than catalog expansion;
- Stacked Phases delivery.

After approval, start implementation with P1 on a feature branch or continue
from the approved branch if the PR strategy calls for a single stacked branch.
