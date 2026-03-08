#!/usr/bin/env bun

import { createSkillContext, runCli } from './lib/bun-platform-core';

await runCli(createSkillContext(import.meta.url), process.argv.slice(2));
