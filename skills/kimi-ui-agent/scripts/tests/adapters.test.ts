import { afterEach, describe, expect, test } from "bun:test";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { adapterWrites, commandParts } from "../lib/adapters";

const temps: string[] = [];
const testDir = dirname(fileURLToPath(import.meta.url));
const cliPath = join(testDir, "..", "kimi-ui-agent.ts");

function tempDir(prefix: string): string {
  const dir = mkdtempSync(join(tmpdir(), prefix));
  temps.push(dir);
  return dir;
}

afterEach(() => {
  for (const dir of temps.splice(0).reverse()) rmSync(dir, { recursive: true, force: true });
});

describe("adapters", () => {
  test("renders all project-local adapters without secrets", () => {
    const writes = adapterWrites({ cliCommand: "kimi-ui-agent" });
    const paths = writes.map((write) => write.path);
    expect(paths).toContain(".agents/skills/kimi-ui-agent/SKILL.md");
    expect(paths).toContain(".kimi-code/skills/kimi-ui-agent/SKILL.md");
    expect(paths).toContain(".claude/skills/kimi-ui-agent/SKILL.md");
    expect(writes.map((write) => write.content || "").join("\n")).not.toContain("sk-");
    expect(writes.map((write) => write.content || "").join("\n")).toContain("MOONSHOT_API_KEY");
  });

  test("splits shell-style CLI command into MCP executable and args", () => {
    const command = "bun 'skills/kimi ui/scripts/kimi-ui-agent.ts'";
    const writes = adapterWrites({ cliCommand: command });
    const kimiMcp = JSON.parse(writes.find((write) => write.path.endsWith("mcp.kimi-ui-agent.example.json"))?.content || "{}");
    const server = kimiMcp.mcpServers.kimi_ui_agent;

    expect(commandParts(command)).toEqual({ command: "bun", args: ["skills/kimi ui/scripts/kimi-ui-agent.ts"] });
    expect(server.command).toBe("bun");
    expect(server.args).toEqual(["skills/kimi ui/scripts/kimi-ui-agent.ts", "mcp"]);

    const codexSnippet = writes.find((write) => write.path.endsWith("config-snippet.toml"))?.content || "";
    expect(codexSnippet).toContain('command = "bun"');
    expect(codexSnippet).toContain('args = ["skills/kimi ui/scripts/kimi-ui-agent.ts","mcp"]');
  });

  test("install defaults project adapters to the bundled Bun CLI", () => {
    const root = tempDir("kimi-ui-agent-install-");
    const result = spawnSync(process.execPath, [cliPath, "--json", "install", "--target", "project", "--apply"], {
      cwd: root,
      encoding: "utf8",
    });

    expect(result.status).toBe(0);
    const skill = readFileSync(join(root, ".agents", "skills", "kimi-ui-agent", "SKILL.md"), "utf8");
    expect(skill).toContain("bun ");
    expect(skill).toContain("scripts/kimi-ui-agent.ts");

    const codexMcp = readFileSync(join(root, ".agents", "kimi-ui-agent", "adapters", "codex", "config-snippet.toml"), "utf8");
    expect(codexMcp).toContain('command = "bun"');
    expect(codexMcp).toContain("scripts/kimi-ui-agent.ts");
    expect(codexMcp).toContain('"mcp"');
  });
});
