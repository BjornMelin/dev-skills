import { describe, expect, test } from "bun:test";
import { Buffer } from "node:buffer";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach } from "bun:test";
import { buildRunRecord, writeRun } from "../lib/lifecycle";
import { encodeMcpMessage, handleJsonRpc, readMcpFrames } from "../lib/mcp";

const temps: string[] = [];

function tempDir(prefix: string): string {
  const dir = mkdtempSync(join(tmpdir(), prefix));
  temps.push(dir);
  return dir;
}

function tempGitProject(): string {
  const root = tempDir("kimi-ui-agent-project-");
  execFileSync("git", ["init"], { cwd: root, stdio: "ignore" });
  execFileSync("git", ["config", "user.email", "kimi-ui-agent@example.com"], { cwd: root });
  execFileSync("git", ["config", "user.name", "Kimi UI Agent"], { cwd: root });
  writeFileSync(join(root, "README.md"), "# Test\n", "utf8");
  execFileSync("git", ["add", "README.md"], { cwd: root });
  execFileSync("git", ["commit", "-m", "init"], { cwd: root, stdio: "ignore" });
  return root;
}

async function withStateHome<T>(stateHome: string, action: () => T | Promise<T>): Promise<T> {
  const previous = process.env.XDG_STATE_HOME;
  process.env.XDG_STATE_HOME = stateHome;
  try {
    return await action();
  } finally {
    if (previous === undefined) delete process.env.XDG_STATE_HOME;
    else process.env.XDG_STATE_HOME = previous;
  }
}

afterEach(() => {
  for (const dir of temps.splice(0).reverse()) rmSync(dir, { recursive: true, force: true });
});

const initialize = {
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    protocolVersion: "2025-11-25",
    capabilities: {},
    clientInfo: { name: "test-client", version: "1.0.0" },
  },
};

describe("MCP stdio framing", () => {
  test("reads newline-delimited JSON-RPC frames", () => {
    const framed = Buffer.from(`${JSON.stringify(initialize)}\n`);
    const decoded = readMcpFrames(framed);
    expect(decoded.frames).toHaveLength(1);
    expect(decoded.frames[0]?.format).toBe("line");
    expect(decoded.frames[0]?.request).toEqual(initialize);
    expect(decoded.remaining.length).toBe(0);
  });

  test("reads Content-Length JSON-RPC frames", () => {
    const framed = Buffer.from(encodeMcpMessage(initialize, "content-length"));
    const decoded = readMcpFrames(framed);
    expect(decoded.frames).toHaveLength(1);
    expect(decoded.frames[0]?.format).toBe("content-length");
    expect(decoded.frames[0]?.request).toEqual(initialize);
    expect(decoded.remaining.length).toBe(0);
  });

  test("waits for complete Content-Length bodies", () => {
    const framed = encodeMcpMessage(initialize, "content-length");
    const split = framed.indexOf("\r\n\r\n") + 6;
    const partial = readMcpFrames(Buffer.from(framed.slice(0, split)));
    expect(partial.frames).toHaveLength(0);
    expect(partial.remaining.length).toBe(split);
  });

  test("caps unterminated Content-Length header buffers", () => {
    const oversized = Buffer.concat([Buffer.from("Content-Length: 1\r\n"), Buffer.alloc(10 * 1024 * 1024 + 1)]);
    expect(() => readMcpFrames(oversized)).toThrow(/maximum size/);
  });

  test("writes Content-Length headers using body byte length", () => {
    const framed = encodeMcpMessage(initialize, "content-length");
    const [header, body] = framed.split("\r\n\r\n");
    expect(header).toBe(`Content-Length: ${Buffer.byteLength(body || "", "utf8")}`);
    expect(JSON.parse(body || "{}")).toEqual(initialize);
  });
});

describe("MCP lifecycle tools", () => {
  test("start can apply a reviewed run id", async () => {
    const root = tempGitProject();
    const stateHome = tempDir("kimi-ui-agent-state-");
    await withStateHome(stateHome, async () => {
      const dryRun = await handleJsonRpc({
        jsonrpc: "2.0",
        id: 1,
        method: "tools/call",
        params: { name: "start", arguments: { projectRoot: root, task: "Improve UI", runId: "run-reviewed-abc123" } },
      });
      expect(dryRun?.error).toBeUndefined();

      const applied = await handleJsonRpc({
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: { name: "start", arguments: { projectRoot: root, task: "Improve UI", runId: "run-reviewed-abc123", apply: true } },
      });
      expect(applied?.error).toBeUndefined();
      expect(JSON.stringify(applied?.result)).toContain("run-reviewed-abc123");
    });
  });

  test("start rejects non-boolean apply", async () => {
    const response = await handleJsonRpc({
      jsonrpc: "2.0",
      id: 1,
      method: "tools/call",
      params: { name: "start", arguments: { task: "Improve UI", apply: "true" } },
    });

    expect(response?.error?.message).toContain("apply must be a boolean");
  });

  test("reply requires explicit apply before mutating artifacts", async () => {
    const root = tempDir("kimi-ui-agent-project-");
    const stateHome = tempDir("kimi-ui-agent-state-");
    await withStateHome(stateHome, async () => {
      const run = buildRunRecord({ projectRoot: root, task: "Improve UI", runId: "run-mcp-abc123", apply: false });
      writeRun(run);

      const dryRun = await handleJsonRpc({
        jsonrpc: "2.0",
        id: 1,
        method: "tools/call",
        params: { name: "reply", arguments: { runId: run.runId, message: "hello" } },
      });
      expect(dryRun?.error?.message).toContain("apply");
      expect(existsSync(join(run.artifactDir, "ANSWERS.md"))).toBe(false);

      const applied = await handleJsonRpc({
        jsonrpc: "2.0",
        id: 2,
        method: "tools/call",
        params: { name: "reply", arguments: { runId: run.runId, message: "hello", apply: true } },
      });
      expect(applied?.error).toBeUndefined();
      expect(readFileSync(join(run.artifactDir, "ANSWERS.md"), "utf8")).toContain("hello");
    });
  });
});
