#!/usr/bin/env node
/**
 * Merge Vitest blob reports.
 *
 * Expects blob JSON files in ./reports.
 * See: https://vitest.dev/guide/reporters
 *
 * Usage:
 *   node scripts/vitest-merge-reports.mjs
 */
import { spawnSync } from 'node:child_process'

const args = [
  'vitest',
  '--merge-reports=reports',
  '--reporter=default',
  '--reporter=json',
  '--reporter=junit',
  '--outputFile.json=reports/merged-results.json',
  '--outputFile.junit=reports/merged-junit.xml',
]

const res = spawnSync('npx', args, { stdio: 'inherit' })
process.exit(res.status ?? 1)
