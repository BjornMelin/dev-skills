import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import type { ManagedWrite, ProjectConfig } from "./types";
import { assertNonSymlinkAncestor, ensureDir, nowIso, prepareInsideWrite, projectConfigDir, projectRunsDir, slugify } from "./paths";

/** Project config filename under `.agents/kimi-ui-agent`. */
export const CONFIG_FILE = "config.json";
/** Project profile freshness lock filename under `.agents/kimi-ui-agent`. */
export const PROFILE_LOCK_FILE = "profile.lock.json";
/** Git ignore file path for volatile run artifacts. */
export const RUNS_GITIGNORE_REL = ".agents/kimi-ui-agent/runs/.gitignore";
/** Git ignore contents that keep run artifacts out while preserving the ignore file. */
export const RUNS_GITIGNORE_CONTENT = "*\n!.gitignore\n";

/**
 * Builds the default project-local Kimi UI Agent policy.
 *
 * @param projectRoot - Project root used to derive the display project name.
 * @returns Default project configuration.
 */
export function defaultConfig(projectRoot: string): ProjectConfig {
  const now = nowIso();
  return {
    schema: "kimi_ui_agent.project.v1",
    projectName: slugify(basename(projectRoot)),
    createdAt: now,
    updatedAt: now,
    protectedPaths: [
      ".env",
      ".env.*",
      ".github/workflows/**",
      "infra/**",
      "migrations/**",
      "secrets/**",
      "**/*secret*",
      "**/*token*",
      "**/bun.lock",
      "**/bun.lockb",
      "**/package-lock.json",
      "**/pnpm-lock.yaml",
      "**/yarn.lock",
    ],
    branchPrefix: "kimi-ui",
    redaction: {
      extraPatterns: [],
    },
  };
}

/**
 * Resolves the project-local config path.
 *
 * @param projectRoot - Project root containing `.agents/kimi-ui-agent`.
 * @returns Absolute config file path.
 */
export function configPath(projectRoot: string): string {
  return join(projectConfigDir(projectRoot), CONFIG_FILE);
}

/**
 * Loads and validates the project-local config when present.
 *
 * @param projectRoot - Project root containing optional Kimi UI Agent config.
 * @returns Parsed project config, or null when no config exists.
 * @throws When the config schema is unsupported.
 */
export function loadConfig(projectRoot: string): ProjectConfig | null {
  const path = configPath(projectRoot);
  if (!existsSync(path)) return null;
  const parsed = JSON.parse(readFileSync(path, "utf8")) as ProjectConfig;
  if (parsed.schema !== "kimi_ui_agent.project.v1") {
    throw new Error(`unsupported project config schema in ${path}`);
  }
  return parsed;
}

/**
 * Renders the managed-file header for generated editable profile files.
 *
 * @param relativePath - Repo-relative path represented by the generated file.
 * @returns Header text to prepend to managed files.
 */
export function managedHeader(relativePath: string): string {
  return `<!-- ${relativePath}: Managed by kimi-ui-agent. Edit project-specific facts, but keep this marker. -->\n\n`;
}

/**
 * Applies or previews managed writes under a project root.
 *
 * @param projectRoot - Project root receiving managed writes.
 * @param writes - Planned file writes to process.
 * @param apply - Whether to write files or only return the plan.
 * @returns The original managed write plan.
 */
export function writeManaged(projectRoot: string, writes: ManagedWrite[], apply: boolean): ManagedWrite[] {
  if (!apply) return writes;
  for (const write of writes) {
    if (write.action === "skip" || write.content === undefined) continue;
    const target = prepareInsideWrite(projectRoot, write.path);
    writeFileSync(target, write.content, { encoding: "utf8", mode: 0o600 });
  }
  return writes;
}

/**
 * Builds the baseline generated project profile files.
 *
 * @param projectRoot - Project root for generated context paths.
 * @param config - Project config to serialize.
 * @returns Managed writes for config and run ignore policy.
 */
export function baseProfileWrites(projectRoot: string, config: ProjectConfig): ManagedWrite[] {
  const configRel = ".agents/kimi-ui-agent/config.json";
  return [
    {
      path: configRel,
      action: "create",
      reason: "canonical project policy",
      content: `${JSON.stringify(config, null, 2)}\n`,
    },
    {
      path: RUNS_GITIGNORE_REL,
      action: "create",
      reason: "keep volatile run artifacts out of version control without root ignore edits",
      content: RUNS_GITIGNORE_CONTENT,
    },
  ];
}

/**
 * Ensures the project-local Kimi UI Agent config and run directories exist.
 *
 * @param projectRoot - Project root containing `.agents/kimi-ui-agent`.
 */
export function ensureProfileDirs(projectRoot: string): void {
  const configDir = projectConfigDir(projectRoot);
  assertNonSymlinkAncestor(configDir, projectRoot);
  ensureDir(configDir);
  const runsDir = projectRunsDir(projectRoot);
  assertNonSymlinkAncestor(runsDir, projectRoot);
  ensureDir(runsDir);
}
