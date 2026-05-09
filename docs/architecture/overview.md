# System Overview

The system has five layers.

## 1. Runtime Skills

Skills are Markdown instruction packs under `skills/<skill-name>/SKILL.md`.
They are loaded by Codex when the user names a skill or the task matches the
skill description.

New or changed skills:

- `deep-researcher`: deep cited research across Codex web, Context7 API,
  GitHub, source, rendered pages, Firecrawl, and evidence ledgers.
- `subagent-creator`: custom Codex subagent authoring and install workflow.
- `subspawn`: delegation and synthesis policy with strict wait behavior.

Skills stay concise. Long-form detail lives in `references/`, deterministic
helpers live in `scripts/`, and reusable role templates live in `templates/`.

## 2. Custom Subagent Roles

Custom subagents are standalone TOML files installed into either:

- `~/.codex/agents` for global personal roles;
- `.codex/agents` for project-scoped roles.

`subagent-creator` ships reusable templates and validates role shape. The
`deep-researcher` skill ships the Focused Six research roles:

- `deep_researcher`
- `github_researcher`
- `context7_researcher`
- `openai_docs_researcher`
- `source_validator`
- `citation_auditor`

These roles are read-only by default and must not spawn nested subagents.

## 3. Delegation Policy

`subspawn` owns the policy for delegation:

- spawn only when explicitly requested or when the user asks for subagents;
- keep subtasks bounded and independent;
- provide a strict spawn contract;
- after spawning a batch, immediately wait for all spawned agents;
- synthesize all results before doing substantive follow-up work.

The main Codex session remains the decision maker. Subagents provide evidence
and bounded analysis.

## 4. Research CLI

`codex-research` is the Rust CLI that makes research repeatable:

- provider-aware research plans;
- source routing and direct fetch probes;
- Context7 REST API calls;
- GitHub REST/`gh` fallback calls;
- Firecrawl scrape fallback;
- SQLite and content-addressed cache initialization;
- JSONL claim/source ledgers;
- Markdown reports;
- doctor and eval checks.

The CLI does not replace Codex-native tools. It complements them.

## 5. Development Operating Layer

`codex-dev` is the planned development control-plane family. It owns task
capsules, thin policy-gate orchestration, PR/eval/bootstrap evidence appenders,
and stable JSON contracts for optional consumers such as a Ratatui workbench.

It does not replace `codex-research`, `subagent-creator`, `subspawn`, or
`gh-pr-review-fix`. It records those tools' outputs as development evidence and
keeps one canonical capsule per goal or branch lane.

See [codex-dev Operating Layer](../specs/codex-dev-operating-layer.md) for the
schema, branch graph, ownership map, and validation expectations.

## Dual-Plane Research

Codex-native plane:

- `web.search_query`, `web.open`, `web.find`;
- GitHub app/plugin;
- Context7 MCP;
- Exa MCP;
- `$opensrc`;
- normal repo shell and file inspection.

CLI plane:

- direct Context7 REST;
- GitHub REST/`gh`;
- Firecrawl;
- direct fetch probes;
- evidence ledgers and reports;
- cache and route metadata;
- repeatable command output.

This split matters because some Codex tools are not external APIs a CLI can
invoke. The subagent uses Codex tools; the CLI records provider evidence and
handles external calls it can own directly.

## Typical Flow

```text
User asks for deep research
  -> main Codex loads deep-researcher and subspawn
  -> codex-research plan sets profile and budgets
  -> main Codex uses native web for official/current checks
  -> focused providers hydrate evidence
  -> optional subagents investigate independent lanes
  -> parent waits for all subagents
  -> source and claim records are written
  -> report summarizes claims, confidence, citations, and residual risk
```

## Stop Rule

Do not present search snippets as settled facts. Hydrate first, cite source IDs,
and label uncertainty when provider coverage, freshness, or authority is weak.
