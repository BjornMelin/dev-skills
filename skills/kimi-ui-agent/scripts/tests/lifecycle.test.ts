import { afterEach, describe, expect, test } from "bun:test";
import { execFileSync, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { appendArtifact, buildRunRecord, loadRun, statusRun, writeRun } from "../lib/lifecycle";
import { ensureDir, runStatePath } from "../lib/paths";

const temps: string[] = [];
const testDir = dirname(fileURLToPath(import.meta.url));
const cliPath = join(testDir, "..", "kimi-ui-agent.ts");

function tempDir(prefix: string): string {
  const dir = mkdtempSync(join(tmpdir(), prefix));
  temps.push(dir);
  return dir;
}

function execGit(root: string, args: string[]): void {
  execFileSync("git", args, { cwd: root, stdio: "ignore" });
}

function tempGitProject(): string {
  const root = tempDir("kimi-ui-agent-git-");
  execGit(root, ["init"]);
  execGit(root, ["config", "user.email", "kimi-ui-agent@example.com"]);
  execGit(root, ["config", "user.name", "Kimi UI Agent Test"]);
  writeFileSync(join(root, "README.md"), "# Test Project\n", "utf8");
  execGit(root, ["add", "README.md"]);
  execGit(root, ["commit", "-m", "init"]);
  return root;
}

function withStateHome<T>(stateHome: string, action: () => T): T {
  const previous = process.env.XDG_STATE_HOME;
  process.env.XDG_STATE_HOME = stateHome;
  try {
    return action();
  } finally {
    if (previous === undefined) delete process.env.XDG_STATE_HOME;
    else process.env.XDG_STATE_HOME = previous;
  }
}

afterEach(() => {
  for (const dir of temps.splice(0).reverse()) rmSync(dir, { recursive: true, force: true });
});

describe("run lifecycle", () => {
  test("start --apply --json keeps stdout parseable and redacts task text", () => {
    const root = tempGitProject();
    const stateHome = tempDir("kimi-ui-agent-state-");
    const secret = "sk-abcdefghijklmnopqrstuvwxyz";
    const result = spawnSync(
      process.execPath,
      [cliPath, "--json", "start", "--task", `Improve UI with ${secret}`, "--run-id", "run-test-abc123", "--apply"],
      {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, XDG_STATE_HOME: stateHome },
      },
    );

    expect(result.status).toBe(0);
    const parsed = JSON.parse(result.stdout);
    const run = parsed.result.run;
    expect(run.task).toContain("[REDACTED]");
    expect(run.task).not.toContain(secret);
    expect(run.branchName).not.toContain(secret);
    expect(parsed.result.launchCommand).toContain("bun ");
    expect(parsed.result.launchCommand).toContain("scripts/kimi-ui-agent.ts");
    expect(parsed.result.launchCommand).toContain("launch --run-id 'run-test-abc123'");

    const artifact = readFileSync(join(run.artifactDir, "run.json"), "utf8");
    expect(artifact).toContain("[REDACTED]");
    expect(artifact).not.toContain(secret);
    expect(readFileSync(join(run.worktreePath, ".agents", "kimi-ui-agent", "runs", ".gitignore"), "utf8")).toBe("*\n!.gitignore\n");
  });

  test("start dry-run returns an apply command pinned to the previewed run id", () => {
    const root = tempGitProject();
    const stateHome = tempDir("kimi-ui-agent-state-");
    const result = spawnSync(process.execPath, [cliPath, "--json", "start", "--task", "Improve empty states"], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, XDG_STATE_HOME: stateHome },
    });

    expect(result.status).toBe(0);
    const parsed = JSON.parse(result.stdout);
    const runId = parsed.result.run.runId;
    expect(parsed.result.applyCommand).toContain("bun ");
    expect(parsed.result.applyCommand).toContain("scripts/kimi-ui-agent.ts");
    expect(parsed.result.applyCommand).toContain("start --task 'Improve empty states'");
    expect(parsed.result.applyCommand).toContain(`--run-id '${runId}'`);
    expect(parsed.result.applyCommand).toContain("--apply");
  });

  test("start --apply snapshots uncommitted setup context into the worktree", () => {
    const root = tempGitProject();
    const stateHome = tempDir("kimi-ui-agent-state-");
    const contextDir = join(root, ".agents", "kimi-ui-agent");
    mkdirSync(contextDir, { recursive: true });
    writeFileSync(join(contextDir, "protected-paths.md"), "# Protected Paths\n\n- payments/**\n", "utf8");
    writeFileSync(join(contextDir, "project-profile.md"), "# Product context\n", "utf8");

    const result = spawnSync(
      process.execPath,
      [cliPath, "--json", "start", "--task", "Improve UI", "--run-id", "run-context-abc123", "--apply"],
      {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, XDG_STATE_HOME: stateHome },
      },
    );

    expect(result.status).toBe(0);
    const run = JSON.parse(result.stdout).result.run;
    expect(readFileSync(join(run.worktreePath, ".agents", "kimi-ui-agent", "protected-paths.md"), "utf8")).toContain("payments/**");
    expect(readFileSync(join(run.worktreePath, ".agents", "kimi-ui-agent", "project-profile.md"), "utf8")).toContain("Product context");
  });

  test("--json --help returns machine-readable output", () => {
    const result = spawnSync(process.execPath, [cliPath, "--json", "--help"], { encoding: "utf8" });
    expect(result.status).toBe(0);
    expect(JSON.parse(result.stdout).ok).toBe(true);
  });

  test("launch shell-quotes worktree paths and submits the prompt file", () => {
    const root = tempDir("kimi-ui-agent-project-");
    const stateHome = join(tempDir("kimi-ui-agent-state-parent-"), "state-$(touch pwned)");
    withStateHome(stateHome, () => {
      const run = buildRunRecord({ projectRoot: root, task: "Improve UI", runId: "run-quote-abc123", apply: false });
      writeRun(run);
      const result = spawnSync(process.execPath, [cliPath, "--json", "launch", "--run-id", run.runId], {
        cwd: root,
        encoding: "utf8",
        env: { ...process.env, XDG_STATE_HOME: stateHome },
      });

      expect(result.status).toBe(0);
      const parsed = JSON.parse(result.stdout);
      expect(parsed.result.command).toBe(`cd '${run.worktreePath}' && kimi --prompt "$(cat '.agents/kimi-ui-agent/runs/${run.runId}/KIMI_PROMPT.md')"`);
      expect(parsed.result.command).not.toContain(`cd "${run.worktreePath}"`);
      expect(parsed.result.command).not.toContain("--plan <");
    });
  });

  test("status includes reply and abort artifacts when present", () => {
    const root = tempDir("kimi-ui-agent-project-");
    const stateHome = tempDir("kimi-ui-agent-state-");
    withStateHome(stateHome, () => {
      const run = buildRunRecord({ projectRoot: root, task: "Improve UI", runId: "run-status-abc123", apply: false });
      writeRun(run);
      appendArtifact(run, "ANSWERS.md", "approved");
      appendArtifact(run, "ABORTED.md", "scope changed");

      expect(statusRun(run.runId).artifacts).toEqual(["ANSWERS.md", "ABORTED.md", "events.jsonl"]);
    });
  });

  test("loaded runs ignore tampered artifactDir values", () => {
    const root = tempDir("kimi-ui-agent-project-");
    const stateHome = tempDir("kimi-ui-agent-state-");
    const outside = tempDir("kimi-ui-agent-outside-");
    withStateHome(stateHome, () => {
      const run = buildRunRecord({ projectRoot: root, task: "Improve UI", runId: "run-tamper-abc123", apply: false });
      ensureDir(dirname(runStatePath(run.runId)));
      writeFileSync(runStatePath(run.runId), `${JSON.stringify({ ...run, artifactDir: outside }, null, 2)}\n`, "utf8");

      const loaded = loadRun(run.runId);
      expect(loaded.artifactDir).not.toBe(outside);
      appendArtifact(loaded, "ANSWERS.md", "safe reply");

      expect(existsSync(join(outside, "ANSWERS.md"))).toBe(false);
      expect(readFileSync(join(loaded.artifactDir, "ANSWERS.md"), "utf8")).toContain("safe reply");
    });
  });

  test("artifact writes reject symlinked artifact roots", () => {
    const root = tempDir("kimi-ui-agent-project-");
    const stateHome = tempDir("kimi-ui-agent-state-");
    const outside = tempDir("kimi-ui-agent-outside-");
    withStateHome(stateHome, () => {
      const run = buildRunRecord({ projectRoot: root, task: "Improve UI", runId: "run-symlink-abc123", apply: false });
      mkdirSync(run.worktreePath, { recursive: true });
      symlinkSync(outside, join(run.worktreePath, ".agents"), "dir");

      expect(() => appendArtifact(run, "ANSWERS.md", "unsafe reply")).toThrow(/refusing symlinked path/);
      expect(existsSync(join(outside, "kimi-ui-agent", "runs", run.runId, "ANSWERS.md"))).toBe(false);
    });
  });
});
