#!/usr/bin/env node
/**
 * Simple flake detector: runs `vitest run` multiple times.
 *
 * Env vars:
 * - FLAKE_RUNS (default 10)
 * - FLAKE_FILTER (optional: passed as -t <pattern>)
 *
 * Usage:
 *   FLAKE_RUNS=20 node scripts/vitest-flake-check.mjs
 */
import { spawnSync } from 'node:child_process'

const runs = Number(process.env.FLAKE_RUNS || '10')
const filter = process.env.FLAKE_FILTER

if (!Number.isFinite(runs) || runs < 1) {
  console.error('Invalid FLAKE_RUNS; must be a positive number.')
  process.exit(2)
}

for (let i = 1; i <= runs; i++) {
  console.log(`\n=== Flake run ${i}/${runs} ===\n`)
  const args = ['vitest', 'run']
  if (filter) args.push('-t', filter)

  const res = spawnSync('npx', args, { stdio: 'inherit' })
  if ((res.status ?? 1) !== 0) {
    console.error(`\nFlake detected on run ${i}/${runs}.`)
    process.exit(res.status ?? 1)
  }
}

console.log(`\nNo failures in ${runs} runs.`)
