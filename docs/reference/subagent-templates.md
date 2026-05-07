# Subagent Templates Reference

This reference covers the new templates added by the research and subagent
systems.

## Deep Researcher Focused Six

Located in:

```text
skills/deep-researcher/templates/agents/
```

| Template | Model | Effort | Sandbox | Use |
| --- | --- | --- | --- | --- |
| `deep_researcher.toml` | `gpt-5.5` | `high` | read-only | lead multi-source research |
| `github_researcher.toml` | `gpt-5.4-mini` | `medium` | read-only | repo/code/issues/releases |
| `context7_researcher.toml` | `gpt-5.4-mini` | `medium` | read-only | direct Context7 REST docs |
| `openai_docs_researcher.toml` | `gpt-5.5` | `medium` | read-only | official OpenAI docs |
| `source_validator.toml` | `gpt-5.4-mini` | `medium` | read-only | implementation/source proof |
| `citation_auditor.toml` | `gpt-5.4-mini` | `medium` | read-only | source freshness and claim audit |

All Focused Six roles:

- are read-only;
- forbid nested subagents;
- preserve parent scope;
- redact secrets;
- return stable sectioned evidence.

Install:

```bash
python3 skills/deep-researcher/scripts/install_agents.py --target global
python3 skills/deep-researcher/scripts/install_agents.py --target project
```

## Subagent Creator Templates

Located in:

```text
skills/subagent-creator/templates/agents/
```

Template families:

- core exploration and implementation;
- docs and dependency research;
- review and false-positive validation;
- security/runtime/performance audits;
- CI/release/environment operations.

Use `list --packs` for the exact current map:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py list --packs
```

Validate planner-visible role names and return contracts:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
```

## Role Selection Guidance

Use the narrowest role that matches the question.

Examples:

- Need official OpenAI API docs: `openai_docs_researcher`.
- Need package docs from Context7: `context7_researcher`.
- Need actual upstream source behavior: `source_validator`.
- Need GitHub issue/release/code archaeology: `github_researcher`.
- Need final evidence quality check: `citation_auditor`.
- Need broad multi-source coordination: `deep_researcher`.

Do not ask a broad lead researcher to do everything when a narrow validator can
answer the question.

## Return Format Standard

Expected sections:

```text
- Status
- Evidence
- Files inspected
- Commands run
- Findings
- Risks/blockers
```

Research-specific roles may add:

- source IDs;
- freshness notes;
- confidence adjustments;
- disagreements.

## Safety Defaults

- No nested subagents.
- No commits.
- No file edits unless the role explicitly owns implementation.
- No external scraping of private material without explicit user permission.
- Parent prompt overrides role defaults.
