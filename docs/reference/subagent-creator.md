# Subagent Creator Reference

Path:

```text
skills/subagent-creator/
```

Purpose: create, validate, install, inventory, diff, plan syncs, prune stale
installs, back up, and smoke-test Codex custom subagent TOML roles.

## Files

```text
skills/subagent-creator/
  SKILL.md
  agents/openai.yaml
  references/authoring-guide.md
  references/workflow-recipes.md
  scripts/subagent_creator.py
  templates/agents/*.toml
```

## Role Destinations

Global:

```text
~/.codex/agents/
```

Project:

```text
.codex/agents/
```

Custom destination:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py install reviewer --dest /tmp/agents
```

## Commands

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py --help
```

Supported commands:

- `list`: list bundled templates.
- `render`: print or copy templates.
- `status`: inventory template drift across global and project installs.
- `install`: install templates.
- `sync`: overwrite installed templates with backups.
- `diff`: compare bundled templates to installed roles.
- `plan-sync`: show the install/overwrite/prune plan without writing files.
- `prune`: remove stale installed roles, dry-run by default.
- `backup`: back up installed roles.
- `validate`: validate TOML roles.
- `doctor`: inspect Codex subagent environment.
- `smoke`: create a temporary smoke setup.
- `pack`: list or install template packs.

## Template Packs

| Pack | Purpose |
| --- | --- |
| `core` | routine delegation roles |
| `docs` | generic docs, OpenAI docs, Context7, dependency research |
| `review` | code-review and false-positive validation lanes |
| `audit` | security, runtime, dependency, performance, docs audits |
| `ops` | CI, release, environment, validation lanes |

Inspect exact pack membership:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py list --packs
python3 skills/subagent-creator/scripts/subagent_creator.py pack list
```

## Common Workflows

Install a core pack into the project:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py pack install core --target project --project-dir . --dry-run
python3 skills/subagent-creator/scripts/subagent_creator.py pack install core --target project --project-dir .
```

Install a single global role:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py install repo_explorer --target global --dry-run
python3 skills/subagent-creator/scripts/subagent_creator.py install repo_explorer --target global
```

Diff installed templates:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py diff --pack docs --target global --include-extra
```

Inventory global and project installs:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py status --pack core --project-dir . --include-extra
python3 skills/subagent-creator/scripts/subagent_creator.py status --pack core --project-dir . --include-extra --fail-on-drift --check project
```

Plan a sync without writing files:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py plan-sync --pack docs --target global --include-extra
python3 skills/subagent-creator/scripts/subagent_creator.py plan-sync --pack docs --target global --prune-extra
```

Sync with backups:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py sync --pack docs --target global
```

Prune stale roles:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py prune --pack docs --target global
python3 skills/subagent-creator/scripts/subagent_creator.py prune --pack docs --target global --confirm
```

`prune` is a dry run unless `--confirm` is present. Confirmed prunes back up
deleted TOML files by default; pass `--no-backup` only when another backup
exists.

Validate everything:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py validate ~/.codex/agents
python3 skills/subagent-creator/scripts/subagent_creator.py validate skills/subagent-creator/templates/agents
```

Smoke test:

```bash
python3 skills/subagent-creator/scripts/subagent_creator.py smoke --pack docs
python3 skills/subagent-creator/scripts/subagent_creator.py smoke --pack docs --run-codex --timeout 120
```

## Validation Rules

The validator checks:

- required TOML fields;
- snake_case role names;
- filename matches role name;
- built-in role shadowing (`default`, `worker`, `explorer`);
- allowed top-level keys;
- model reasoning effort values;
- sandbox values;
- nickname shape;
- common safety/footer language.

## Best Practices

- Avoid custom role names that shadow built-ins.
- Keep role prompts narrow and self-contained.
- Prefer read-only roles unless the role explicitly implements or tests.
- For edit-capable roles, define owned file paths.
- Include return sections that expose status, evidence, inspected files, commands,
  findings, and risks.
- Pair installed roles with `$subspawn`; templates alone do not enforce runtime
  wait behavior.
- Run `status --include-extra` before broad `sync` or `prune` operations so
  hand-authored global roles are visible before writes.
