#!/usr/bin/env node
/**
 * Vitest sharding runner.
 *
 * Uses env vars:
 * - VITEST_SHARD_INDEX (1-based)
 * - VITEST_SHARD_TOTAL
 *
 * Example:
 *   VITEST_SHARD_INDEX=1 VITEST_SHARD_TOTAL=4 node scripts/vitest-shard.mjs
 *
 * Writes a blob report so results can be merged.
 * See:
 * - blob reporter & merge: https://vitest.dev/guide/reporters
 */
import { spawnSync } from 'node:child_process'
import { mkdirSync } from 'node:fs'

const index = Number(process.env.VITEST_SHARD_INDEX || '')
const total = Number(process.env.VITEST_SHARD_TOTAL || '')

if (!Number.isFinite(index) || !Number.isFinite(total) || index < 1 || total < 1 || index > total) {
  console.error('Invalid shard settings. Provide VITEST_SHARD_INDEX (1..N) and VITEST_SHARD_TOTAL (N).')
  process.exit(2)
}

mkdirSync('reports', { recursive: true })

const shardArg = `--shard=${index}/${total}`
const outputFile = `reports/blob-${index}-of-${total}.json`

const args = [
  'vitest',
  'run',
  shardArg,
  '--reporter=blob',
  `--outputFile=${outputFile}`,
]

// Inherit stdio for CI logs
const res = spawnSync('npx', args, { stdio: 'inherit' })
process.exit(res.status ?? 1)
