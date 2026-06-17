import { join } from "node:path";
import type { ManagedWrite } from "./types";

/** Adapter rendering options for project-local harness integrations. */
export type AdapterOptions = {
  cliCommand: string;
};

type CommandParts = {
  command: string;
  args: string[];
};

/**
 * Renders all project-local adapter files for supported harnesses.
 *
 * @param options - Adapter rendering options, including the CLI command to invoke.
 * @returns Managed writes for Codex, Kimi Code, Claude Code, and MCP snippets.
 */
export function adapterWrites(options: AdapterOptions): ManagedWrite[] {
  const command = options.cliCommand;
  return [
    {
      path: ".agents/skills/kimi-ui-agent/SKILL.md",
      action: "create",
      reason: "Codex project-local adapter skill",
      content: codexSkill(command),
    },
    {
      path: ".kimi-code/skills/kimi-ui-agent/SKILL.md",
      action: "create",
      reason: "Kimi Code project-local adapter skill",
      content: kimiCodeSkill(command),
    },
    {
      path: ".claude/skills/kimi-ui-agent/SKILL.md",
      action: "create",
      reason: "Claude Code project-local adapter skill",
      content: claudeSkill(command),
    },
    {
      path: ".agents/kimi-ui-agent/adapters/kimi-code/mcp.kimi-ui-agent.example.json",
      action: "create",
      reason: "Kimi Code MCP snippet; copy or merge manually unless install is explicitly extended to write MCP config",
      content: `${JSON.stringify(kimiMcpSnippet(command), null, 2)}\n`,
    },
    {
      path: ".agents/kimi-ui-agent/adapters/codex/config-snippet.toml",
      action: "create",
      reason: "Codex MCP config snippet",
      content: codexConfigSnippet(command),
    },
    {
      path: ".agents/kimi-ui-agent/adapters/claude-code/env.kimi-k27.example.sh",
      action: "create",
      reason: "Claude Code Kimi K2.7 Code provider environment template with placeholders only",
      content: claudeEnvTemplate(),
    },
    {
      path: ".agents/kimi-ui-agent/adapters/claude-code/plugin.kimi-ui-agent.template.json",
      action: "create",
      reason: "Claude Code plugin template for teams that package skill plus hooks/MCP",
      content: `${JSON.stringify(claudePluginTemplate(command), null, 2)}\n`,
    },
  ];
}

function codexSkill(command: string): string {
  return `---\nname: kimi-ui-agent\ndescription: Explicit-only adapter for the configured Kimi UI Agent CLI. Use when the user asks to set up, profile, run, inspect, or finalize a Kimi-powered frontend/UI worktree orchestration workflow from Codex.\n---\n\n# Kimi UI Agent\n\nUse the configured \`${command}\` CLI as the source of truth.\n\n1. Run \`${command} --json doctor\` first.\n2. Run \`${command} --json setup --dry-run\` before first use in this repo.\n3. Apply setup only when the user wants durable project context: \`${command} --json setup --apply\`.\n4. Start work with plan-first isolation: \`${command} --json start --task \"<task>\" --dry-run\`, then rerun with \`--apply\` after review.\n5. Use \`${command} --json status --run-id <id>\`, \`${command} --json reply --run-id <id> --message \"...\" --apply\`, and \`${command} --json finalize --run-id <id> --apply\` for lifecycle control.\n\nDo not paste secrets into prompts or config. Do not use autonomous launch flags unless the user explicitly requests that risk.\n`;
}

function kimiCodeSkill(command: string): string {
  return `---\nname: kimi-ui-agent\ndescription: Project frontend/UI orchestration workflow using the configured Kimi UI Agent CLI, project profile, worktree isolation, and review artifacts.\ntype: prompt\nwhenToUse: When asked to plan, audit, redesign, implement, or review frontend/UI changes through Kimi UI Agent.\ndisableModelInvocation: true\n---\n\nUse \`${command}\` for deterministic project setup, run lifecycle, and artifact status. Read \`.agents/kimi-ui-agent/project-profile.md\`, \`frontend-map.md\`, \`design-system.md\`, \`verification.md\`, and \`protected-paths.md\` when present.\n\nStart with \`${command} --json doctor\`. Prefer plan-first worktree runs. Do not use YOLO/autonomous modes unless the user explicitly requests them.\n`;
}

function claudeSkill(command: string): string {
  return `---\nname: kimi-ui-agent\ndescription: Use the configured Kimi UI Agent CLI for frontend/UI repo profiling, Kimi-powered planning, worktree orchestration, and review artifacts from Claude Code.\nallowed-tools: Bash(${command}:*) Read Glob Grep\ncontext: fork\n---\n\nUse \`${command}\` as the deterministic controller.\n\n- Run \`${command} --json doctor\` before lifecycle commands.\n- Run \`${command} --json setup --dry-run\` to inspect generated project intelligence before applying.\n- Use \`${command} --json setup --apply\` only when durable \`.agents/kimi-ui-agent\` context should be written.\n- Keep secrets out of prompts and config.\n- Keep implementation plan-first unless the user explicitly chooses a more autonomous mode.\n`;
}

function kimiMcpSnippet(command: string): Record<string, unknown> {
  const invocation = commandParts(command);
  return {
    mcpServers: {
      kimi_ui_agent: {
        command: invocation.command,
        args: [...invocation.args, "mcp"],
        enabledTools: ["start", "status", "reply", "continue", "finalize", "abort"],
      },
    },
  };
}

function codexConfigSnippet(command: string): string {
  const invocation = commandParts(command);
  return `[mcp_servers.kimi_ui_agent]\ncommand = ${JSON.stringify(invocation.command)}\nargs = ${JSON.stringify([...invocation.args, "mcp"])}\nstartup_timeout_sec = 10\ntool_timeout_sec = 120\nenabled_tools = ["start", "status", "reply", "continue", "finalize", "abort"]\n`;
}

function claudeEnvTemplate(): string {
  return `# Source this file when you want Claude Code to use Kimi K2.7 Code through Moonshot's Anthropic-compatible endpoint.\n# Fill MOONSHOT_API_KEY in your shell or secret manager. Do not commit real tokens.\nexport ANTHROPIC_BASE_URL=\"https://api.moonshot.ai/anthropic\"\nexport ANTHROPIC_AUTH_TOKEN=\"\${MOONSHOT_API_KEY}\"\nexport ANTHROPIC_MODEL=\"kimi-k2.7-code\"\nexport ANTHROPIC_DEFAULT_OPUS_MODEL=\"kimi-k2.7-code\"\nexport ANTHROPIC_DEFAULT_SONNET_MODEL=\"kimi-k2.7-code\"\nexport ANTHROPIC_DEFAULT_HAIKU_MODEL=\"kimi-k2.7-code\"\nexport CLAUDE_CODE_SUBAGENT_MODEL=\"kimi-k2.7-code\"\nexport ENABLE_TOOL_SEARCH=false\nexport CLAUDE_CODE_AUTO_COMPACT_WINDOW=262144\n`;
}

function claudePluginTemplate(command: string): Record<string, unknown> {
  const invocation = commandParts(command);
  return {
    name: "kimi-ui-agent",
    version: "0.1.0",
    description: "Project-local frontend/UI orchestration through the configured Kimi UI Agent CLI.",
    skills: "./skills",
    mcpServers: {
      kimi_ui_agent: {
        command: invocation.command,
        args: [...invocation.args, "mcp"],
      },
    },
  };
}

/**
 * Splits a shell-style command into executable and argument parts.
 *
 * @param command - Shell-style command string with quote and escape support.
 * @returns Executable command and argument array.
 * @throws When the command is empty or contains an unterminated quoted segment.
 */
export function commandParts(command: string): CommandParts {
  const parts: string[] = [];
  let current = "";
  let quote: "'" | '"' | null = null;
  let escaped = false;
  for (const char of command) {
    if (escaped) {
      current += char;
      escaped = false;
      continue;
    }
    if (char === "\\" && quote !== "'") {
      escaped = true;
      continue;
    }
    if (quote) {
      if (char === quote) quote = null;
      else current += char;
      continue;
    }
    if (char === "'" || char === '"') {
      quote = char;
      continue;
    }
    if (/\s/.test(char)) {
      if (current) {
        parts.push(current);
        current = "";
      }
      continue;
    }
    current += char;
  }
  if (escaped) current += "\\";
  if (quote) throw new Error("unterminated quote in CLI command");
  if (current) parts.push(current);
  const [executable, ...args] = parts;
  if (!executable) throw new Error("CLI command must not be empty");
  return { command: executable, args };
}
