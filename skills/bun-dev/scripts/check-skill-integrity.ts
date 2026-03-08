#!/usr/bin/env bun
/**
 * Skill integrity checks:
 * - All rule IDs referenced in `SKILL.md` exist in `rules/`.
 * - `references/index.md` exists.
 * - `rules/_index.md` exists.
 * - `references/` is flat (no subdirectories).
 *
 * Run:
 *   bun ~/.agents/skills/bun-dev/scripts/check-skill-integrity.ts
 */

import { readFile, readdir, stat } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const skillRoot = join(__filename, "..", "..");

function extractBacktickedIds(md: string): string[] {
  const out: string[] = [];
  const re = /`([a-z0-9-]+)`/g;
  for (;;) {
    const m = re.exec(md);
    if (!m) break;
    out.push(m[1]);
  }
  return out;
}

async function fileExists(path: string): Promise<boolean> {
  try {
    await stat(path);
    return true;
  } catch {
    return false;
  }
}

async function main() {
  const skillMdPath = join(skillRoot, "SKILL.md");
  const rulesDir = join(skillRoot, "rules");
  const refsDir = join(skillRoot, "references");

  const skillMd = await readFile(skillMdPath, "utf8");
  const ids = new Set(extractBacktickedIds(skillMd));

  // Only validate ids that look like rule IDs by prefix.
  const rulePrefixes = [
    "pm-",
    "runtime-",
    "vercel-",
    "scripts-",
    "tsconfig-",
    "test-",
    "build-",
    "perf-",
    "migrate-",
    "troubleshooting-",
  ] as const;

  const referencedRuleIds = [...ids].filter((id) => {
    // Ignore backticked prefix strings like `pm-` used in tables.
    if (id.endsWith("-")) return false;
    // Must be a plausible rule id: at least one hyphen segment.
    if (!/^[a-z0-9]+(?:-[a-z0-9]+)+$/.test(id)) return false;
    return rulePrefixes.some((p) => id.startsWith(p));
  });

  const ruleFiles = (await readdir(rulesDir)).filter((f) => f.endsWith(".md"));
  const existingRuleIds = new Set(ruleFiles.map((f) => f.replace(/\.md$/, "")));

  const missing = referencedRuleIds.filter((id) => !existingRuleIds.has(id));
  if (missing.length > 0) {
    throw new Error(`Missing rule files for ids:\n${missing.map((m) => `- ${m}`).join("\n")}`);
  }

  if (!(await fileExists(join(rulesDir, "_index.md")))) {
    throw new Error("Missing rules/_index.md (run build-rules-index.ts)");
  }
  if (!(await fileExists(join(refsDir, "index.md")))) {
    throw new Error("Missing references/index.md");
  }

  // Ensure references are flat (no subdirectories).
  for (const name of await readdir(refsDir)) {
    const p = join(refsDir, name);
    const s = await stat(p);
    if (s.isDirectory()) {
      throw new Error(`references/ must be flat; found directory: references/${name}`);
    }
  }

  console.log("OK: bun-dev skill integrity checks passed.");
}

await main();

