#!/usr/bin/env node

import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { resolve } from 'node:path';
import process from 'node:process';

function printHelp() {
  console.log(`Usage: convex-test-capability-check.mjs [paths...] [options]

Static guardrail check for Signr convex-test suites.

Options:
  --strict-warnings       Exit non-zero on warnings as well as errors.
  --help                  Show this help.
`);
}

function parseArgs(argv) {
  const options = {
    paths: [],
    strictWarnings: false,
  };

  for (const arg of argv) {
    if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else if (arg === '--strict-warnings') {
      options.strictWarnings = true;
    } else if (arg.startsWith('--')) {
      throw new Error(`Unknown option: ${arg}`);
    } else {
      options.paths.push(arg);
    }
  }

  if (!options.paths.length) {
    options.paths = ['packages/backend/test/convex'];
  }
  return options;
}

function filesFromPath(inputPath) {
  const absolutePath = resolve(inputPath);
  if (!existsSync(absolutePath)) {
    return [];
  }
  const stat = statSync(absolutePath);
  if (stat.isFile()) {
    return [absolutePath];
  }
  const files = [];
  for (const name of readdirSync(absolutePath)) {
    files.push(...filesFromPath(`${absolutePath}/${name}`));
  }
  return files;
}

function toPosix(path) {
  return path.replaceAll('\\', '/');
}

function isTestFile(path) {
  return path.endsWith('.test.ts') || path.endsWith('.convex-test.ts');
}

function checkFile(filePath) {
  const relativePath = toPosix(process.cwd() === '/' ? filePath : filePath.replace(`${process.cwd()}/`, ''));
  const text = readFileSync(filePath, 'utf8');
  const errors = [];
  const warnings = [];

  if (relativePath.includes('packages/backend/convex/') && relativePath.endsWith('.test.ts')) {
    errors.push('Convex tests must live under packages/backend/test/convex/**, not packages/backend/convex/**.');
  }
  if (relativePath.endsWith('.convex-test.ts')) {
    errors.push('Do not use .convex-test.ts suffixes; use *.test.ts under packages/backend/test/convex/**.');
  }
  if (/@vitest-environment\b/.test(text)) {
    errors.push('Do not add file-level @vitest-environment pragmas; backend-convex Vitest owns the environment.');
  }
  if (/\bconvexTest\s*\(/.test(text)) {
    if (!/from ['"].*convex\/schema['"]/.test(text)) {
      warnings.push('convexTest test does not import schema from ../../../convex/schema.');
    }
    if (!/convexModules/.test(text)) {
      warnings.push('convexTest test does not reference convexModules.');
    }
  }
  if (/finish(?:All|InProgress)ScheduledFunctions/.test(text) && !/vi\.useFakeTimers\s*\(/.test(text)) {
    warnings.push('Scheduled-function drain appears without vi.useFakeTimers().');
  }
  if (/new Promise\s*\([^)]*setTimeout|setTimeout\s*\(/s.test(text)) {
    warnings.push('Avoid wall-clock sleeps in Convex tests; use fake timers or state assertions.');
  }

  return { errors, file: relativePath, warnings };
}

function main() {
  let options;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    console.error('Run with --help for usage.');
    process.exit(2);
  }

  if (options.help) {
    printHelp();
    return;
  }

  const files = options.paths.flatMap(filesFromPath).filter(isTestFile);
  const results = files.map(checkFile).filter((result) => result.errors.length || result.warnings.length);
  for (const result of results) {
    for (const error of result.errors) {
      console.error(`ERROR ${result.file}: ${error}`);
    }
    for (const warning of result.warnings) {
      console.warn(`WARN ${result.file}: ${warning}`);
    }
  }

  const errorCount = results.reduce((count, result) => count + result.errors.length, 0);
  const warningCount = results.reduce((count, result) => count + result.warnings.length, 0);
  console.log(`convex-test capability check: ${files.length} files, ${errorCount} errors, ${warningCount} warnings`);
  process.exit(errorCount || (options.strictWarnings && warningCount) ? 1 : 0);
}

main();
