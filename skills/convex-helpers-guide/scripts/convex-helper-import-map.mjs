#!/usr/bin/env node

import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import process from 'node:process';

function printHelp() {
  console.log(`Usage: convex-helper-import-map.mjs [options]

Render a convex-helpers export/import map from package.json exports.

Options:
  --source <path>       convex-helpers package root. Default: cwd.
  --format <json|md>    Output format. Default: md.
  --output <path>       Write output to a file.
  --help                Show this help.
`);
}

function parseArgs(argv) {
  const options = {
    format: 'md',
    outputPath: undefined,
    source: process.cwd(),
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else if (arg === '--source') {
      options.source = resolve(requireValue(argv, (index += 1), arg));
    } else if (arg === '--format') {
      options.format = requireValue(argv, (index += 1), arg);
      if (!['json', 'md'].includes(options.format)) {
        throw new Error('--format must be json or md.');
      }
    } else if (arg === '--output') {
      options.outputPath = requireValue(argv, (index += 1), arg);
    } else {
      throw new Error(`Unknown option: ${arg}`);
    }
  }

  return options;
}

function requireValue(argv, index, flag) {
  const value = argv[index];
  if (!value || value.startsWith('--')) {
    throw new Error(`${flag} requires a value.`);
  }
  return value;
}

function readPackageJson(source) {
  const packagePath = join(source, 'package.json');
  if (!existsSync(packagePath)) {
    throw new Error(`package.json not found at ${packagePath}`);
  }
  return JSON.parse(readFileSync(packagePath, 'utf8'));
}

function pickTarget(value) {
  if (typeof value === 'string') {
    return value;
  }
  if (!value || typeof value !== 'object') {
    return '';
  }
  for (const key of ['import', 'default', 'require', 'types']) {
    const target = pickTarget(value[key]);
    if (target) {
      return target;
    }
  }
  return '';
}

function importPath(exportPath) {
  return exportPath === '.'
    ? 'convex-helpers'
    : `convex-helpers/${exportPath.replace(/^\.\//, '')}`;
}

function sourcePath(target) {
  return target.replace(/^\.\//, '');
}

function rowsFromExports(packageJson) {
  const exportsField = packageJson.exports;
  if (!exportsField) {
    return [];
  }
  const entries =
    typeof exportsField === 'string'
      ? [['.', exportsField]]
      : Object.entries(exportsField).sort(([a], [b]) => a.localeCompare(b));

  return entries.map(([exportPath, value]) => {
    const target = pickTarget(value);
    return {
      export: exportPath,
      import: importPath(exportPath),
      source: sourcePath(target),
    };
  });
}

function escapeCell(value) {
  return String(value ?? '').replaceAll('|', '\\|').replaceAll('\n', ' ');
}

function renderMarkdown(rows) {
  return `${[
    '| Export | Import | Source target |',
    '| --- | --- | --- |',
    ...rows.map((row) =>
      `| ${escapeCell(row.export)} | ${escapeCell(row.import)} | ${escapeCell(row.source)} |`,
    ),
  ].join('\n')}\n`;
}

function writeOutput(output, outputPath) {
  if (outputPath) {
    writeFileSync(resolve(outputPath), output);
    return;
  }
  process.stdout.write(output);
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

  const rows = rowsFromExports(readPackageJson(options.source));
  writeOutput(
    options.format === 'json'
      ? `${JSON.stringify(rows, null, 2)}\n`
      : renderMarkdown(rows),
    options.outputPath,
  );
}

main();
