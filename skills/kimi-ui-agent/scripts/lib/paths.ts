import { lstatSync, mkdirSync, realpathSync } from "node:fs";
import { homedir, platform } from "node:os";
import { basename, dirname, isAbsolute, join, relative, resolve } from "node:path";
import { execFileSync } from "node:child_process";

/** Marker string written into managed generated files. */
export const MANAGED_MARKER = "Managed by kimi-ui-agent";

/**
 * Returns the current timestamp as an ISO-8601 string.
 *
 * @returns Current UTC timestamp.
 */
export function nowIso(): string {
  return new Date().toISOString();
}

/**
 * Expands a leading home-directory marker in user-provided paths.
 *
 * @param input - Path that may start with `~` or `~/`.
 * @returns Path with the current user's home directory expanded.
 */
export function expandHome(input: string): string {
  if (input === "~") return homedir();
  if (input.startsWith("~/")) return join(homedir(), input.slice(2));
  return input;
}

/**
 * Resolves the XDG state home used for parent-controller state.
 *
 * @returns XDG state directory or the platform-local fallback.
 */
export function xdgStateHome(): string {
  return process.env.XDG_STATE_HOME || join(homedir(), ".local", "state");
}

/**
 * Resolves the Kimi UI Agent parent-controller state root.
 *
 * @returns Absolute state root path.
 */
export function stateRoot(): string {
  return join(xdgStateHome(), "dev-skills", "kimi-ui-agent");
}

/**
 * Validates a filesystem-safe single path segment.
 *
 * @param input - Segment to validate.
 * @param label - Human-readable label for error messages.
 * @returns The validated segment.
 * @throws When the segment is empty, unsafe, too long, or contains parent traversal.
 */
export function safeSegment(input: string, label = "value"): string {
  if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,79}$/.test(input)) {
    throw new Error(`${label} must be a safe single path segment`);
  }
  if (input === "." || input === ".." || input.includes("..")) {
    throw new Error(`${label} must not contain parent traversal`);
  }
  return input;
}

/**
 * Converts free-form text into a stable shell-friendly slug.
 *
 * @param input - Text to slugify.
 * @param fallback - Fallback slug when the input has no slug-safe characters.
 * @returns Lowercase dash-separated slug.
 */
export function slugify(input: string, fallback = "project"): string {
  const slug = input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
  return slug || fallback;
}

/**
 * Creates a directory with parent directories as needed.
 *
 * @param path - Directory path to create.
 */
export function ensureDir(path: string): void {
  mkdirSync(path, { recursive: true, mode: 0o700 });
}

/**
 * Resolves a child path and rejects paths outside the managed root.
 *
 * @param root - Managed root directory.
 * @param child - Relative child path to resolve inside the root.
 * @returns Absolute child path inside the root.
 * @throws When the child path escapes the managed root.
 */
export function resolveInside(root: string, child: string): string {
  const rootAbs = resolve(expandHome(root));
  const childAbs = resolve(rootAbs, child);
  const rel = relative(rootAbs, childAbs);
  if (rel === "" || (!rel.startsWith("..") && !isAbsolute(rel))) {
    return childAbs;
  }
  throw new Error(`path escapes managed root: ${child}`);
}

/**
 * Rejects writes that traverse symlinked path ancestors.
 *
 * @param path - Target path whose ancestors should be inspected.
 * @param stopAt - Optional ancestor at which inspection stops.
 * @throws When an existing inspected path is a symlink or cannot be inspected safely.
 */
export function assertNonSymlinkAncestor(path: string, stopAt?: string): void {
  const abs = resolve(path);
  const stop = stopAt ? resolve(stopAt) : dirname(abs);
  let current = abs;
  while (true) {
    try {
      if (lstatSync(current).isSymbolicLink()) {
        throw new Error(`refusing symlinked path: ${current}`);
      }
    } catch (error) {
      if (error instanceof Error && error.message.startsWith("refusing symlinked path:")) throw error;
      const code = typeof error === "object" && error && "code" in error ? (error as { code?: string }).code : undefined;
      if (code !== "ENOENT" && code !== "ENOTDIR") throw error;
    }
    if (current === stop || current === dirname(current)) break;
    if (relative(stop, current).startsWith("..") || isAbsolute(relative(stop, current))) break;
    current = dirname(current);
  }
}

/**
 * Prepares a safe path for writing inside a managed root.
 *
 * @param root - Managed root directory.
 * @param child - Relative child path to write.
 * @returns Absolute write path inside the root.
 * @throws When the path escapes the root or traverses a symlinked ancestor.
 */
export function prepareInsideWrite(root: string, child: string): string {
  const target = resolveInside(root, child);
  assertNonSymlinkAncestor(target, root);
  ensureDir(dirname(target));
  assertNonSymlinkAncestor(target, root);
  return target;
}

/**
 * Finds the git repository root for a starting directory.
 *
 * @param start - Directory where `git rev-parse` should run.
 * @returns Git root path, or null when not in a repository or git fails.
 */
export function findGitRoot(start: string): string | null {
  try {
    return execFileSync("git", ["rev-parse", "--show-toplevel"], {
      cwd: start,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return null;
  }
}

/**
 * Resolves the active project root from explicit input or git discovery.
 *
 * @param start - Current working directory used for git discovery.
 * @param explicit - Optional explicit project root path.
 * @returns Realpath-normalized project root.
 */
export function projectRootFrom(start: string, explicit?: string): string {
  const root = explicit ? resolve(expandHome(explicit)) : findGitRoot(start) || resolve(start);
  return realpathSync.native(root);
}

/**
 * Resolves the project-local Kimi UI Agent config directory.
 *
 * @param projectRoot - Project root.
 * @returns Config directory path.
 */
export function projectConfigDir(projectRoot: string): string {
  return join(projectRoot, ".agents", "kimi-ui-agent");
}

/**
 * Resolves the project-local volatile run artifact directory.
 *
 * @param projectRoot - Project root.
 * @returns Run artifact directory path.
 */
export function projectRunsDir(projectRoot: string): string {
  return join(projectConfigDir(projectRoot), "runs");
}

/**
 * Renders a path relative to a root when possible.
 *
 * @param path - Path to display.
 * @param root - Root used for relative display.
 * @returns Relative path when inside the root, otherwise the original path.
 */
export function displayPath(path: string, root: string): string {
  const rel = relative(root, path);
  return rel && !rel.startsWith("..") ? rel : path;
}

/**
 * Checks whether a command is available on the current platform.
 *
 * @param name - Executable name to look up.
 * @returns True when the command can be found.
 */
export function commandExists(name: string): boolean {
  try {
    if (platform() === "win32") {
      execFileSync("where.exe", [name], { stdio: "ignore" });
    } else {
      execFileSync("sh", ["-c", `command -v ${shellQuote(name)} >/dev/null 2>&1`], {
        stdio: "ignore",
      });
    }
    return true;
  } catch {
    return false;
  }
}

/**
 * Runs a command and returns trimmed stdout on success.
 *
 * @param command - Executable to run.
 * @param args - Command arguments.
 * @param cwd - Working directory for the command.
 * @returns Trimmed stdout, or null when the command fails.
 */
export function commandOutput(command: string, args: string[], cwd: string): string | null {
  try {
    return execFileSync(command, args, {
      cwd,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return null;
  }
}

/**
 * Quotes a value for safe POSIX shell display.
 *
 * @param value - Raw value to quote.
 * @returns Single-quoted shell token.
 */
export function shellQuote(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`;
}

/**
 * Generates a safe run identifier from time and random suffix.
 *
 * @returns Safe run id segment.
 */
export function runIdFromTime(): string {
  const stamp = new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14);
  return safeSegment(`run-${stamp}-${Math.random().toString(36).slice(2, 8)}`, "run id");
}

/**
 * Computes a stable short hash for a project root path.
 *
 * @param projectRoot - Project root path to hash.
 * @returns Unsigned hexadecimal hash string.
 */
export function repoHash(projectRoot: string): string {
  let hash = 2166136261;
  for (const char of projectRoot) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16);
}

/**
 * Resolves the parent-controller worktree root for a project.
 *
 * @param projectRoot - Project root.
 * @returns Worktree root path for delegated runs.
 */
export function worktreeRootFor(projectRoot: string): string {
  return join(stateRoot(), "worktrees", `${slugify(basename(projectRoot))}-${repoHash(projectRoot)}`);
}

/**
 * Resolves the parent-controller state file path for a run.
 *
 * @param runId - Run identifier.
 * @returns Absolute run state path.
 */
export function runStatePath(runId: string): string {
  return join(stateRoot(), "runs", safeSegment(runId, "run id"), "run.json");
}
