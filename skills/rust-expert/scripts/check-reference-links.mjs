#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const roots = process.argv.slice(2);
if (roots.length === 0) {
  console.error("Usage: check-reference-links.mjs <skill-dir...>");
  process.exit(2);
}

let ok = true;
for (const rootArg of roots) {
  const skillDir = path.resolve(rootArg);
  for (const file of listMarkdown(skillDir)) {
    const text = fs.readFileSync(file, "utf8");
    const relFile = path.relative(skillDir, file);
    const patterns = [
      /`((?:references|scripts|assets|templates)\/[^`]+)`/g,
      /\[[^\]]+\]\(((?:references|scripts|assets|templates)\/[^)]+)\)/g,
    ];
    for (const pattern of patterns) {
      for (const match of text.matchAll(pattern)) {
        const target = match[1].split("#")[0].split(/\s+/)[0];
        const full = path.resolve(skillDir, target);
        const insideSkill = full === skillDir || full.startsWith(`${skillDir}${path.sep}`);
        if (!insideSkill || !fs.existsSync(full)) {
          ok = false;
          console.error(`${path.basename(skillDir)}/${relFile}: missing linked file ${target}`);
        }
      }
    }
  }
}

if (!ok) process.exit(1);
console.log("skill reference links OK");

function listMarkdown(dir) {
  const out = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...listMarkdown(full));
    else if (entry.name.endsWith(".md")) out.push(full);
  }
  return out;
}
