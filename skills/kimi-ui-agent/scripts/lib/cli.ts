import type { CliResult, JsonRecord, ManagedWrite } from "./types";
import { join } from "node:path";
import { adapterWrites } from "./adapters";
import { defaultConfig, ensureProfileDirs, loadConfig, writeManaged } from "./config";
import { applyRunCommand, loadRun, startRun, statusRun, worktreeSummary } from "./lifecycle";
import { fail, ok, printResult } from "./output";
import { commandExists, commandOutput, projectRootFrom, safeSegment, shellQuote } from "./paths";
import { profileWrites } from "./setup";
import { runMcpServer } from "./mcp";

type ParsedArgs = {
  command: string;
  flags: Map<string, string | boolean>;
  rest: string[];
  json: boolean;
};

const BOOLEAN_FLAGS = new Set(["json", "help", "apply", "dry-run", "refresh"]);

function usage(): string {
  return `kimi-ui-agent\n\nUsage:\n  kimi-ui-agent [--json] <command> [options]\n\nCommands:\n  doctor                  Inspect CLI, adapters, project profile, and harness tools\n  init                    Create .agents/kimi-ui-agent project policy files (requires --apply)\n  setup                   Scan repo and generate lean project intelligence (dry-run by default)\n  profile --refresh       Refresh generated project intelligence\n  install                 Render project-local Codex/Kimi/Claude adapters (dry-run by default)\n  start --task <text>     Create a plan-first isolated worktree run (dry-run by default)\n  status --run-id <id>    Read run state\n  reply --run-id <id>     Append a response to ANSWERS.md (requires --apply)\n  continue --run-id <id>  Mark run ready to continue (requires --apply)\n  finalize --run-id <id>  Mark run finalized (requires --apply)\n  abort --run-id <id>     Mark run aborted (requires --apply)\n  launch --run-id <id>    Print the safe launch command for a run\n  mcp                     Run stdio MCP server\n\nWrite commands are dry-run by default. Use --apply to write files or mutate run state.\n`;
}

function parseArgv(argv: string[]): ParsedArgs {
  const flags = new Map<string, string | boolean>();
  const rest: string[] = [];
  let command = "";
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index]!;
    if (arg === "--help" || arg === "-h") {
      command = "help";
      continue;
    }
    if (arg.startsWith("--")) {
      const [rawKey, inline] = arg.slice(2).split("=", 2);
      const key = rawKey || "";
      if (!key) throw new Error("empty flag name");
      if (inline !== undefined) {
        flags.set(key, inline);
      } else if (BOOLEAN_FLAGS.has(key)) {
        flags.set(key, true);
      } else if (index + 1 < argv.length && !argv[index + 1]!.startsWith("--")) {
        flags.set(key, argv[index + 1]!);
        index += 1;
      } else {
        flags.set(key, true);
      }
      continue;
    }
    if (!command) command = arg;
    else rest.push(arg);
  }
  return { command: command || "help", flags, rest, json: flagBoolValue(flags.get("json"), "json") };
}

function flag(args: ParsedArgs, name: string): string | undefined {
  const value = args.flags.get(name);
  if (value === undefined || value === true || value === false) return undefined;
  return value;
}

function flagBoolValue(value: string | boolean | undefined, name: string): boolean {
  if (value === undefined) return false;
  if (typeof value === "boolean") return value;
  const normalized = value.trim().toLowerCase();
  if (["true", "1"].includes(normalized)) return true;
  if (["false", "0"].includes(normalized)) return false;
  throw new Error(`--${name} must be a boolean`);
}

function bool(args: ParsedArgs, name: string): boolean {
  return flagBoolValue(args.flags.get(name), name);
}

function applyMode(args: ParsedArgs): boolean {
  const apply = bool(args, "apply");
  if (apply && bool(args, "dry-run")) throw new Error("--apply cannot be combined with --dry-run");
  return apply;
}

function requireFlag(args: ParsedArgs, name: string): string {
  const value = flag(args, name);
  if (!value) throw new Error(`--${name} is required`);
  return value;
}

function writesSummary(writes: ManagedWrite[]): JsonRecord[] {
  return writes.map(({ path, action, reason }) => ({ path, action, reason }));
}

function defaultCliCommand(): string {
  const skillDir = process.env.KIMI_UI_AGENT_SKILL_DIR;
  if (skillDir) return `bun ${shellQuote(join(skillDir, "scripts", "kimi-ui-agent.ts"))}`;
  const entrypoint = process.argv[1];
  if (entrypoint && entrypoint.endsWith("kimi-ui-agent.ts")) return `bun ${shellQuote(entrypoint)}`;
  return "kimi-ui-agent";
}

async function doctor(args: ParsedArgs, cwd: string): Promise<CliResult> {
  const projectRoot = projectRootFrom(cwd, flag(args, "project-dir") || flag(args, "project-root"));
  const config = loadConfig(projectRoot);
  const checks = {
    projectRoot,
    profilePresent: Boolean(config),
    installedGlobal: commandExists("kimi-ui-agent"),
    bun: commandOutput("bun", ["--version"], cwd),
    kimi: commandOutput("kimi", ["--version"], cwd),
    claude: commandExists("claude"),
    codex: commandExists("codex"),
    gitRoot: commandOutput("git", ["rev-parse", "--show-toplevel"], projectRoot),
  };
  return ok("doctor", checks, "doctor complete");
}

async function initProject(args: ParsedArgs, cwd: string): Promise<CliResult> {
  const projectRoot = projectRootFrom(cwd, flag(args, "project-dir") || flag(args, "project-root"));
  const config = loadConfig(projectRoot) || defaultConfig(projectRoot);
  const writes = profileWrites(projectRoot, config).filter((write) =>
    [".agents/kimi-ui-agent/config.json", ".agents/kimi-ui-agent/runs/.gitignore"].includes(write.path),
  );
  const apply = applyMode(args);
  if (apply) ensureProfileDirs(projectRoot);
  writeManaged(projectRoot, writes, apply);
  return ok("init", { projectRoot, apply, writes: writesSummary(writes) }, apply ? "init applied" : "init dry-run");
}

async function setupProject(args: ParsedArgs, cwd: string): Promise<CliResult> {
  const projectRoot = projectRootFrom(cwd, flag(args, "project-dir") || flag(args, "project-root"));
  const existing = loadConfig(projectRoot);
  const writes = profileWrites(projectRoot, existing);
  const apply = applyMode(args);
  if (apply) ensureProfileDirs(projectRoot);
  writeManaged(projectRoot, writes, apply);
  return ok("setup", { projectRoot, apply, writes: writesSummary(writes) }, apply ? "setup applied" : "setup dry-run");
}

async function installProject(args: ParsedArgs, cwd: string): Promise<CliResult> {
  const projectRoot = projectRootFrom(cwd, flag(args, "project-dir") || flag(args, "project-root"));
  const target = flag(args, "target") || "project";
  if (target !== "project") throw new Error("--target currently supports only project");
  const cliCommand = flag(args, "command") || defaultCliCommand();
  const writes = adapterWrites({ cliCommand });
  const apply = applyMode(args);
  writeManaged(projectRoot, writes, apply);
  return ok("install", { projectRoot, target, apply, writes: writesSummary(writes) }, apply ? "install applied" : "install dry-run");
}

async function start(args: ParsedArgs, cwd: string): Promise<CliResult> {
  const projectRoot = projectRootFrom(cwd, flag(args, "project-dir") || flag(args, "project-root"));
  const task = requireFlag(args, "task");
  const runId = flag(args, "run-id");
  const apply = applyMode(args);
  const result = startRun({ projectRoot, task, apply, ...(runId ? { runId } : {}) });
  return ok(
    "start",
    {
      apply,
      run: result.run,
      writes: writesSummary(result.writes),
      launchCommand: result.launchCommand,
      applyCommand: result.applyCommand,
    },
    apply ? "run started" : "start dry-run",
  );
}

async function status(args: ParsedArgs): Promise<CliResult> {
  const runId = safeSegment(requireFlag(args, "run-id"), "run id");
  const run = statusRun(runId);
  return ok("status", { ...run, gitStatus: worktreeSummary(run) }, "status complete");
}

async function writeRunCommand(args: ParsedArgs, command: "reply" | "continue" | "finalize" | "abort"): Promise<CliResult> {
  const message = flag(args, "message");
  const reason = flag(args, "reason");
  const run = applyRunCommand(safeSegment(requireFlag(args, "run-id"), "run id"), command, {
    apply: applyMode(args),
    ...(command === "reply" && message ? { message } : {}),
    ...(reason ? { reason } : {}),
  });
  const messages: Record<"reply" | "continue" | "finalize" | "abort", string> = {
    reply: "reply recorded",
    continue: "run marked ready",
    finalize: "run finalized",
    abort: "run aborted",
  };
  return ok(command, run, messages[command]);
}

async function launch(args: ParsedArgs): Promise<CliResult> {
  const run = loadRun(safeSegment(requireFlag(args, "run-id"), "run id"));
  const promptPath = `.agents/kimi-ui-agent/runs/${run.runId}/KIMI_PROMPT.md`;
  const command = `cd ${shellQuote(run.worktreePath)} && kimi --plan`;
  const promptCommand = `cd ${shellQuote(run.worktreePath)} && cat ${shellQuote(promptPath)}`;
  return ok(
    "launch",
    {
      runId: run.runId,
      command,
      promptPath,
      promptCommand,
      note: "Run promptCommand, review the generated prompt, then paste it into the interactive plan-mode Kimi session. Non-interactive --prompt and autonomous/yolo modes are intentionally omitted.",
    },
    "launch command rendered",
  );
}

/**
 * Parses arguments, dispatches a CLI command, and prints the result.
 *
 * @param argv - Raw CLI arguments after the executable name.
 * @param cwd - Working directory used for project discovery.
 * @returns Process exit code for the command.
 */
export async function runCli(argv: string[], cwd = process.cwd()): Promise<number> {
  let args: ParsedArgs = { command: "parse", flags: new Map(), rest: [], json: argv.includes("--json") || argv.includes("--json=true") || argv.includes("--json=1") };
  let result: CliResult;
  try {
    args = parseArgv(argv);
    if (args.command === "mcp") {
      await runMcpServer();
      return 0;
    }
    if (args.command === "help") {
      if (args.json) {
        printResult(ok("help", { usage: usage() }, "help rendered"), true);
        return 0;
      }
      process.stdout.write(usage());
      return 0;
    }
    if (args.command === "doctor") result = await doctor(args, cwd);
    else if (args.command === "init") result = await initProject(args, cwd);
    else if (args.command === "setup") result = await setupProject(args, cwd);
    else if (args.command === "profile") {
      if (!bool(args, "refresh")) throw new Error("profile currently supports --refresh");
      result = await setupProject(args, cwd);
      result.command = "profile";
    } else if (args.command === "install") result = await installProject(args, cwd);
    else if (args.command === "start") result = await start(args, cwd);
    else if (args.command === "status") result = await status(args);
    else if (["reply", "continue", "finalize", "abort"].includes(args.command)) {
      result = await writeRunCommand(args, args.command as "reply" | "continue" | "finalize" | "abort");
    } else if (args.command === "launch") result = await launch(args);
    else throw new Error(`unknown command: ${args.command}`);
  } catch (error) {
    result = fail(args.command, error instanceof Error ? error.message : String(error));
  }
  printResult(result, args.json);
  return result.ok ? 0 : 1;
}
