#!/usr/bin/env bun
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { runCli } from "./lib/cli";

const scriptDir = dirname(fileURLToPath(import.meta.url));
process.env.KIMI_UI_AGENT_SKILL_DIR ||= resolve(scriptDir, "..");

const exitCode = await runCli(process.argv.slice(2));
process.exit(exitCode);
