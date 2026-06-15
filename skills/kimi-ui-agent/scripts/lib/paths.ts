import { lstatSync, mkdirSync, realpathSync } from "node:fs";
import { homedir } from "node:os";
import { basename, dirname, isAbsolute, join, relative, resolve } from "node:path";
import { execFileSync } from "node:child_process";

export const MANAGED_MARKER = "Managed by kimi-ui-agent";

export function nowIso(): string {
  return new Date().toISOString();
}

export function expandHome(input: string): string {
  if (input === "~") return homedir();
  if (input.startsWith("~/")) return join(homedir(), input.slice(2));
  return input;
}

export function xdgStateHome(): string {
  return process.env.XDG_STATE_HOME || join(homedir(), ".local", "state");
}

export function stateRoot(): string {
  return join(xdgStateHome(), "dev-skills", "kimi-ui-agent");
}

export function safeSegment(input: string, label = "value"): string {
  if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,79}$/.test(input)) {
    throw new Error(`${label} must be a safe single path segment`);
  }
  if (input === "." || input === ".." || input.includes("..")) {
    throw new Error(`${label} must not contain parent traversal`);
  }
  return input;
}

export function slugify(input: string, fallback = "project"): string {
  const slug = input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
  return slug || fallback;
}

export function ensureDir(path: string): void {
  mkdirSync(path, { recursive: true, mode: 0o700 });
}

export function resolveInside(root: string, child: string): string {
  const rootAbs = resolve(expandHome(root));
  const childAbs = resolve(rootAbs, child);
  const rel = relative(rootAbs, childAbs);
  if (rel === "" || (!rel.startsWith("..") && !isAbsolute(rel))) {
    return childAbs;
  }
  throw new Error(`path escapes managed root: ${child}`);
}

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

export function prepareInsideWrite(root: string, child: string): string {
  const target = resolveInside(root, child);
  assertNonSymlinkAncestor(target, root);
  ensureDir(dirname(target));
  assertNonSymlinkAncestor(target, root);
  return target;
}

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

export function projectRootFrom(start: string, explicit?: string): string {
  const root = explicit ? resolve(expandHome(explicit)) : findGitRoot(start) || resolve(start);
  return realpathSync.native(root);
}

export function projectConfigDir(projectRoot: string): string {
  return join(projectRoot, ".agents", "kimi-ui-agent");
}

export function projectRunsDir(projectRoot: string): string {
  return join(projectConfigDir(projectRoot), "runs");
}

export function displayPath(path: string, root: string): string {
  const rel = relative(root, path);
  return rel && !rel.startsWith("..") ? rel : path;
}

export function commandExists(name: string): boolean {
  try {
    execFileSync("sh", ["-lc", `command -v ${shellQuote(name)} >/dev/null 2>&1`], {
      stdio: "ignore",
    });
    return true;
  } catch {
    return false;
  }
}

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

export function shellQuote(value: string): string {
  return `'${value.replace(/'/g, `'\\''`)}'`;
}

export function runIdFromTime(): string {
  const stamp = new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14);
  return safeSegment(`run-${stamp}-${Math.random().toString(36).slice(2, 8)}`, "run id");
}

export function repoHash(projectRoot: string): string {
  let hash = 2166136261;
  for (const char of projectRoot) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16);
}

export function worktreeRootFor(projectRoot: string): string {
  return join(stateRoot(), "worktrees", `${slugify(basename(projectRoot))}-${repoHash(projectRoot)}`);
}

export function runStatePath(runId: string): string {
  return join(stateRoot(), "runs", safeSegment(runId, "run id"), "run.json");
}
