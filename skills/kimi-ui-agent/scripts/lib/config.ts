import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";
import type { ManagedWrite, ProjectConfig } from "./types";
import { assertNonSymlinkAncestor, ensureDir, nowIso, prepareInsideWrite, projectConfigDir, projectRunsDir, slugify } from "./paths";

export const CONFIG_FILE = "config.json";
export const PROFILE_LOCK_FILE = "profile.lock.json";
export const RUNS_GITIGNORE_REL = ".agents/kimi-ui-agent/runs/.gitignore";
export const RUNS_GITIGNORE_CONTENT = "*\n!.gitignore\n";

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

export function configPath(projectRoot: string): string {
  return join(projectConfigDir(projectRoot), CONFIG_FILE);
}

export function loadConfig(projectRoot: string): ProjectConfig | null {
  const path = configPath(projectRoot);
  if (!existsSync(path)) return null;
  const parsed = JSON.parse(readFileSync(path, "utf8")) as ProjectConfig;
  if (parsed.schema !== "kimi_ui_agent.project.v1") {
    throw new Error(`unsupported project config schema in ${path}`);
  }
  return parsed;
}

export function managedHeader(relativePath: string): string {
  return `<!-- ${relativePath}: Managed by kimi-ui-agent. Edit project-specific facts, but keep this marker. -->\n\n`;
}

export function writeManaged(projectRoot: string, writes: ManagedWrite[], apply: boolean): ManagedWrite[] {
  if (!apply) return writes;
  for (const write of writes) {
    if (write.action === "skip" || write.content === undefined) continue;
    const target = prepareInsideWrite(projectRoot, write.path);
    writeFileSync(target, write.content, { encoding: "utf8", mode: 0o600 });
  }
  return writes;
}

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

export function ensureProfileDirs(projectRoot: string): void {
  const configDir = projectConfigDir(projectRoot);
  assertNonSymlinkAncestor(configDir, projectRoot);
  ensureDir(configDir);
  const runsDir = projectRunsDir(projectRoot);
  assertNonSymlinkAncestor(runsDir, projectRoot);
  ensureDir(runsDir);
}
