#!/usr/bin/env bun
/**
 * Deterministically rebuild `rules/_index.md`.
 *
 * Run:
 *   bun ~/.agents/skills/bun-dev/scripts/build-rules-index.ts
 */

import { readdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const skillRoot = join(__filename, "..", "..");
const rulesDir = join(skillRoot, "rules");

function isRuleFile(name: string): boolean {
  return name.endsWith(".md") && name !== "_index.md";
}

function ruleIdFromFile(name: string): string {
  return name.replace(/\.md$/, "");
}

async function main() {
  const names = (await readdir(rulesDir)).filter(isRuleFile).sort((a, b) => a.localeCompare(b));

  const lines: string[] = [];
  lines.push("# Rules Index");
  lines.push("");
  lines.push("Open `SKILL.md` first to route by priority. Prefer opening specific rules over references.");
  lines.push("");

  const groups: Array<{ title: string; prefix: string }> = [
    { title: "Package Manager + Lockfiles (P1)", prefix: "pm-" },
    { title: "Runtime Selection (P1)", prefix: "runtime-" },
    { title: "Vercel Bun Runtime (P1)", prefix: "vercel-" },
    { title: "Scripts + Monorepos (P2)", prefix: "scripts-" },
    { title: "TypeScript + Tooling (P2)", prefix: "tsconfig-" },
    { title: "Testing (P3)", prefix: "test-" },
    { title: "Build + Bundling (P3)", prefix: "build-" },
    { title: "Performance (P4)", prefix: "perf-" },
    { title: "Migration (P5)", prefix: "migrate-" },
    { title: "Troubleshooting (P5)", prefix: "troubleshooting-" },
  ];

  const remaining = new Set(names);

  for (const g of groups) {
    const groupNames = names.filter((n) => n.startsWith(g.prefix));
    if (groupNames.length === 0) continue;
    lines.push(`## ${g.title}`);
    lines.push("");
    for (const name of groupNames) {
      remaining.delete(name);
      lines.push(`- \`${ruleIdFromFile(name)}\``);
    }
    lines.push("");
  }

  const leftovers = [...remaining].sort((a, b) => a.localeCompare(b));
  if (leftovers.length > 0) {
    lines.push("## Other");
    lines.push("");
    for (const name of leftovers) lines.push(`- \`${ruleIdFromFile(name)}\``);
    lines.push("");
  }

  await writeFile(join(rulesDir, "_index.md"), `${lines.join("\n")}\n`, "utf8");
}

await main();

