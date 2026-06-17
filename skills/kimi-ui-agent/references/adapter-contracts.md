# Adapter Contracts

## MCP

The stdio MCP server name should be `kimi_ui_agent`. Tool names are concise
inside the server namespace:

- `start`
- `status`
- `reply`
- `continue`
- `finalize`
- `abort`

All MCP arguments must be validated server-side. Reject unknown fields and
unsafe run IDs. Lifecycle mutation tools (`reply`, `continue`, `finalize`, and
`abort`) require explicit `apply: true`; missing or false `apply` is a dry-run
error.

## Codex

Use project `.agents/skills/kimi-ui-agent/SKILL.md` plus a printed MCP config
snippet. Keep `agents/openai.yaml` explicit-only.

## Kimi Code

Use project `.kimi-code/skills/kimi-ui-agent/SKILL.md` and a project MCP JSON
snippet. Kimi hooks are optional workflow guards; do not rely on them as the
security boundary.

## Claude Code

Use project `.claude/skills/kimi-ui-agent/SKILL.md` and a plugin template for
teams that want to package skill plus MCP. Provider env templates must contain
placeholders only.
