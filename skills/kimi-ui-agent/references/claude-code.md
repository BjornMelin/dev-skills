# Claude Code Notes

Claude Code support is native through generated project skill and plugin
templates.

Generated env template values:

```bash
export ANTHROPIC_BASE_URL="https://api.moonshot.ai/anthropic"
export ANTHROPIC_AUTH_TOKEN="${MOONSHOT_API_KEY}"
export ANTHROPIC_MODEL="kimi-k2.7-code"
export CLAUDE_CODE_SUBAGENT_MODEL="kimi-k2.7-code"
export ENABLE_TOOL_SEARCH=false
export CLAUDE_CODE_AUTO_COMPACT_WINDOW=262144
```

Do not commit real tokens. Source the env file only after replacing placeholders
through a local shell secret.
