---
name: subagent-creator
description: Create, validate, install, and smoke-test Codex custom subagents from TOML templates. Use for ~/.codex/agents or project .codex/agents roles, subagent templates, model and sandbox defaults, or reusable agent packs.
---

# Subagent Creator

Create Codex custom agents as standalone TOML role files. Use `$subspawn` for
runtime delegation policy; use this skill to author, validate, install, and
test reusable agent roles.

## Workflow

1. Choose the target:
   - global personal roles: `~/.codex/agents`
   - project-scoped roles: `.codex/agents` under the trusted repo
   - custom destination: explicit directory passed to the helper
2. Choose templates from `templates/agents/`, a bundled pack, or create a
   narrow custom role.
3. Use `scripts/subagent_creator.py` for deterministic list, status,
   plan-sync, install, validate, render, diff, sync, prune, backup, doctor, and
   smoke workflows.
4. Validate every generated TOML file before telling the user it is ready.
5. For live runtime behavior, pair generated roles with `$subspawn` strict
   rendezvous guidance so spawned results are considered before next work.

## Helper Commands

Resolve paths relative to this skill directory.

```bash
python3 scripts/subagent_creator.py list
python3 scripts/subagent_creator.py list --packs
python3 scripts/subagent_creator.py doctor --project-dir .
python3 scripts/subagent_creator.py status --pack core --project-dir . --include-extra
python3 scripts/subagent_creator.py plan-sync --pack core --target global --include-extra
python3 scripts/subagent_creator.py install reviewer repo_explorer --target global --dry-run
python3 scripts/subagent_creator.py pack install core --target project --project-dir . --dry-run
python3 scripts/subagent_creator.py diff --pack core --target global --include-extra
python3 scripts/subagent_creator.py sync --pack core --target global --dry-run
python3 scripts/subagent_creator.py prune --pack core --target global
python3 scripts/subagent_creator.py backup --target global
python3 scripts/subagent_creator.py validate ~/.codex/agents
python3 scripts/subagent_creator.py smoke --pack docs
```

Use `--overwrite` only when replacing a role intentionally. Prefer `sync` when
updating already-installed templates because it backs up overwritten files by
default. Use `--dry-run` before writing to global or project config when the
target is unclear. Use `status --include-extra` and `plan-sync` before broad
updates. `prune` is dry-run only unless `--confirm` is present.

## Defaults

- Names: snake_case role names and matching `<name>.toml` filenames.
- Models: `gpt-5.6-terra` for bounded retrieval and mechanical inventory;
  `gpt-5.6-sol` for judgment, implementation, planning, and synthesis.
- Reasoning: `medium` by default; `high` for complex decisions; Terra `max`
  only for independent adversarial validation.
- Names: do not shadow Codex built-ins (`default`, `worker`, `explorer`) unless
  the user explicitly asks for an override. Use names such as `repo_explorer`.
- Sandbox: least privilege. Read-only for reviewer, repo exploration, docs, and
  CI triage roles; workspace-write only for roles that run tests, debug
  browsers, or implement fixes.
- Tools: inherit parent tools and MCP servers unless a role has a strong reason
  to pin a narrower config.
- Every template must forbid nested subagents by default, preserve parent
  boundaries, redact secrets, and return evidence in a stable sectioned shape.

## Template Packs

- `core`: baseline roles for routine Codex delegation.
- `docs`: generic docs, OpenAI docs, Context7, and dependency research.
- `review`: PR/code-review helper lanes with false-positive validation.
- `audit`: security, runtime, dependency, performance, and docs audit lanes.
- `ops`: CI, release, environment, and validation lanes.

Use `pack list` or `list --packs` to inspect exact membership.

Read `references/authoring-guide.md` before creating new role families,
changing model policy, or adding MCP/server-specific config.
Read `references/workflow-recipes.md` before creating PR review or audit
orchestration workflows around the bundled agents.
