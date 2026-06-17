# Codex Notes

Codex support uses:

- Explicit `$kimi-ui-agent` invocation.
- Project `.agents/skills/kimi-ui-agent/SKILL.md` adapter.
- Optional MCP config snippet for a configured `mcp` command.

Run the skill-local Bun CLI with `--json doctor` before using lifecycle
commands. Keep worktree starts dry-run-first and review the generated branch,
worktree, and artifact paths before applying.
