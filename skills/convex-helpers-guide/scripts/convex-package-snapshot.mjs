#!/usr/bin/env node

import { existsSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import process from 'node:process';

const DEFAULT_PACKAGES = ['convex', 'convex-test', 'convex-helpers'];

const PACKAGE_SOURCES = {
  convex: {
    repo: 'convex-backend',
    subdir: 'npm-packages/convex',
  },
  'convex-helpers': {
    repo: 'convex-helpers',
    subdir: 'packages/convex-helpers',
  },
  'convex-test': {
    repo: 'convex-test',
    subdir: '',
  },
};

function printHelp() {
  console.log(`Usage: convex-package-snapshot.mjs [options]

Snapshot installed Convex package metadata and local opensrc source paths.

Options:
  --package <names>       Comma-separated packages. Default: ${DEFAULT_PACKAGES.join(',')}.
  --repo-root <path>      Repository root. Default: nearest package.json above cwd.
  --opensrc-root <path>   get-convex opensrc root. Default: ~/.opensrc/repos/github.com/get-convex.
  --format <json|md>      Output format. Default: md.
  --output <path>         Write output to a file.
  --help                  Show this help.
`);
}

function parseArgs(argv) {
  const options = {
    format: 'md',
    opensrcRoot: join(homedir(), '.opensrc/repos/github.com/get-convex'),
    outputPath: undefined,
    packages: DEFAULT_PACKAGES,
    repoRoot: findRepoRoot(process.cwd()),
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else if (arg === '--package') {
      options.packages = requireValue(argv, (index += 1), arg)
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean);
    } else if (arg === '--repo-root') {
      options.repoRoot = resolve(requireValue(argv, (index += 1), arg));
    } else if (arg === '--opensrc-root') {
      options.opensrcRoot = resolve(requireValue(argv, (index += 1), arg));
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

function findRepoRoot(start) {
  let current = resolve(start);
  while (true) {
    if (existsSync(join(current, 'package.json'))) {
      return current;
    }
    const parent = dirname(current);
    if (parent === current) {
      return resolve(start);
    }
    current = parent;
  }
}

function readJson(filePath) {
  if (!existsSync(filePath)) {
    return undefined;
  }
  return JSON.parse(readFileSync(filePath, 'utf8'));
}

function packageFiles(repoRoot) {
  const files = [join(repoRoot, 'package.json')];
  const packagesDir = join(repoRoot, 'packages');
  if (existsSync(packagesDir)) {
    for (const name of readdirSync(packagesDir)) {
      const filePath = join(packagesDir, name, 'package.json');
      if (existsSync(filePath)) {
        files.push(filePath);
      }
    }
  }
  return files;
}

function dependencySpec(packageName, repoRoot) {
  for (const filePath of packageFiles(repoRoot)) {
    const packageJson = readJson(filePath);
    for (const field of ['dependencies', 'devDependencies', 'peerDependencies']) {
      const spec = packageJson?.[field]?.[packageName];
      if (spec) {
        return { file: filePath, field, spec };
      }
    }
  }
  return undefined;
}

function installedPackageDir(packageName, repoRoot) {
  const dir = join(repoRoot, 'node_modules', ...packageName.split('/'));
  return existsSync(join(dir, 'package.json')) ? dir : undefined;
}

function cleanVersion(spec, installedVersion) {
  if (installedVersion) {
    return installedVersion;
  }
  return spec?.match(/\d+\.\d+\.\d+(?:[-+][\w.-]+)?/)?.[0];
}

function versionDirs(repoDir) {
  if (!existsSync(repoDir)) {
    return [];
  }
  return readdirSync(repoDir)
    .filter((name) => {
      const path = join(repoDir, name);
      return statSync(path).isDirectory() && /^\d+\.\d+\.\d+/.test(name);
    })
    .sort((a, b) => a.localeCompare(b, undefined, { numeric: true }));
}

function sourcePath(packageName, version, opensrcRoot) {
  const source = PACKAGE_SOURCES[packageName];
  if (!source) {
    return undefined;
  }
  const repoDir = join(opensrcRoot, source.repo);
  const versions = version ? [version, ...versionDirs(repoDir).reverse()] : versionDirs(repoDir).reverse();
  for (const candidateVersion of versions) {
    const candidate = join(repoDir, candidateVersion, source.subdir);
    if (existsSync(join(candidate, 'package.json'))) {
      return candidate;
    }
  }
  return undefined;
}

function exportKeys(packageJson) {
  const exportsField = packageJson?.exports;
  if (!exportsField) {
    return [];
  }
  return typeof exportsField === 'string'
    ? ['.']
    : Object.keys(exportsField).sort();
}

function readReadmeHeadings(dir) {
  const readme = join(dir, 'README.md');
  if (!existsSync(readme)) {
    return [];
  }
  return readFileSync(readme, 'utf8')
    .split('\n')
    .map((line) => line.match(/^#{1,3}\s+(.+?)\s*$/)?.[1])
    .filter(Boolean)
    .slice(0, 25);
}

function snapshotPackage(packageName, options) {
  const dependency = dependencySpec(packageName, options.repoRoot);
  const installedDir = installedPackageDir(packageName, options.repoRoot);
  const installedPackageJson = installedDir ? readJson(join(installedDir, 'package.json')) : undefined;
  const version = cleanVersion(dependency?.spec, installedPackageJson?.version);
  const sourceDir = sourcePath(packageName, version, options.opensrcRoot);
  const sourcePackageJson = sourceDir ? readJson(join(sourceDir, 'package.json')) : undefined;
  const effectivePackageJson = sourcePackageJson ?? installedPackageJson;

  return {
    package: packageName,
    dependency,
    installedPath: installedDir,
    installedVersion: installedPackageJson?.version,
    sourcePath: sourceDir,
    sourceVersion: sourcePackageJson?.version,
    exports: exportKeys(effectivePackageJson),
    readmeHeadings: sourceDir ? readReadmeHeadings(sourceDir) : [],
  };
}

function escapeCell(value) {
  return String(value ?? '').replaceAll('|', '\\|').replaceAll('\n', ' ');
}

function renderMarkdown(snapshots) {
  const rows = [
    '| Package | Spec | Installed | Source | Exports | README headings |',
    '| --- | --- | --- | --- | ---: | ---: |',
  ];
  for (const snapshot of snapshots) {
    rows.push(
      [
        snapshot.package,
        snapshot.dependency?.spec ?? '',
        snapshot.installedVersion ?? '',
        snapshot.sourcePath ?? '',
        snapshot.exports.length,
        snapshot.readmeHeadings.length,
      ]
        .map(escapeCell)
        .join(' | ')
        .replace(/^/, '| ')
        .replace(/$/, ' |'),
    );
  }

  for (const snapshot of snapshots) {
    rows.push('', `## ${snapshot.package}`, '');
    rows.push(`- dependency: ${snapshot.dependency?.spec ?? 'not found'}`);
    rows.push(`- installed: ${snapshot.installedVersion ?? 'not found'}`);
    rows.push(`- source: ${snapshot.sourcePath ?? 'not found'}`);
    if (snapshot.exports.length) {
      rows.push(`- exports: ${snapshot.exports.join(', ')}`);
    }
    if (snapshot.readmeHeadings.length) {
      rows.push(`- README headings: ${snapshot.readmeHeadings.join('; ')}`);
    }
  }

  return `${rows.join('\n')}\n`;
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

  const snapshots = options.packages.map((packageName) => snapshotPackage(packageName, options));
  writeOutput(
    options.format === 'json'
      ? `${JSON.stringify(snapshots, null, 2)}\n`
      : renderMarkdown(snapshots),
    options.outputPath,
  );
}

main();
