# Claude subagent pack

Tracked Claude Code subagent definitions. Sibling of `subagents/codex/`, which ships Codex CLI
roles as TOML; this pack ships Claude Code agents as Markdown with YAML frontmatter.

```text
subagents/claude/
  agents/global/*.md        agent definitions
  scripts/sync_agents.py    installer with timestamped backups
  ROLE_CATALOG.md           routing matrix and house contract
```

## Install

```bash
# validate the catalog without touching the install
python3 subagents/claude/scripts/sync_agents.py --validate

# install to ~/.claude/agents
python3 subagents/claude/scripts/sync_agents.py --target global

# or into the current repo's .claude/agents
python3 subagents/claude/scripts/sync_agents.py --target project
```

`--dry-run` prints what would change. Files that already match are skipped; files that differ
are backed up to `agent-backups/claude-<timestamp>` beside the target directory first.

## Authoring rules

- Filename must equal the frontmatter `name`.
- `model` and `effort` are required and must be pinned explicitly. `model: inherit` is
  rejected by the validator, per `MODELS.md`.
- Scope tools to the smallest surface that does the job. Reviewer roles get no `Edit`/`Write`.
- Keep bodies focused: what the role does, what it must not do, its boundaries, and the exact
  return shape.
- Never write a literal home path into a tracked file. This repository is public and
  `tools/policy/check_public_leaks.py` fails the commit. Resolve home at runtime.

See `ROLE_CATALOG.md` for the current roster and the rationale behind the role split.
