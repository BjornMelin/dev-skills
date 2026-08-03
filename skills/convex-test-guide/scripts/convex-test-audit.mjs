#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { relative, resolve } from 'node:path';
import process from 'node:process';

const SUPPORTED_EXTENSIONS = new Set([
  '.cjs',
  '.cts',
  '.js',
  '.jsx',
  '.mjs',
  '.mts',
  '.ts',
  '.tsx',
]);

const EXCLUDED_SEGMENTS = new Set([
  '.git',
  '.next',
  '.turbo',
  'build',
  'coverage',
  'dist',
  'node_modules',
  'output',
]);

const GENERATED_MARKERS = ['/_generated/', '/generated/'];

const RULES = {
  forbiddenPath: {
    confidence: 'validated',
    recommendation:
      'Move Convex integration tests to packages/backend/test/convex/**/*.test.ts.',
    severity: 'high',
    title: 'Forbidden Convex test location',
  },
  forbiddenSuffix: {
    confidence: 'validated',
    recommendation:
      'Rename to *.test.ts under packages/backend/test/convex/**.',
    severity: 'medium',
    title: 'Forbidden .convex-test.ts suffix',
  },
  envPragma: {
    confidence: 'validated',
    recommendation:
      'Remove file-level environment pragmas and let the backend-convex Vitest project own the runtime.',
    severity: 'high',
    title: 'File-level Vitest environment pragma',
  },
  missingHarnessShape: {
    confidence: 'likely',
    recommendation:
      'Use convexTest(schema, convexModules) with Signr test utilities.',
    severity: 'high',
    title: 'Non-canonical convex-test harness',
  },
  schedulerTimers: {
    confidence: 'likely',
    recommendation:
      'Use vi.useFakeTimers(), finishInProgressScheduledFunctions or finishAllScheduledFunctions, and restore timers.',
    severity: 'high',
    title: 'Scheduled-function test timer candidate',
  },
  sleep: {
    confidence: 'likely',
    recommendation:
      'Replace wall-clock sleeps with fake timers, scheduler drains, or direct state assertions.',
    severity: 'medium',
    title: 'Wall-clock sleep candidate',
  },
  liveNetwork: {
    confidence: 'candidate',
    recommendation:
      'Mock fetch or test Convex HTTP actions through t.fetch instead of live network calls.',
    severity: 'medium',
    title: 'Live network candidate',
  },
  exactError: {
    confidence: 'candidate',
    recommendation:
      'Assert product-visible behavior or stable ConvexError.data instead of exact backend error text.',
    severity: 'low',
    title: 'Exact error-string assertion candidate',
  },
  coverage: {
    confidence: 'candidate',
    recommendation:
      'Add or extend a convex-test suite using convexTest(schema, convexModules), t.run, t.withIdentity, and public/internal function calls.',
    severity: 'medium',
    title: 'Convex function coverage candidate',
  },
  seedDuplication: {
    confidence: 'candidate',
    recommendation:
      'Move repeated setup into packages/backend/test_utils/convex/** only when it is reused by multiple suites.',
    severity: 'low',
    title: 'Repeated seed/setup candidate',
  },
};

function printHelp() {
  console.log(`Usage: convex-test-audit.mjs </help|/audit|/review> [mode] [options]

Commands:
  /help
      Print supported commands and descriptions.
  /audit [backend|tests|full|refresh]
      Report convex-test coverage and harness-quality candidates.
  /review [diff|pr|full] [base]
      Review changed files, PR-style branch changes, or the full Convex test domain.

Modes:
  /audit
      Domain-bounded scan of backend Convex source, Convex tests, and test utils.
  /audit backend
      Scan packages/backend/convex for functions needing convex-test coverage.
  /audit tests
      Scan packages/backend/test/convex and test_utils/convex.
  /audit full
      Scan backend Convex source, tests, and test_utils.
  /audit refresh
      Same scope as /audit; use with package snapshot before final reliance.
  /review diff
      Review unstaged and staged local diffs.
  /review pr [base]
      Review branch plus local changes against base. Default: origin/main or main.
  /review full
      Review all default Convex test-domain files.

Options:
  --repo-root <path>  Repository root. Default: cwd.
  --format <md|json>  Output format. Default: md.
  --paths <csv>       Comma-separated explicit paths.
  --help              Show this help.
`);
}

function parseArgs(argv) {
  const options = {
    command: 'audit',
    format: 'md',
    mode: undefined,
    paths: undefined,
    repoRoot: process.cwd(),
  };

  const rest = [...argv];
  if (rest[0] && !rest[0].startsWith('--')) {
    options.command = rest.shift().replace(/^\//, '');
  }
  if (options.command === 'help') {
    options.help = true;
  }
  if (rest[0] && !rest[0].startsWith('--')) {
    options.mode = rest.shift();
  }
  if (options.command === 'review' && rest[0] && !rest[0].startsWith('--')) {
    options.base = rest.shift();
  }

  for (let index = 0; index < rest.length; index += 1) {
    const arg = rest[index];
    if (arg === '--help' || arg === '-h') {
      options.help = true;
    } else if (arg === '--repo-root') {
      options.repoRoot = resolve(requireValue(rest, (index += 1), arg));
    } else if (arg === '--format') {
      options.format = requireValue(rest, (index += 1), arg);
      if (!['json', 'md'].includes(options.format)) {
        throw new Error('--format must be md or json.');
      }
    } else if (arg === '--paths') {
      options.paths = requireValue(rest, (index += 1), arg)
        .split(',')
        .map((value) => value.trim())
        .filter(Boolean);
    } else {
      throw new Error(`Unknown option: ${arg}`);
    }
  }

  validateCommand(options);
  return options;
}

function validateCommand(options) {
  if (options.help) {
    return;
  }
  if (!['audit', 'review'].includes(options.command)) {
    throw new Error('Command must be /help, /audit, or /review.');
  }
  const allowedModes =
    options.command === 'audit'
      ? new Set([undefined, 'backend', 'tests', 'full', 'refresh'])
      : new Set([undefined, 'diff', 'pr', 'full']);
  if (!allowedModes.has(options.mode)) {
    throw new Error(
      `${options.command} mode must be one of: ${[...allowedModes]
        .filter(Boolean)
        .join(', ')}`,
    );
  }
}

function requireValue(argv, index, flag) {
  const value = argv[index];
  if (!value || value.startsWith('--')) {
    throw new Error(`${flag} requires a value.`);
  }
  return value;
}

function extension(path) {
  const match = path.match(/(\.[^.\/]+)$/);
  return match?.[1] ?? '';
}

function toPosix(path) {
  return path.replaceAll('\\', '/');
}

function repoRelative(repoRoot, path) {
  return toPosix(relative(repoRoot, path));
}

function commandExists(command) {
  const result = spawnSync('sh', ['-lc', `command -v ${quoteShell(command)}`], {
    encoding: 'utf8',
    timeout: 5_000,
  });
  return result.status === 0;
}

function quoteShell(value) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function runGit(repoRoot, args) {
  const result = spawnSync('git', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    timeout: 10_000,
  });
  return result.status === 0
    ? result.stdout
        .split('\n')
        .map((line) => line.trim())
        .filter(Boolean)
    : [];
}

function gitRefExists(repoRoot, ref) {
  return spawnSync('git', ['rev-parse', '--verify', '--quiet', ref], {
    cwd: repoRoot,
    stdio: 'ignore',
    timeout: 10_000,
  }).status === 0;
}

function defaultBase(repoRoot) {
  return gitRefExists(repoRoot, 'origin/main') ? 'origin/main' : 'main';
}

function reviewPaths(options) {
  const mode = options.mode ?? 'diff';
  if (mode === 'full') {
    return undefined;
  }
  if (mode === 'pr') {
    const base = options.base ?? defaultBase(options.repoRoot);
    return unique([
      ...runGit(options.repoRoot, ['diff', '--name-only', '--diff-filter=ACMR', `${base}...HEAD`]),
      ...runGit(options.repoRoot, ['diff', '--name-only', '--diff-filter=ACMR']),
      ...runGit(options.repoRoot, ['diff', '--cached', '--name-only', '--diff-filter=ACMR']),
    ]);
  }
  return unique([
    ...runGit(options.repoRoot, ['diff', '--name-only', '--diff-filter=ACMR']),
    ...runGit(options.repoRoot, ['diff', '--cached', '--name-only', '--diff-filter=ACMR']),
  ]);
}

function unique(values) {
  return [...new Set(values)];
}

function rootPaths(options) {
  if (options.paths) {
    return options.paths;
  }
  if (options.command === 'review') {
    const paths = reviewPaths(options);
    if (paths && (paths.length > 0 || options.mode)) {
      return paths;
    }
  }

  const mode = options.mode === 'refresh' ? 'domain' : (options.mode ?? 'domain');
  if (mode === 'backend') {
    return ['packages/backend/convex'];
  }
  if (mode === 'tests') {
    return ['packages/backend/test/convex', 'packages/backend/test_utils/convex'];
  }
  if (mode === 'full') {
    return ['packages/backend/convex', 'packages/backend/test', 'packages/backend/test_utils'];
  }
  return ['packages/backend/convex', 'packages/backend/test/convex', 'packages/backend/test_utils/convex'];
}

function inventoryFiles(repoRoot, inputPaths) {
  const inventory = {
    binary: 0,
    excluded: 0,
    generated: 0,
    missing: 0,
    scanned: 0,
    unsupported: 0,
  };
  const files = [];

  for (const inputPath of inputPaths) {
    const absolutePath = resolve(repoRoot, inputPath);
    collectFiles(repoRoot, absolutePath, inventory, files);
  }

  return { files: unique(files), inventory };
}

function collectFiles(repoRoot, absolutePath, inventory, files) {
  if (!existsSync(absolutePath)) {
    inventory.missing += 1;
    return;
  }
  const relativePath = repoRelative(repoRoot, absolutePath);
  if (isExcluded(relativePath)) {
    inventory.excluded += 1;
    return;
  }
  const dirent = readdirSafe(absolutePath);
  if (dirent) {
    for (const child of dirent) {
      collectFiles(repoRoot, resolve(absolutePath, child), inventory, files);
    }
    return;
  }
  if (isGenerated(relativePath)) {
    inventory.generated += 1;
    return;
  }
  if (!SUPPORTED_EXTENSIONS.has(extension(relativePath))) {
    inventory.unsupported += 1;
    return;
  }
  files.push(absolutePath);
  inventory.scanned += 1;
}

function readdirSafe(path) {
  try {
    return readdirSync(path);
  } catch {
    return undefined;
  }
}

function isExcluded(relativePath) {
  return relativePath.split('/').some((segment) => EXCLUDED_SEGMENTS.has(segment));
}

function isGenerated(relativePath) {
  return GENERATED_MARKERS.some((marker) => relativePath.includes(marker));
}

function addFinding(findings, ruleId, file, line, lineText, detail) {
  const rule = RULES[ruleId];
  findings.push({
    confidence: rule.confidence,
    detail,
    file,
    line,
    recommendation: rule.recommendation,
    severity: rule.severity,
    title: rule.title,
    ruleId,
    evidence: lineText.trim().slice(0, 220),
  });
}

function scanFile(repoRoot, filePath, findings, corpus) {
  const file = repoRelative(repoRoot, filePath);
  const text = readFileSync(filePath, 'utf8');
  const lines = text.split('\n');
  const isTest = file.includes('/test/') || file.endsWith('.test.ts') || file.endsWith('.convex-test.ts');

  if (file.includes('packages/backend/convex/') && file.endsWith('.test.ts')) {
    addFinding(findings, 'forbiddenPath', file, 1, file, 'Test file is inside Convex-loaded source tree.');
  }
  if (file.endsWith('.convex-test.ts')) {
    addFinding(findings, 'forbiddenSuffix', file, 1, file, 'Forbidden suffix.');
  }

  let insertCount = 0;
  let tRunCount = 0;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const lineNo = index + 1;

    if (/@vitest-environment\b/.test(line)) {
      addFinding(findings, 'envPragma', file, lineNo, line, 'File-level Vitest runtime override.');
    }
    if (/\bconvexTest\s*\(/.test(line) && (!/convexModules/.test(text) || !/convex\/schema/.test(text))) {
      addFinding(findings, 'missingHarnessShape', file, lineNo, line, 'Missing schema or convexModules reference.');
    }
    if (
      isTest &&
      /finish(?:All|InProgress)ScheduledFunctions/.test(line) &&
      !/vi\.useFakeTimers\s*\(/.test(text)
    ) {
      addFinding(findings, 'schedulerTimers', file, lineNo, line, 'Scheduler drain without fake timers in file.');
    }
    if (isTest && /new Promise\s*\([^)]*setTimeout|setTimeout\s*\(/.test(line)) {
      addFinding(findings, 'sleep', file, lineNo, line, 'Timer or sleep-like construct.');
    }
    if (isTest && /\bfetch\s*\(/.test(line) && !/\bt\.fetch\s*\(/.test(line)) {
      addFinding(findings, 'liveNetwork', file, lineNo, line, 'Global fetch in test file.');
    }
    if (isTest && /\.toThrow(?:Error)?\s*\(\s*['"`]/.test(line)) {
      addFinding(findings, 'exactError', file, lineNo, line, 'Exact thrown-message assertion.');
    }
    insertCount += count(line, /\.db\.insert\s*\(/g);
    tRunCount += count(line, /\bt\.run\s*\(/g);
  }

  if (isTest && insertCount >= 8 && tRunCount >= 3) {
    addFinding(
      findings,
      'seedDuplication',
      file,
      firstLine(lines, /\.db\.insert\s*\(/),
      'repeated inserts in t.run setup',
      `${insertCount} inserts and ${tRunCount} t.run calls in one test file.`,
    );
  }

  if (!isTest && file.startsWith('packages/backend/convex/')) {
    scanFunctionCoverage(file, lines, findings, corpus.testText);
  }
}

function scanFunctionCoverage(file, lines, findings, testText) {
  const exportRegex = /export\s+const\s+([A-Za-z0-9_]+)\s*=\s*(internalQuery|internalMutation|internalAction|query|mutation|action)\s*\(/;
  for (let index = 0; index < lines.length; index += 1) {
    const match = lines[index].match(exportRegex);
    if (!match) {
      continue;
    }
    const [, name, kind] = match;
    if (!testText.includes(name)) {
      addFinding(
        findings,
        'coverage',
        file,
        index + 1,
        lines[index],
        `${kind} ${name} is not referenced by name in Convex test files.`,
      );
    }
  }
}

function count(value, regex) {
  return [...value.matchAll(regex)].length;
}

function firstLine(lines, regex) {
  const index = lines.findIndex((line) => regex.test(line));
  return index === -1 ? 1 : index + 1;
}

function capabilities() {
  return Object.fromEntries(
    ['rg', 'ast-grep', 'codeql', 'semgrep', 'scip', 'zoekt', 'jq', 'fzf', 'git', 'bun']
      .map((tool) => [tool, commandExists(tool)]),
  );
}

function severityOrder(severity) {
  return ['critical', 'high', 'medium', 'low', 'nit', 'info'].indexOf(severity);
}

function renderMarkdown(report) {
  const lines = [
    `# convex-test ${report.command} report`,
    '',
    `- mode: ${report.mode}`,
    `- generated: ${report.generatedAt}`,
    `- scanned files: ${report.inventory.scanned}`,
    `- excluded/generated/unsupported/missing: ${report.inventory.excluded}/${report.inventory.generated}/${report.inventory.unsupported}/${report.inventory.missing}`,
    `- tools: ${Object.entries(report.capabilities)
      .filter(([, ok]) => ok)
      .map(([tool]) => tool)
      .join(', ') || 'none'}`,
    '',
    '## Candidate findings',
    '',
  ];

  const findings = [...report.findings].sort(
    (a, b) => severityOrder(a.severity) - severityOrder(b.severity),
  );

  if (!findings.length) {
    lines.push('No candidates found in scanned files.', '');
  }
  for (const finding of findings) {
    lines.push(
      `### ${finding.severity.toUpperCase()} ${finding.ruleId}: ${finding.title}`,
      '',
      `- confidence: ${finding.confidence}`,
      `- location: ${finding.file}:${finding.line}`,
      `- evidence: \`${finding.evidence.replaceAll('`', "'")}\``,
      `- detail: ${finding.detail}`,
      `- recommendation: ${finding.recommendation}`,
      `- continue-safe: validate first; implement only if current code confirms the candidate.`,
      '',
    );
  }

  lines.push(
    '## Continue contract',
    '',
    'If the user replies `continue`, implement all safe validated findings from the report, stop for risky or ambiguous items, and return completed/remaining/next-batch sections.',
    '',
  );
  return `${lines.join('\n')}\n`;
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

  const inputPaths = rootPaths(options);
  const { files, inventory } = inventoryFiles(options.repoRoot, inputPaths);
  const testCorpusFiles = inventoryFiles(options.repoRoot, [
    'packages/backend/test/convex',
    'packages/backend/test_utils/convex',
  ]).files;
  const corpus = {
    testText: testCorpusFiles.map((file) => readFileSync(file, 'utf8')).join('\n'),
  };
  const findings = [];
  for (const file of files) {
    scanFile(options.repoRoot, file, findings, corpus);
  }

  const report = {
    capabilities: capabilities(),
    command: options.command,
    generatedAt: new Date().toISOString(),
    inputPaths,
    inventory,
    mode: options.mode ?? 'domain',
    findings,
  };

  process.stdout.write(
    options.format === 'json'
      ? `${JSON.stringify(report, null, 2)}\n`
      : renderMarkdown(report),
  );
}

main();
