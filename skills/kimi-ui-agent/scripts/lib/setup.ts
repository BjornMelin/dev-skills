import { existsSync, lstatSync, readFileSync, readdirSync } from "node:fs";
import { basename, join } from "node:path";
import type { ManagedWrite, ProjectConfig, ProjectScan } from "./types";
import { displayPath } from "./paths";
import { managedHeader } from "./config";
import { baseProfileWrites, defaultConfig, PROFILE_LOCK_FILE } from "./config";

function readJsonIfExists(path: string): Record<string, unknown> | null {
  if (!existsSync(path)) return null;
  try {
    return JSON.parse(readFileSync(path, "utf8")) as Record<string, unknown>;
  } catch {
    return null;
  }
}

function hasDep(pkg: Record<string, unknown> | null, name: string): boolean {
  const scopes = ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];
  return scopes.some((scope) => {
    const deps = pkg?.[scope];
    return Boolean(deps && typeof deps === "object" && name in deps);
  });
}

function listDirs(projectRoot: string, candidates: string[]): string[] {
  return candidates.filter((candidate) => {
    const full = join(projectRoot, candidate);
    try {
      const st = lstatSync(full);
      return st.isDirectory() && !st.isSymbolicLink();
    } catch {
      return false;
    }
  });
}

function scanForDirs(projectRoot: string, names: string[], maxDepth = 4): string[] {
  const found: string[] = [];
  const ignored = new Set([
    ".git",
    ".agents",
    ".claude",
    ".codex",
    ".kimi-code",
    "node_modules",
    "target",
    ".next",
    "dist",
    "build",
    ".turbo",
  ]);
  function walk(dir: string, depth: number): void {
    if (depth > maxDepth || found.length >= 40) return;
    let entries: string[] = [];
    try {
      entries = readdirSync(dir);
    } catch {
      return;
    }
    for (const entry of entries) {
      if (ignored.has(entry)) continue;
      const full = join(dir, entry);
      let st;
      try {
        st = lstatSync(full);
      } catch {
        continue;
      }
      if (!st.isDirectory() || st.isSymbolicLink()) continue;
      const rel = displayPath(full, projectRoot);
      if (names.includes(entry) || names.some((name) => rel.endsWith(`/${name}`))) {
        found.push(rel);
      }
      walk(full, depth + 1);
    }
  }
  walk(projectRoot, 0);
  return Array.from(new Set(found)).sort();
}

export function scanProject(projectRoot: string): ProjectScan {
  const pkg = readJsonIfExists(join(projectRoot, "package.json"));
  const frameworks = new Set<string>();
  const uiLibraries = new Set<string>();
  const styling = new Set<string>();
  const validationCommands = new Set<string>();

  if (hasDep(pkg, "next")) frameworks.add("Next.js");
  if (hasDep(pkg, "react")) frameworks.add("React");
  if (hasDep(pkg, "vite")) frameworks.add("Vite");
  if (hasDep(pkg, "expo")) frameworks.add("Expo");
  if (hasDep(pkg, "@storybook/react") || hasDep(pkg, "storybook")) frameworks.add("Storybook");

  if (hasDep(pkg, "@radix-ui/react-slot") || hasDep(pkg, "class-variance-authority")) uiLibraries.add("shadcn/Radix-style components");
  if (hasDep(pkg, "@mui/material")) uiLibraries.add("MUI");
  if (hasDep(pkg, "antd")) uiLibraries.add("Ant Design");
  if (hasDep(pkg, "@mantine/core")) uiLibraries.add("Mantine");

  if (hasDep(pkg, "tailwindcss") || existsSync(join(projectRoot, "tailwind.config.ts")) || existsSync(join(projectRoot, "tailwind.config.js"))) styling.add("Tailwind CSS");
  if (hasDep(pkg, "styled-components")) styling.add("styled-components");
  if (hasDep(pkg, "@emotion/react")) styling.add("Emotion");
  if (hasDep(pkg, "sass")) styling.add("Sass");

  const scripts = (pkg?.scripts && typeof pkg.scripts === "object" ? pkg.scripts : {}) as Record<string, unknown>;
  for (const key of ["lint", "typecheck", "test", "build", "storybook", "test:e2e"]) {
    if (typeof scripts[key] === "string") {
      validationCommands.add(`${packageRunner(projectRoot)} ${key}`);
    }
  }

  const packageManager = detectPackageManager(projectRoot);
  const appDirs = listDirs(projectRoot, ["app", "pages", "src/app", "src/pages", "src/routes", "routes"]);
  const componentDirs = scanForDirs(projectRoot, ["components", "ui"]);
  const protectedPathHints = [".env*", ".github/workflows/**", "infra/**", "migrations/**", "secrets/**"];

  return {
    projectRoot,
    projectName: basename(projectRoot),
    packageManager,
    frameworks: Array.from(frameworks).sort(),
    uiLibraries: Array.from(uiLibraries).sort(),
    styling: Array.from(styling).sort(),
    appDirs,
    componentDirs,
    validationCommands: Array.from(validationCommands).sort(),
    protectedPathHints,
  };
}

export function detectPackageManager(projectRoot: string): string | null {
  if (existsSync(join(projectRoot, "bun.lock")) || existsSync(join(projectRoot, "bun.lockb"))) return "bun";
  if (existsSync(join(projectRoot, "pnpm-lock.yaml"))) return "pnpm";
  if (existsSync(join(projectRoot, "yarn.lock"))) return "yarn";
  if (existsSync(join(projectRoot, "package-lock.json"))) return "npm";
  const packageManager = readJsonIfExists(join(projectRoot, "package.json"))?.packageManager;
  if (typeof packageManager === "string") {
    const name = packageManager.trim().split("@", 1)[0] || "";
    if (["bun", "pnpm", "yarn", "npm"].includes(name)) return name;
  }
  return null;
}

function packageRunner(projectRoot: string): string {
  const pm = detectPackageManager(projectRoot);
  if (pm === "bun") return "bun run";
  if (pm === "pnpm") return "pnpm";
  if (pm === "yarn") return "yarn";
  if (pm === "npm") return "npm run";
  return "bun run";
}

function editableProfileWrite(projectRoot: string, write: ManagedWrite): ManagedWrite {
  if (existsSync(join(projectRoot, write.path))) {
    return {
      path: write.path,
      action: "skip",
      reason: `preserve existing editable ${basename(write.path)}`,
    };
  }
  return write;
}

function portableScan(scan: ProjectScan): Omit<ProjectScan, "projectRoot"> {
  const { projectRoot: _projectRoot, ...portable } = scan;
  return portable;
}

export function profileWrites(projectRoot: string, existing?: ProjectConfig | null): ManagedWrite[] {
  const scan = scanProject(projectRoot);
  const config = existing || defaultConfig(projectRoot);
  const markdownList = (items: string[]) => (items.length ? items.map((item) => `- ${item}`).join("\n") : "- none detected");
  const writes: ManagedWrite[] = [
    ...baseProfileWrites(projectRoot, { ...config, updatedAt: new Date().toISOString() }),
    editableProfileWrite(projectRoot, {
      path: ".agents/kimi-ui-agent/project-profile.md",
      action: "create",
      reason: "durable high-level repo profile for active harness refinement",
      content: `${managedHeader("project-profile.md")}# ${scan.projectName} UI Profile\n\n## Stack\n\n- Package manager: ${scan.packageManager || "unknown"}\n- Frameworks:\n${markdownList(scan.frameworks)}\n- UI libraries:\n${markdownList(scan.uiLibraries)}\n- Styling:\n${markdownList(scan.styling)}\n\n## Harness Refinement Notes\n\nAdd project-specific product, UX, and implementation context here after Codex, Claude Code, or Kimi Code reviews the deterministic scan.\n`,
    }),
    editableProfileWrite(projectRoot, {
      path: ".agents/kimi-ui-agent/frontend-map.md",
      action: "create",
      reason: "front-end directory map for future orchestration runs",
      content: `${managedHeader("frontend-map.md")}# Frontend Map\n\n## App and Route Directories\n\n${markdownList(scan.appDirs)}\n\n## Component Directories\n\n${markdownList(scan.componentDirs)}\n\n## Notes\n\nKeep this lean. Add only high-signal ownership boundaries, route conventions, or component families that future UI agents should know.\n`,
    }),
    editableProfileWrite(projectRoot, {
      path: ".agents/kimi-ui-agent/design-system.md",
      action: "create",
      reason: "design system and UI quality context",
      content: `${managedHeader("design-system.md")}# Design System\n\n## Detected Styling\n\n${markdownList(scan.styling)}\n\n## Detected UI Libraries\n\n${markdownList(scan.uiLibraries)}\n\n## Project-Specific Rules\n\nDocument tokens, density, component conventions, accessibility rules, and visual QA expectations here.\n`,
    }),
    editableProfileWrite(projectRoot, {
      path: ".agents/kimi-ui-agent/verification.md",
      action: "create",
      reason: "validation command catalog for agent runs",
      content: `${managedHeader("verification.md")}# Verification\n\n## Detected Commands\n\n${markdownList(scan.validationCommands)}\n\n## Visual QA\n\nDocument screenshot, Playwright, Storybook, browser, or device checks here.\n`,
    }),
    editableProfileWrite(projectRoot, {
      path: ".agents/kimi-ui-agent/protected-paths.md",
      action: "create",
      reason: "human-readable protected path policy",
      content: `${managedHeader("protected-paths.md")}# Protected Paths\n\n## Configured Protected Paths\n\n${markdownList(config.protectedPaths)}\n\n## Project Overrides\n\nUpdate \`.agents/kimi-ui-agent/config.json\` for machine-readable protected paths, then refresh this Markdown policy if the human-readable view should change.\n`,
    }),
    {
      path: `.agents/kimi-ui-agent/${PROFILE_LOCK_FILE}`,
      action: "create",
      reason: "profile freshness evidence for doctor",
      content: `${JSON.stringify(
        {
          schema: "kimi_ui_agent.profile_lock.v1",
          generatedAt: new Date().toISOString(),
          scan: portableScan(scan),
        },
        null,
        2,
      )}\n`,
    },
  ];
  return writes;
}
