#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";

const roots = process.argv.slice(2);
if (roots.length === 0) {
  console.error("Usage: check-trigger-evals.mjs <skill-dir...>");
  process.exit(2);
}

let ok = true;
for (const rootArg of roots) {
  const skillDir = path.resolve(rootArg);
  const skillName = path.basename(skillDir);
  const evalPath = path.join(skillDir, "assets", "trigger-evals.json");
  if (!fs.existsSync(evalPath)) {
    ok = false;
    console.error(`${skillName}: missing assets/trigger-evals.json`);
    continue;
  }
  const skillPath = path.join(skillDir, "SKILL.md");
  const openaiPath = path.join(skillDir, "agents", "openai.yaml");
  const skillText = fs.existsSync(skillPath) ? fs.readFileSync(skillPath, "utf8") : "";
  const openaiText = fs.existsSync(openaiPath) ? fs.readFileSync(openaiPath, "utf8") : "";
  const explicitOnly =
    /\binvocation:\s*explicit-only\b/.test(skillText) ||
    /\ballow_implicit_invocation:\s*false\b/.test(openaiText);
  let data;
  try {
    data = JSON.parse(fs.readFileSync(evalPath, "utf8"));
  } catch (error) {
    ok = false;
    console.error(`${skillName}: invalid JSON: ${error.message}`);
    continue;
  }
  if (!Array.isArray(data) || data.length < 6) {
    ok = false;
    console.error(`${skillName}: expected at least 6 eval cases`);
    continue;
  }
  const positives = data.filter((item) => item?.should_trigger === true).length;
  const negatives = data.filter((item) => item?.should_trigger === false).length;
  if (positives < 3 || negatives < 3) {
    ok = false;
    console.error(`${skillName}: expected at least 3 positive and 3 negative evals`);
  }
  for (const [index, item] of data.entries()) {
    if (typeof item.query !== "string" || item.query.length < 10) {
      ok = false;
      console.error(`${skillName}[${index}]: missing realistic query`);
    }
    if (typeof item.should_trigger !== "boolean") {
      ok = false;
      console.error(`${skillName}[${index}]: should_trigger must be boolean`);
    }
    if (typeof item.reason !== "string" || item.reason.length < 8) {
      ok = false;
      console.error(`${skillName}[${index}]: missing reason`);
    }
    if (
      explicitOnly &&
      item.should_trigger === true &&
      !item.query.includes(skillName) &&
      !item.query.includes(`$${skillName}`)
    ) {
      ok = false;
      console.error(`${skillName}[${index}]: explicit-only positive eval must name ${skillName}`);
    }
  }
}

if (!ok) process.exit(1);
console.log("trigger eval fixtures OK");
