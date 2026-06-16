# Kimi UI Agent

`kimi-ui-agent` is an explicit-only Agent Skill plus skill-local Bun CLI for
Kimi-powered frontend/UI work. It replaces the archived `kimi-ui-advisor` by
combining read-only UI advice with project profiling, harness adapter setup, and
plan-first isolated worktree orchestration. Invoke the bundled script with Bun;
normal skill installs do not add a `kimi-ui-agent` binary to PATH.

## Command Surface

```bash
KIMI_UI_AGENT_SKILL_DIR="${KIMI_UI_AGENT_SKILL_DIR:-${CODEX_HOME:-$HOME/.codex}/skills/kimi-ui-agent}"
KIMI_UI_AGENT_CLI="${KIMI_UI_AGENT_CLI:-$KIMI_UI_AGENT_SKILL_DIR/scripts/kimi-ui-agent.ts}"
bun "$KIMI_UI_AGENT_CLI" --json doctor
bun "$KIMI_UI_AGENT_CLI" --json setup --dry-run
bun "$KIMI_UI_AGENT_CLI" --json setup --apply
bun "$KIMI_UI_AGENT_CLI" --json profile --refresh --dry-run
bun "$KIMI_UI_AGENT_CLI" --json install --target project --dry-run
bun "$KIMI_UI_AGENT_CLI" --json install --target project --apply
bun "$KIMI_UI_AGENT_CLI" --json start --task "Improve dashboard empty states" --dry-run
bun "$KIMI_UI_AGENT_CLI" --json status --run-id <run-id>
bun "$KIMI_UI_AGENT_CLI" --json launch --run-id <run-id>
bun "$KIMI_UI_AGENT_CLI" mcp
```

All setup, install, and lifecycle writes are dry-run-first. Use `--apply` to
write project profile files, adapter files, or run state.

## Project Profile

`setup --apply` writes durable project intelligence under
`.agents/kimi-ui-agent/`:

- `config.json`
- `project-profile.md`
- `frontend-map.md`
- `design-system.md`
- `verification.md`
- `protected-paths.md`
- `profile.lock.json`

Volatile run artifacts live under `.agents/kimi-ui-agent/runs/`; that directory
contains its own `.gitignore` so users do not need a root ignore entry.
Later `setup --apply` or `profile --refresh --apply` runs preserve existing
editable Markdown context files and refresh machine-owned JSON evidence.
Applied `start` runs snapshot existing setup context into the isolated worktree
so uncommitted profile/protection files are available to the launched Kimi
session.
`launch` renders a prompt-mode Kimi command that submits the generated
`KIMI_PROMPT.md`; Kimi Code does not accept `--prompt` together with `--plan`,
so plan-first behavior is enforced by the generated run prompt and artifacts.

## Adapters

`install --target project --apply` writes project-local adapter files for:

- Codex: `.agents/skills/kimi-ui-agent/SKILL.md`
- Kimi Code: `.kimi-code/skills/kimi-ui-agent/SKILL.md`
- Claude Code: `.claude/skills/kimi-ui-agent/SKILL.md`

It also writes adapter snippets under `.agents/kimi-ui-agent/adapters/`. The
default adapter command is the bundled Bun script, and MCP snippets split that
shell command into executable plus args. Provider/API secrets are placeholders
only.

## Validation

For changes to this skill, run:

```bash
python3 tools/skill/quick_validate.py skills/kimi-ui-agent
bun install --cwd skills/kimi-ui-agent --frozen-lockfile
bun run --cwd skills/kimi-ui-agent typecheck
bun test --cwd skills/kimi-ui-agent
bun skills/kimi-ui-agent/scripts/kimi-ui-agent.ts doctor --json
cargo run -q -p codex-dev -- --json skills audit
```
