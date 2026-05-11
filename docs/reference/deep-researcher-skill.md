# Deep Researcher Skill Reference

Path:

```text
skills/deep-researcher/
```

Purpose: deep, cited, current research across official docs, Codex web tools,
Context7 API, GitHub, package source, rendered pages, Firecrawl, and evidence
ledgers.

For Focused Six template ownership, duplicate-role expectations, and packaged
fallback rules, see [Subagent Templates](subagent-templates.md). This reference
covers the deep-researcher skill itself.

## Files

```text
skills/deep-researcher/
  SKILL.md
  agents/openai.yaml
  references/architecture.md
  references/runbook.md
  scripts/codex-research
  scripts/install_agents.py
  templates/agents/*.toml
```

## When to Use

Use for:

- library/API decisions;
- dependency investigations;
- release/changelog analysis;
- GitHub source archaeology;
- standards or security-sensitive research;
- agent-prompt and subagent-design research;
- any answer that needs citations and current facts.

Do not use for:

- simple stable facts;
- code edits that only need local repo inspection;
- private-content scraping through external providers without explicit approval.

## Operating Contract

The skill uses a dual-plane model:

- Codex plane: native web tools, GitHub app, Context7 MCP, Exa MCP, opensrc.
- CLI plane: `codex-research` for provider calls it can own, evidence ledgers,
  cache, route decisions, reports, doctor, and evals.

Core rule:

```text
Search results are leads. Hydrated source records are evidence.
```

## Focused Six Agent Pack

`skills/deep-researcher/templates/agents/` is the canonical source for the
Focused Six research roles. The installer below copies only those roles. Use
`subagent_creator.py` when you need generic packs, installed-role drift checks,
sync backups, pruning, or cross-directory template validation.

Install:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target project
python3 skills/deep-researcher/scripts/install_agents.py --target global
```

Validate:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate ~/.codex/agents
```

Roles:

| Role | Purpose |
| --- | --- |
| `deep_researcher` | lead multi-source researcher and synthesis owner |
| `github_researcher` | repository, code, issues, PRs, releases, changelogs |
| `context7_researcher` | direct Context7 REST docs research |
| `openai_docs_researcher` | official OpenAI docs and Codex docs |
| `source_validator` | package/source/release implementation proof |
| `citation_auditor` | claim-to-source, freshness, and confidence audit |

These agents are read-only and must not spawn nested subagents.

## Wrapper Script

`scripts/codex-research` finds or runs the CLI:

1. `CODEX_RESEARCH_BIN`, if set.
2. `codex-research` from `PATH`.
3. `./target/debug/codex-research`.
4. `cargo run -q -p codex-research -- ...` when run from this repo.

Use the wrapper from skill-relative contexts where PATH may not include the
installed binary.

## Evidence Bundle

Default research artifact layout:

```text
.codex/research/
  ledger.jsonl
  report.md
```

Global CLI cache:

```text
~/.cache/codex-research/
  research.sqlite
  blobs/
```

## Best Practices

- Run `codex-research plan` before broad research.
- Keep provider calls proportional to profile: `quick`, `standard`, `deep`, or
  `exhaustive`.
- Use `fetch probe` before deciding on direct/browser/Firecrawl extraction.
- Prefer GitHub APIs over scraping GitHub HTML.
- Prefer version-pinned Context7 library IDs for package docs.
- Use Firecrawl only for public rendered/blocked/crawl-heavy pages under the
  classified policy.
- Use `$subspawn` only for independent research lanes and wait for all agents.
- Mark stale or weakly sourced claims as `UNVERIFIED`.

## Validation

```bash
python3 tools/skill/quick_validate.py skills/deep-researcher
python3 -m compileall -q skills/deep-researcher/scripts
python3 skills/deep-researcher/scripts/install_agents.py --target project --project-dir /tmp/deep-researcher-smoke --dry-run
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents
```
