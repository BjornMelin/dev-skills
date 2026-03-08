#!/usr/bin/env bun

/**
 * Compatibility wrapper for the Bun platform audit CLI.
 *
 * This keeps the long-lived bun-dev entrypoint stable while delegating to the
 * shared platform engine.
 */

import { createSkillContext, runCli } from './lib/bun-platform-core';

await runCli(createSkillContext(import.meta.url), process.argv.slice(2));
