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
  collect: {
    confidence: 'likely',
    recommendation:
      'Prefer an index-bounded query, native paginate, or convex-helpers filter/pagination/stream only after bounding reads.',
    severity: 'high',
    title: 'Unbounded collect candidate',
  },
  relationships: {
    confidence: 'candidate',
    recommendation:
      'Check whether relationship helpers such as getOrThrow, getAll, getOneFrom, getManyFrom, or getManyVia remove repeated point reads.',
    severity: 'medium',
    title: 'Repeated relationship read candidate',
  },
  customFunctions: {
    confidence: 'candidate',
    recommendation:
      'Check whether customQuery/customMutation/customAction/customCtx can centralize repeated auth, org scope, and ctx enrichment.',
    severity: 'medium',
    title: 'Custom function wrapper candidate',
  },
  pagination: {
    confidence: 'candidate',
    recommendation:
      'Check whether paginator/getPage or stream/mergedStream better express this pagination shape.',
    severity: 'medium',
    title: 'Pagination helper candidate',
  },
  cors: {
    confidence: 'likely',
    recommendation:
      'Use corsRouter when the route is ordinary CORS handling and Signr auth/webhook constraints stay explicit.',
    severity: 'medium',
    title: 'Manual CORS candidate',
  },
  retries: {
    confidence: 'candidate',
    recommendation:
      'Check whether makeActionRetrier or withJitter replaces hand-rolled retry/backoff behavior.',
    severity: 'medium',
    title: 'Retry helper candidate',
  },
  rateLimit: {
    confidence: 'candidate',
    recommendation:
      'Check whether defineRateLimits, rateLimit, or checkRateLimit replaces local rate-window bookkeeping.',
    severity: 'medium',
    title: 'Rate-limit helper candidate',
  },
  validators: {
    confidence: 'candidate',
    recommendation:
      'Check whether typedV, doc, partial, literals, nullable, validate, parse, or Zod conversion helpers reduce validator duplication.',
    severity: 'low',
    title: 'Validator helper candidate',
  },
  triggers: {
    confidence: 'candidate',
    recommendation:
      'Check whether Triggers/writerWithTriggers should own invariant maintenance inside wrapped mutations.',
    severity: 'medium',
    title: 'Trigger helper candidate',
  },
  clientCache: {
    confidence: 'candidate',
    recommendation:
      'Check whether withArgs, sessions, or React cache helpers reduce repeated client query/session glue.',
    severity: 'low',
    title: 'Client helper candidate',
  },
  existing: {
    confidence: 'validated',
    recommendation:
      'Existing convex-helpers import; review whether usage follows Signr guardrails and is not hiding authz or indexing.',
    severity: 'info',
    title: 'Existing convex-helpers usage',
  },
};

function printHelp() {
  console.log(`Usage: convex-helpers-audit.mjs </help|/audit|/review> [mode] [options]

Commands:
  /help
      Print supported commands and descriptions.
  /audit [backend|tests|client|full|refresh]
      Report package-leverage candidates across the default Convex domain.
  /review [diff|pr|full] [base]
      Review changed files, PR-style branch changes, or the full Convex domain.

Modes:
  /audit
      Domain-bounded scan of backend Convex, Convex tests, and Convex clients.
  /audit backend
      Scan packages/backend/convex.
  /audit tests
      Scan packages/backend/test/convex and test_utils/convex.
  /audit client
      Scan apps/web and apps/mobile for Convex client-helper opportunities.
  /audit full
      Scan apps, packages, and scripts.
  /audit refresh
      Same scope as /audit; use with package snapshot before final reliance.
  /review diff
      Review unstaged and staged local diffs.
  /review pr [base]
      Review branch plus local changes against base. Default: origin/main or main.
  /review full
      Review all default Convex-domain files.

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
      ? new Set([undefined, 'backend', 'tests', 'client', 'full', 'refresh'])
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
  if (mode === 'client') {
    return ['apps/web', 'apps/mobile'];
  }
  if (mode === 'full') {
    return ['apps', 'packages', 'scripts'];
  }
  return [
    'packages/backend/convex',
    'packages/backend/test/convex',
    'packages/backend/test_utils/convex',
    'apps/web',
    'apps/mobile',
  ];
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

function scanFile(repoRoot, filePath, findings) {
  const file = repoRelative(repoRoot, filePath);
  const text = readFileSync(filePath, 'utf8');
  const lines = text.split('\n');
  let dbGetCount = 0;
  let queryBuilderCount = 0;
  let paginateCount = 0;
  let validatorCount = 0;
  let authCount = 0;
  let patchCount = 0;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const codeLine = !isCommentLine(line);
    const lineNo = index + 1;

    if (line.includes('convex-helpers/')) {
      addFinding(findings, 'existing', file, lineNo, line, 'Existing helper import.');
    }
    if (file.startsWith('packages/backend/convex/') && line.includes('.collect(')) {
      addFinding(findings, 'collect', file, lineNo, line, 'Audit read bounds and index coverage.');
    }
    if (/access-control-allow-|Access-Control-Allow-/i.test(line)) {
      addFinding(findings, 'cors', file, lineNo, line, 'Manual CORS header handling.');
    }
    if (codeLine && /\b(retry|backoff|jitter|maxAttempts|attempts)\b/i.test(line)) {
      addFinding(findings, 'retries', file, lineNo, line, 'Retry/backoff keyword.');
    }
    if (codeLine && /\b(rateLimit|rate limit|windowMs|throttle)\b/i.test(line)) {
      addFinding(findings, 'rateLimit', file, lineNo, line, 'Rate-limit keyword.');
    }
    if (/withArgs|useQuery|usePaginatedQuery|sessionId|anonymous/i.test(line) && file.startsWith('apps/')) {
      addFinding(findings, 'clientCache', file, lineNo, line, 'Client Convex hook/session glue.');
    }

    dbGetCount += count(line, /\.db\.get\s*\(/g);
    queryBuilderCount += count(line, /\.db\.query\s*\(/g);
    paginateCount += count(line, /\.paginate\s*\(/g);
    validatorCount += count(line, /\bv\.(object|union|literal|array|optional|id)\s*\(/g);
    authCount += count(line, /\b(auth|identity|orgId|tenant|requireUser|requireOrg|getAuthUser)\b/g);
    patchCount += count(line, /\.db\.patch\s*\(|\.db\.replace\s*\(|\.db\.insert\s*\(/g);
  }

  if (dbGetCount >= 8) {
    addFinding(
      findings,
      'relationships',
      file,
      firstLine(lines, /\.db\.get\s*\(/),
      'multiple ctx.db.get calls',
      `${dbGetCount} point reads in one file.`,
    );
  }
  if (queryBuilderCount >= 3 && authCount >= 3) {
    addFinding(
      findings,
      'customFunctions',
      file,
      firstLine(lines, /\.db\.query\s*\(|auth|orgId/),
      'repeated query/auth patterns',
      `${queryBuilderCount} query builders and ${authCount} auth/org references.`,
    );
  }
  if (paginateCount >= 2) {
    addFinding(
      findings,
      'pagination',
      file,
      firstLine(lines, /\.paginate\s*\(/),
      'multiple paginate calls',
      `${paginateCount} paginate calls in one file.`,
    );
  }
  if (validatorCount >= 25) {
    addFinding(
      findings,
      'validators',
      file,
      firstLine(lines, /\bv\.(object|union|literal|array|optional|id)\s*\(/),
      'dense validator definitions',
      `${validatorCount} Convex validator calls in one file.`,
    );
  }
  if (patchCount >= 8 && /summary|stats|invariant|aggregate/i.test(text)) {
    addFinding(
      findings,
      'triggers',
      file,
      firstLine(lines, /\.db\.patch\s*\(|summary|stats|invariant/),
      'mutation-side invariant maintenance',
      `${patchCount} writes plus invariant/summary language.`,
    );
  }
}

function count(value, regex) {
  return [...value.matchAll(regex)].length;
}

function isCommentLine(line) {
  return /^\s*(?:\/\/|\/\*|\*|\*\/)/.test(line);
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
    `# convex-helpers ${report.command} report`,
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
  const findings = [];
  for (const file of files) {
    scanFile(options.repoRoot, file, findings);
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
