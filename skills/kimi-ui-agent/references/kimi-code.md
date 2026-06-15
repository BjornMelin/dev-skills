# Kimi Code Notes

`kimi-ui-agent` targets current Kimi Code behavior:

- Kimi Code scans project `.kimi-code/skills/` and `.agents/skills/`.
- Kimi Code supports MCP through project or user `mcp.json`.
- Kimi Code supports stdio, HTTP, and SSE MCP transports.
- Submit generated run prompts with `kimi --prompt "$(cat KIMI_PROMPT.md)"`;
  prompt mode is the supported non-interactive CLI path and cannot be combined
  with `--plan`.
- Keep the generated prompt plan-first by requiring `PLAN.md` before
  implementation artifacts.
- Treat Kimi session directories and exported logs as sensitive local debug
  material.

The default model guidance for generated docs is Kimi K2.7 Code. Keep thinking
content/tool-call continuity intact when using the API directly. The CLI itself
does not write provider tokens.
