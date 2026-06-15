# Install Targets

Use this reference before applying adapter writes.

## Default

`install` targets the current project by default and is dry-run-first:

```bash
KIMI_UI_AGENT_CLI="skills/kimi-ui-agent/scripts/kimi-ui-agent.ts"
bun "$KIMI_UI_AGENT_CLI" --json install --target project --dry-run
bun "$KIMI_UI_AGENT_CLI" --json install --target project --apply
```

Project-local outputs include:

- `.agents/skills/kimi-ui-agent/SKILL.md` for Codex.
- `.kimi-code/skills/kimi-ui-agent/SKILL.md` for Kimi Code.
- `.claude/skills/kimi-ui-agent/SKILL.md` for Claude Code.
- `.agents/kimi-ui-agent/adapters/**` snippets and templates.

By default, generated adapters invoke the bundled Bun script that launched the
installer. Pass `--command "<command>"` only when the team has a stable
alternative command. MCP snippets split shell-style commands into executable
`command` plus `args`; for example `bun path/to/kimi-ui-agent.ts` becomes
`command = "bun"` with the script path and `mcp` in args.

User-global adapter writes are intentionally not the default. Use project
install first so teams can review generated files before sharing them.

## Secrets

Generated Claude/Kimi provider files contain placeholders only. Never write
real API tokens to adapter templates or committed profile files.
