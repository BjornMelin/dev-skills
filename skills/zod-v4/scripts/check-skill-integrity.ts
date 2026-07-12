#!/usr/bin/env bun
/**
 * Check that SKILL.md <-> rules <-> references <-> scripts/templates stay in sync.
 *
 * Usage:
 *   bun ~/.agents/skills/zod-v4/scripts/check-skill-integrity.ts
 */

import { readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const skillRoot = path.resolve(__dirname, "..");

const skillMdPath = path.join(skillRoot, "SKILL.md");
const rulesDir = path.join(skillRoot, "rules");
const refsDir = path.join(skillRoot, "references");
const refsIndexPath = path.join(refsDir, "index.md");

type Problem = Readonly<{ kind: string; message: string }>;

function existsFile(p: string): boolean {
  const st = statSync(p, { throwIfNoEntry: false });
  return Boolean(st && st.isFile());
}

function listMarkdownFiles(dir: string): readonly string[] {
  return readdirSync(dir)
    .filter((f) => f.endsWith(".md"))
    .sort((a, b) => a.localeCompare(b));
}

function checkInlinePaths(
  problems: Problem[],
  kind: string,
  content: string,
  prefixes: readonly string[],
): void {
  const escaped = prefixes.map((p) => p.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"));
  const re = new RegExp(`\\\`((${escaped.join("|")})/[A-Za-z0-9._/-]+)\\\``, "g");
  for (const m of content.matchAll(re)) {
    const rel = m[1] ?? "";
    const p = path.join(skillRoot, rel);
    if (!existsFile(p)) {
      problems.push({
        kind,
        message: `Inline resource path is missing: ${rel}`,
      });
    }
  }
}

function main(): void {
  const problems: Problem[] = [];

  // 1) SKILL.md: every backticked rule id must exist as rules/<id>.md
  const skillMd = readFileSync(skillMdPath, "utf8");
  const ruleIds = new Set<string>();
  for (const m of skillMd.matchAll(/`([a-z0-9]+(?:-[a-z0-9]+)+)`/g)) {
    const id = m[1] ?? "";
    if (
      id.startsWith("migrate-") ||
      id.startsWith("parse-") ||
      id.startsWith("error-") ||
      id.startsWith("object-") ||
      id.startsWith("schema-") ||
      id.startsWith("meta-") ||
      id.startsWith("jsonschema-") ||
      id.startsWith("codec-") ||
      id.startsWith("package-")
    ) {
      ruleIds.add(id);
    }
  }

  checkInlinePaths(problems, "SKILL.md", skillMd, [
    "rules",
    "references",
    "scripts",
    "assets",
  ]);

  for (const id of ruleIds) {
    const p = path.join(rulesDir, `${id}.md`);
    if (!existsFile(p)) {
      problems.push({
        kind: "SKILL.md",
        message: `Rule id referenced but missing file: ${id} (${p})`,
      });
    }
  }

  // 2) references/index.md: every listed reference must exist
  const refsIndex = readFileSync(refsIndexPath, "utf8");
  for (const m of refsIndex.matchAll(/`(references\/[A-Za-z0-9._-]+\.md)`/g)) {
    const rel = m[1] ?? "";
    const p = path.join(skillRoot, rel);
    if (!existsFile(p)) {
      problems.push({
        kind: "references/index.md",
        message: `Reference listed but missing file: ${rel}`,
      });
    }
  }

  // 3) references/*.md: any `rules/<file>.md` references must exist
  for (const refFile of listMarkdownFiles(refsDir)) {
    if (refFile === "index.md") continue;
    const p = path.join(refsDir, refFile);
    const content = readFileSync(p, "utf8");
    checkInlinePaths(problems, `references/${refFile}`, content, [
      "rules",
      "references",
      "scripts",
      "assets",
    ]);
    for (const m of content.matchAll(/`(rules\/[A-Za-z0-9._-]+\.md)`/g)) {
      const rel = m[1] ?? "";
      const abs = path.join(skillRoot, rel);
      if (!existsFile(abs)) {
        problems.push({
          kind: `references/${refFile}`,
          message: `Broken rules link: ${rel}`,
        });
      }
    }
  }

  // 4) rules: ensure every rule file has a title in frontmatter (best-effort)
  for (const ruleFile of listMarkdownFiles(rulesDir)) {
    if (ruleFile === "_index.md") continue;
    const p = path.join(rulesDir, ruleFile);
    const content = readFileSync(p, "utf8");
    const fm = content.match(/^---\n([\s\S]*?)\n---\n/m);
    if (!fm || !/\btitle\s*:\s*/.test(fm[1] ?? "")) {
      problems.push({
        kind: `rules/${ruleFile}`,
        message: "Missing YAML frontmatter title:",
      });
    }
  }

  if (problems.length === 0) {
    process.stdout.write("OK: skill integrity checks passed.\n");
    return;
  }

  for (const p of problems) {
    process.stdout.write(`[${p.kind}] ${p.message}\n`);
  }
  process.exit(1);
}

main();
