import { existsSync, lstatSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { execFileSync } from "node:child_process";
import type { ManagedWrite, RunRecord } from "./types";
import {
  ensureDir,
  nowIso,
  prepareInsideWrite,
  runIdFromTime,
  runStatePath,
  safeSegment,
  shellQuote,
  slugify,
  stateRoot,
  worktreeRootFor,
} from "./paths";
import { RUNS_GITIGNORE_CONTENT, RUNS_GITIGNORE_REL, loadConfig } from "./config";
import { redact } from "./redact";

export type StartOptions = {
  projectRoot: string;
  task: string;
  apply: boolean;
  runId?: string;
};

export type RunCommand = "reply" | "continue" | "finalize" | "abort";

const ARTIFACT_FILE_NAMES = [
  "INPUT.md",
  "KIMI_PROMPT.md",
  "QUESTIONS.md",
  "PLAN.md",
  "APPROVAL.md",
  "RESULT.md",
  "CHANGED_FILES.txt",
  "ANSWERS.md",
  "ABORTED.md",
  "run.json",
  "events.jsonl",
] as const;

const ARTIFACT_FILES = new Set<string>(ARTIFACT_FILE_NAMES);
const SETUP_CONTEXT_FILES = [
  "config.json",
  "project-profile.md",
  "frontend-map.md",
  "design-system.md",
  "verification.md",
  "protected-paths.md",
  "profile.lock.json",
] as const;

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string" || !value) throw new Error(`${label} must be a non-empty string`);
  return value;
}

function canonicalWorktreePath(projectRoot: string, runId: string): string {
  return join(worktreeRootFor(projectRoot), runId);
}

function canonicalArtifactDir(worktreePath: string, runId: string): string {
  return join(worktreePath, ".agents", "kimi-ui-agent", "runs", runId);
}

function normalizeRunRecord(parsed: RunRecord, expectedRunId?: string): RunRecord {
  const runId = safeSegment(requireString(parsed.runId, "run id"), "run id");
  if (expectedRunId && runId !== expectedRunId) throw new Error(`run id mismatch in stored state: ${runId}`);
  const projectRoot = requireString(parsed.projectRoot, "project root");
  const expectedWorktree = canonicalWorktreePath(projectRoot, runId);
  if (requireString(parsed.worktreePath, "worktree path") !== expectedWorktree) {
    throw new Error(`run worktree path does not match managed state root: ${runId}`);
  }
  return {
    ...parsed,
    runId,
    projectRoot,
    worktreePath: expectedWorktree,
    artifactDir: canonicalArtifactDir(expectedWorktree, runId),
  };
}

function artifactWritePath(run: RunRecord, file: string): string {
  if (!ARTIFACT_FILES.has(file)) throw new Error(`unsupported run artifact: ${file}`);
  return prepareInsideWrite(run.worktreePath, join(".agents", "kimi-ui-agent", "runs", run.runId, file));
}

function controllerCommand(): string {
  const skillDir = process.env.KIMI_UI_AGENT_SKILL_DIR;
  if (skillDir) return `bun ${shellQuote(join(skillDir, "scripts", "kimi-ui-agent.ts"))}`;
  const entrypoint = process.argv[1];
  if (entrypoint && entrypoint.endsWith("kimi-ui-agent.ts")) return `bun ${shellQuote(entrypoint)}`;
  return "kimi-ui-agent";
}

export function buildRunRecord(options: StartOptions): RunRecord {
  const runId = safeSegment(options.runId || runIdFromTime(), "run id");
  const config = loadConfig(options.projectRoot);
  const task = redact(options.task, config?.redaction.extraPatterns || []);
  const branchPrefix = config?.branchPrefix || "kimi-ui";
  const branchName = `${branchPrefix}/${slugify(task, "task")}-${runId.slice(-6)}`;
  const worktreePath = canonicalWorktreePath(options.projectRoot, runId);
  const artifactDir = canonicalArtifactDir(worktreePath, runId);
  const now = nowIso();
  return {
    schema: "kimi_ui_agent.run.v1",
    runId,
    createdAt: now,
    updatedAt: now,
    state: "planned",
    projectRoot: options.projectRoot,
    worktreePath,
    branchName,
    task,
    artifactDir,
  };
}

export function startRun(options: StartOptions): { run: RunRecord; writes: ManagedWrite[]; launchCommand: string; applyCommand: string } {
  const run = buildRunRecord(options);
  const prompt = buildPrompt(run);
  const writes = [...setupContextWrites(run), ...runArtifactWrites(run, prompt)];
  const launchCommand = `${controllerCommand()} launch --run-id ${shellQuote(run.runId)}`;
  const applyCommand = `${controllerCommand()} start --task ${shellQuote(run.task)} --run-id ${shellQuote(run.runId)} --apply`;
  if (options.apply) {
    ensureDir(join(stateRoot(), "runs", run.runId));
    ensureDir(join(run.worktreePath, ".."));
    if (!existsSync(run.worktreePath)) {
      execFileSync("git", ["worktree", "add", "--quiet", "-b", run.branchName, run.worktreePath, "HEAD"], {
        cwd: options.projectRoot,
        stdio: ["ignore", "ignore", "pipe"],
      });
    }
    for (const write of writes) {
      const target = prepareInsideWrite(run.worktreePath, write.path);
      if (write.path === RUNS_GITIGNORE_REL && existsSync(target)) continue;
      writeFileSync(target, write.content || "", "utf8");
    }
    writeRun(run);
  }
  return { run, writes, launchCommand, applyCommand };
}

function setupContextWrites(run: RunRecord): ManagedWrite[] {
  return SETUP_CONTEXT_FILES.flatMap((file): ManagedWrite[] => {
    const source = join(run.projectRoot, ".agents", "kimi-ui-agent", file);
    try {
      const stat = lstatSync(source);
      if (!stat.isFile() || stat.isSymbolicLink()) return [];
      return [
        {
          path: `.agents/kimi-ui-agent/${file}`,
          action: "create",
          reason: "snapshot setup context into isolated worktree",
          content: readFileSync(source, "utf8"),
        },
      ];
    } catch {
      return [];
    }
  });
}

function runArtifactWrites(run: RunRecord, prompt: string): ManagedWrite[] {
  const rel = `.agents/kimi-ui-agent/runs/${run.runId}`;
  return [
    {
      path: RUNS_GITIGNORE_REL,
      action: "create",
      reason: "keep volatile run artifacts out of version control in isolated worktrees",
      content: RUNS_GITIGNORE_CONTENT,
    },
    { path: `${rel}/INPUT.md`, action: "create", reason: "task input", content: `# Input\n\n${redact(run.task)}\n` },
    { path: `${rel}/KIMI_PROMPT.md`, action: "create", reason: "launch prompt", content: `${prompt}\n` },
    { path: `${rel}/QUESTIONS.md`, action: "create", reason: "questions from child agent", content: "# Questions\n\n" },
    { path: `${rel}/PLAN.md`, action: "create", reason: "implementation plan artifact", content: "# Plan\n\n" },
    { path: `${rel}/APPROVAL.md`, action: "create", reason: "approval artifact", content: "# Approval\n\n" },
    { path: `${rel}/RESULT.md`, action: "create", reason: "final result artifact", content: "# Result\n\n" },
    { path: `${rel}/CHANGED_FILES.txt`, action: "create", reason: "changed file list", content: "" },
    {
      path: `${rel}/run.json`,
      action: "create",
      reason: "machine-readable run state",
      content: `${JSON.stringify(run, null, 2)}\n`,
    },
    {
      path: `${rel}/events.jsonl`,
      action: "create",
      reason: "machine-readable run events",
      content: `${JSON.stringify({ ts: nowIso(), event: "created", runId: run.runId })}\n`,
    },
  ];
}

function buildPrompt(run: RunRecord): string {
  return `You are running as Kimi UI Agent for a frontend/UI task.\n\nTask:\n${redact(run.task)}\n\nRules:\n- Stay inside this git worktree: ${run.worktreePath}\n- Start in plan mode and write your proposed plan to .agents/kimi-ui-agent/runs/${run.runId}/PLAN.md.\n- If you need decisions, write them to QUESTIONS.md and stop.\n- Do not touch protected paths from .agents/kimi-ui-agent/protected-paths.md unless the parent agent explicitly approves.\n- When complete, write RESULT.md and CHANGED_FILES.txt.\n`;
}

export function writeRun(run: RunRecord): void {
  const normalized = normalizeRunRecord(run);
  ensureDir(dirname(runStatePath(run.runId)));
  writeFileSync(runStatePath(run.runId), `${JSON.stringify({ ...normalized, updatedAt: nowIso() }, null, 2)}\n`);
}

export function loadRun(runId: string): RunRecord {
  const safe = safeSegment(runId, "run id");
  const path = runStatePath(safe);
  if (!existsSync(path)) throw new Error(`run not found: ${safe}`);
  const parsed = JSON.parse(readFileSync(path, "utf8")) as RunRecord;
  if (parsed.schema !== "kimi_ui_agent.run.v1") throw new Error(`unsupported run schema: ${path}`);
  return normalizeRunRecord(parsed, safe);
}

export function appendArtifact(run: RunRecord, file: string, content: string): void {
  const target = artifactWritePath(run, file);
  const previous = existsSync(target) ? readFileSync(target, "utf8") : "";
  writeFileSync(target, `${previous}${content}\n`, "utf8");
  appendEvent(run, file.replace(/\.md$/i, "").toLowerCase(), { file });
}

export function appendEvent(run: RunRecord, event: string, data: Record<string, unknown> = {}): void {
  const target = artifactWritePath(run, "events.jsonl");
  const previous = existsSync(target) ? readFileSync(target, "utf8") : "";
  writeFileSync(target, `${previous}${JSON.stringify({ ts: nowIso(), event, runId: run.runId, ...data })}\n`, "utf8");
}

export function updateRunState(run: RunRecord, state: RunRecord["state"]): RunRecord {
  const next = normalizeRunRecord({ ...run, state, updatedAt: nowIso() });
  writeRun(next);
  if (existsSync(next.artifactDir)) {
    writeFileSync(artifactWritePath(next, "run.json"), `${JSON.stringify(next, null, 2)}\n`, "utf8");
    appendEvent(next, state);
  }
  return next;
}

export function applyRunCommand(
  runId: string,
  command: RunCommand,
  options: { apply: boolean; message?: string; reason?: string },
): RunRecord {
  if (!options.apply) throw new Error(`${command} requires --apply`);
  const run = loadRun(safeSegment(runId, "run id"));
  if (command === "reply") {
    if (!options.message) throw new Error("--message is required");
    appendArtifact(run, "ANSWERS.md", `\n${options.message}\n`);
    return updateRunState(run, "waiting");
  }
  if (command === "continue") return updateRunState(run, "ready");
  if (command === "finalize") return updateRunState(run, "finalized");
  if (options.reason) appendArtifact(run, "ABORTED.md", options.reason);
  return updateRunState(run, "aborted");
}

export function statusRun(runId: string): RunRecord & { artifacts: string[] } {
  const run = loadRun(runId);
  const artifacts = existsSync(run.artifactDir)
    ? ARTIFACT_FILE_NAMES.filter((file) => existsSync(join(run.artifactDir, file)))
    : [];
  return { ...run, artifacts };
}

export function worktreeSummary(run: RunRecord): string | null {
  try {
    return execFileSync("git", ["status", "--short"], {
      cwd: run.worktreePath,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return null;
  }
}
