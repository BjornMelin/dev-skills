# Subagent Templates Reference

This is the authority reference for custom subagent template ownership,
packaging copies, and duplicate-role validation in this repo.

Codex custom subagents are TOML role files installed under `~/.codex/agents/`
or a project `.codex/agents/` directory. Agent Skills are separate portable
skill folders with `SKILL.md` plus optional `scripts/`, `references/`,
`assets/`, and role metadata. Plugins or `.skill` bundles are the distribution
unit; they should be self-contained and redistributable.

## Authority Model

| Path | Authority | Purpose | Edit when |
| --- | --- | --- | --- |
| `skills/subagent-creator/templates/agents/` | Canonical for general reusable custom-agent packs | Author, validate, install, sync, diff, back up, prune, and smoke-test general role templates | Adding or changing reusable docs, review, audit, ops, implementation, or CI roles |
| `skills/deep-researcher/templates/agents/` | Canonical for the deep-researcher Focused Six | Ship the research-specific roles used by the deep-researcher skill and its focused installer | Changing the deep research role contract, model posture, evidence contract, or installable Focused Six set |
| `skills/subspawn/templates/agents/` | Packaged fallback copies, not a primary authoring source in a full checkout | Let the standalone `subspawn.skill` resolve preset roles without sibling skills | A `subspawn` preset depends on a role that must remain available when only the packaged subspawn skill is installed |
| `subagents/codex/agents/` | Separate global/repo overlay catalog | Global Codex baseline and repo overlay distribution, validated by Codex subagent sync tooling | Changing global defaults or repo-specific overlays |

Do not add a second copy just because a role is useful in multiple places. Add
the role to the canonical owner first, then add a fallback copy only when a
standalone packaged skill needs that role without depending on sibling skill
folders.

## Expected Duplicate Roles

`subspawn_plan.py validate-roles` intentionally reads template directories in
this order in a full repository checkout:

1. `skills/deep-researcher/templates/agents/`
2. `skills/subagent-creator/templates/agents/`
3. `skills/subspawn/templates/agents/`

The first copy wins. Later duplicate names are reported as ignored so drift is
visible, but the duplicates are not automatically a failure.

Expected duplicates are limited to one of these cases:

- a `subspawn` fallback copy mirrors a role used by a preset;
- a general `subagent-creator` pack also includes the same role name with
  pack-specific model or instruction posture for installation convenience;
- a Codex subagent overlay intentionally carries its own catalog and is
  validated by the Codex subagent release checks.

Treat a duplicate as a bug when:

- the role name appears in a new directory that is not listed in the authority
  table;
- the fallback copy changes model, sandbox, edit permission, safety posture, or
  required return sections without the canonical owner changing first;
- a convenience-pack duplicate is assumed to be planner-visible even though the
  deeper registry entry wins in `subspawn_plan.py`;
- the duplicate is not needed by a packaged standalone skill, pack installer, or
  documented overlay workflow;
- a role shadows a built-in role name such as `default`, `worker`, or
  `explorer`.

## Packaging Rules

Every skill folder should be usable from source and safe to bundle as a `.skill`
ZIP archive:

- keep the canonical entrypoint at `skills/<skill-name>/SKILL.md`;
- put long-form instructions in `references/`;
- put executable deterministic helpers in `scripts/`;
- include `templates/agents/` only when those templates are part of the skill's
  standalone behavior;
- keep generated bundles, provider dumps, local caches, secrets, and run-specific
  `.codex/research/` artifacts out of packaged skill folders;
- update README and `docs/index.md` as portals only, then keep command details
  in the reference or runbook that owns them.

`tools/skill/package_skill.py` validates `SKILL.md`, writes archive entries as
`<skill-name>/...`, and skips common generated caches such as `__pycache__`,
`*.pyc`, `*.skill`, `.codex/`, and local tool caches. It rejects output
directories nested inside the source skill folder and skips symlinks so the
bundle cannot package itself or out-of-tree targets. `quick_validate.py`
validates only `SKILL.md` frontmatter;
custom-agent TOML files are validated by
`subagent_creator.py validate`, and `agents/openai.yaml` metadata is validated
by `python3 tools/eval/skill_subagent_eval.py --json`.

When a fallback template exists for packaging, keep it intentionally small:
mirror the role contract required by the packaged skill and avoid adding
unrelated install/sync behavior to the fallback directory.

## Validation Matrix

Validate template metadata:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate \
  skills/deep-researcher/templates/agents \
  skills/subagent-creator/templates/agents \
  skills/subspawn/templates/agents \
  subagents/codex/agents
```

Validate duplicate handling and planner-visible return contracts:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
```

When fallback copies change, manually compare the fallback against the canonical
owner for model, sandbox, edit permissions, safety posture, and required return
sections. The current validators report duplicate paths and validate each file,
but they do not fail on fallback parity drift.

Validate standalone preset planning:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset research \
  --task "validation smoke" \
  --scope "docs and template metadata" \
  --json
```

If a template or packaging rule changes, also run the touched skill validators
and docs link checker from [Validation](../runbooks/validation.md).

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

`install_agents.py` installs only this Focused Six pack. Use
`subagent_creator.py` for broader pack installation, drift checks, backups, and
sync/prune workflows.

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

`skills/subagent-creator/templates/agents/` is the source of truth for the
general reusable packs. Use `skills/subspawn/templates/agents/` only for
standalone packaged fallback copies required by `subspawn` presets.

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
