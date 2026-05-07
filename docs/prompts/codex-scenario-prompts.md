# Codex Scenario Prompts

These prompts are meant to be pasted into Codex sessions. Adjust repository
names, package names, and scope before running.

## Deep Research With Evidence Bundle

```text
Use $deep-researcher $codex-utils $opensrc and GitHub/Context7/Exa tools as needed.

Research: <question>.

Requirements:
- Start with codex-research doctor and codex-research plan --profile deep.
- Initialize .codex/research/run.json and pass --run to provider commands.
- Debit native Codex web calls with codex-research run debit --provider codex-web.
- Prefer Codex native web for official/current facts.
- Use Context7 direct API for library docs when relevant.
- Use GitHub app or codex-research github for repository/source/issues/releases.
- Use opensrc for package implementation proof when behavior matters.
- Use fetch probe before browser or Firecrawl escalation.
- Do not send private or ambiguous content to external providers unless I explicitly allow it.
- Create a .codex/research/ledger.jsonl with source and claim records.
- Render .codex/research/report.md.
- Final answer must separate confirmed facts, recommendation, conflicts, and residual risk.
```

## GitHub Archaeology

```text
Use $deep-researcher and GitHub tools.

Investigate the GitHub history and current source for <repo/package/behavior>.

Scope:
- Search repositories, code, issues/PRs, releases, tags, changelogs, and package manifests.
- Treat GitHub search hits as leads only.
- Hydrate top hits through codex-research github issue/pr/compare/tags/release/file, GitHub APIs, raw files, or local clone when needed.
- Report incomplete search results, rate limits, stale issues, and source refs.

Deliver:
- Source-backed timeline.
- Current behavior.
- Breaking changes or migration notes.
- Confidence per major claim.
```

## Context7 Plus Source Validation

```text
Use $deep-researcher $opensrc and Context7.

Verify the current docs and implementation for <library/API/feature>.

Steps:
- Use codex-research context7 search/context for docs and keep returned source IDs.
- Use a version-pinned Context7 library ID when the target repo pins a version.
- If latest-critical, consider refresh and verify with another primary source.
- Inspect package source with opensrc or GitHub hydrated files.
- Record conflicts between docs and implementation.

Final answer:
- Docs-supported behavior.
- Source-confirmed behavior.
- Any docs/source drift.
- Recommended implementation posture.
```

## Official OpenAI Docs Research

```text
Use $deep-researcher and $openai-docs.

Research the latest official OpenAI guidance for <topic>.

Constraints:
- Use only official OpenAI docs, official OpenAI GitHub/cookbook sources, and current Codex runtime/tool schemas unless I explicitly ask for third-party comparisons.
- Verify current model/tool guidance because availability changes.
- Label anything inferred from runtime behavior as inference.

Deliver:
- Current official guidance.
- Deprecated or changed guidance.
- Practical implementation rules for this repo.
- Links/sources.
```

## Subagent Fanout Research

```text
Use $subspawn $deep-researcher.

First generate the orchestration plan:
python3 skills/subspawn/scripts/subspawn_plan.py plan --preset research --task "<question>" --scope "official docs, GitHub source, releases, and cited evidence only"

Spawn exactly three read-only subagents and wait for all before doing any other substantive work:
1. openai_docs_researcher: official docs lane for <question>.
2. github_researcher: GitHub/source/release lane for <question>.
3. citation_auditor: audit the evidence quality after the other findings are available; if it starts before they are ready, audit the parent-provided sources only.

Use the generated spawn prompts as the base contract. Each subagent prompt must include scope, read-only mode, strict wait expectation, role, inherited model/effort, and return sections:
- Status
- Sources hydrated
- Claims with confidence and source IDs
- Provider limits
- Privacy notes
- Recommended next verification
- Risks/blockers

After all complete, synthesize conflicts and produce a claim/source ledger.
```

## Build a New Custom Subagent

```text
Use $subagent-creator $skill-creator if needed.

Create a new Codex custom subagent for <role>.

Requirements:
- Do not shadow built-ins: default, worker, explorer.
- Use snake_case role name and matching filename.
- Least-privilege sandbox.
- No nested subagents by default.
- Parent prompt remains authority.
- Redact secrets.
- Stable return sections.
- Validate with subagent_creator.py validate.
- If installing, dry-run first and back up any overwritten role.
```

## Audit Existing Subagent Templates

```text
Use $subagent-creator $subspawn.

Audit the installed custom subagents under <path>.

Check:
- TOML validity.
- Built-in shadowing.
- Model/effort fit.
- Sandbox privilege.
- Prompt scope.
- Return contract.
- Secret redaction.
- Nested subagent prohibition.
- Compatibility with subspawn strict rendezvous.

Run:
python3 skills/subagent-creator/scripts/subagent_creator.py validate <path>
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subagent-creator/scripts/subagent_creator.py diff --target global --include-extra

Return prioritized fixes and exact files to edit.
```

## Rendered Docs Page Investigation

```text
Use $deep-researcher and browser/Firecrawl tools only when needed.

Investigate <docs URL>.

Steps:
- Run codex-research fetch probe <url>.
- If route is direct, use direct fetch or native Codex web first.
- If route is agent-browser, use local browser extraction before Firecrawl.
- If route is firecrawl, confirm content is public and allowed by policy; use --fresh when latest-critical.
- If Firecrawl refuses private or ambiguous content, use direct/GitHub/Context7/local browser routes instead unless I explicitly approve --allow-private-external.
- Record which route was chosen and why.

Deliver:
- Extracted evidence.
- Route decision.
- Confidence and freshness.
- Any provider limitations.
```

## Validate This Repo After Docs and Skill Changes

```text
Run the repo-native validation for dev-skills changes:

cargo fmt --all --check
cargo check -p codex-research
cargo clippy -p codex-research --all-targets -- -D warnings
cargo test -p codex-research
codex-research --json doctor
codex-research --json eval
tmp=$(mktemp -d)
codex-research --json run init validation-smoke --profile quick --topic github --out "$tmp/run.json"
codex-research --json run debit --run "$tmp/run.json" --provider github --count 1 --note validation
python3 -m compileall -q skills/deep-researcher/scripts skills/subagent-creator/scripts skills/subspawn/scripts
for d in skills/*; do [ -f "$d/SKILL.md" ] && python3 tools/skill/quick_validate.py "$d"; done
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/deep-researcher/templates/agents skills/subagent-creator/templates/agents
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
python3 skills/subspawn/scripts/subspawn_plan.py plan --preset research --task "validation smoke" --scope "docs and template metadata" --json
git diff --check

Report exact command outcomes and residual risk.
```
