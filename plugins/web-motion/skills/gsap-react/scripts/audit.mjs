#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "gsap-react",
  "rules": [
    {
      "id": "gsap.private-registry",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      // gsap-react-audit-ignore gsap.private-registry
      "pattern": "npm\\.greensock\\.com|GREEN.?SOCK.*TOKEN|greensock.*authToken|gsap.*license.?key",
      "message": "Outdated private GSAP registry/token guidance or config is present.",
      "recommendation": "Use the public gsap npm package; modern GSAP plugins do not require a private registry token."
    },
    {
      "id": "gsap.react-confusable-package",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageHasAny",
      "packages": ["gsap-react"],
      "message": "The unofficial/confusable gsap-react package is installed.",
      "recommendation": "Use the official @gsap/react package with the public gsap package."
    },
    {
      "id": "gsap.react-usegsap-missing-package",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageMissingWhenFileMatches",
      "include": "from\\s+['\"]@gsap/react['\"]",
      "packages": ["@gsap/react"],
      "message": "Source imports @gsap/react but package.json does not declare it.",
      "recommendation": "Add @gsap/react with the target repo's package manager, or remove the import if it is stale."
    },
    {
      "id": "gsap.react-missing-gsap-peer",
      "severity": "medium",
      "confidence": "medium",
      "category": "dependency",
      "kind": "packageMissingWhenPackagePresent",
      "whenPackage": "@gsap/react",
      "requiredPackage": "gsap",
      "message": "@gsap/react is declared but its gsap peer is not declared in package.json.",
      "recommendation": "Declare gsap alongside @gsap/react so the React hook resolves the intended GSAP runtime."
    },
    {
      "id": "gsap.react-effect-no-cleanup",
      "severity": "high",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "useEffect\\s*\\(|useLayoutEffect\\s*\\(",
      "also": "gsap\\.|ScrollTrigger",
      "without": "useGSAP|gsap\\.context|ctx\\.revert|\\.revert\\s*\\(|\\.kill\\s*\\(",
      "message": "GSAP appears inside a React effect without an obvious cleanup path.",
      "recommendation": "Prefer useGSAP from @gsap/react, or wrap the setup in gsap.context() and return ctx.revert()."
    },
    {
      "id": "gsap.react-usegsap-unregistered",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "fileContainsBothWithout",
      "include": "from\\s+['\"]@gsap/react['\"]",
      "also": "\\buseGSAP\\s*\\(",
      "without": "gsap\\.registerPlugin\\s*\\(\\s*useGSAP\\s*\\)",
      "message": "useGSAP is imported but not obviously registered as a GSAP plugin.",
      "recommendation": "Call gsap.registerPlugin(useGSAP) once in the module or shared client setup before useGSAP runs."
    },
    {
      "id": "gsap.react-next-use-client",
      "severity": "medium",
      "confidence": "medium",
      "category": "ssr",
      "kind": "fileContainsBothWithout",
      "pathPattern": "(^|[/\\\\])app[/\\\\].*\\.(jsx|tsx)$",
      "include": "from\\s+['\"]@gsap/react['\"]",
      "also": "\\buseGSAP\\s*\\(",
      "without": "['\"]use client['\"]",
      "message": "A Next.js App Router file uses useGSAP without an obvious client-component directive.",
      "recommendation": "Add 'use client' at the top of the component file or move the GSAP code into a client component."
    },
    {
      "id": "gsap.react-unscoped-selector",
      "severity": "medium",
      "confidence": "low",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "\\buseGSAP\\s*\\(",
      "also": "gsap\\.(to|from|fromTo|set|timeline)\\s*\\(\\s*['\"](?:[.#\\[]|[A-Za-z][A-Za-z0-9_-]*(?=['\"\\s>+~.#:\\[]))",
      "without": "scope\\s*:",
      "message": "useGSAP appears to animate selector text without an obvious scope.",
      "recommendation": "Pass a container ref as useGSAP({ scope }) or target specific refs so selectors cannot escape the component."
    },
    {
      "id": "gsap.react-context-unscoped-selector",
      "severity": "medium",
      "confidence": "low",
      "category": "lifecycle",
      "kind": "contextSelectorWithoutScope",
      "include": "gsap\\.context\\s*\\(",
      "also": "gsap\\.(to|from|fromTo|set|timeline)\\s*\\(\\s*['\"](?:[.#\\[]|[A-Za-z][A-Za-z0-9_-]*(?=['\"\\s>+~.#:\\[]))",
      "message": "gsap.context() appears to animate selector text without an obvious scope argument.",
      "recommendation": "Pass a container ref/element as the second gsap.context() argument or replace selector text with refs."
    },
    {
      "id": "gsap.react-delayed-without-contextsafe",
      "severity": "medium",
      "confidence": "low",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "\\b(onClick|onPointer|onMouse|setTimeout|setInterval|addEventListener)\\b",
      "also": "gsap\\.",
      "without": "contextSafe\\s*\\(",
      "message": "GSAP appears in an event/delayed callback without an obvious contextSafe wrapper.",
      "recommendation": "Wrap GSAP-created delayed or event handlers in contextSafe() and clean up native listeners or timers."
    },
    {
      "id": "gsap.react-missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\b(gsap\\.(to|from|fromTo|timeline|set)|ScrollTrigger\\.)\\b",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|AccessibilityInfo|reduceMotion|matchMedia\\s*\\(",
      "message": "GSAP motion code was found without an obvious reduced-motion branch in the same file.",
      "recommendation": "Add reduced-motion behavior in the component or route the effect through a shared reduced-motion layer."
    }
  ]
};

const skipDirs = new Set([
  '.git', 'node_modules', '.next', '.nuxt', 'dist', 'build', 'coverage',
  '.expo', '.turbo', '.vercel', '.cache', '.codex', '.agents',
  'output', 'tmp', 'temp', 'vendor', 'playwright-report', 'storybook-static',
]);
const fileExtensions = new Set([
  '.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.css', '.scss', '.sass',
  '.html', '.vue', '.svelte', '.json',
]);
const fileNames = new Set([
  '.npmrc', '.yarnrc', '.yarnrc.yml', '.pnpmfile.cjs',
]);
const severities = ['low', 'medium', 'high'];

function usage() {
  return `Usage:
  scripts/audit.mjs scan [--root <path>] [--format markdown|json] [--output <path>] [--max-files <n>]
  scripts/audit.mjs doctor [--root <path>] [--format markdown|json]

Options:
  --root <path>       Target repo root. Defaults to current working directory.
  --format <format>   markdown or json. Defaults to markdown.
  --json              Alias for --format json.
  --output <path>     Optional caller-chosen file path for report output.
  --max-files <n>     Max files to scan. Defaults to 2000.
  --help              Show this help.

Examples:
  scripts/audit.mjs --json doctor --root .
  scripts/audit.mjs scan --root . --format markdown
  scripts/audit.mjs scan --root . --format json --output gsap-react-audit.json

Config:
  Optional .gsap-react-audit.json at --root supports:
  {
    "ignoreRules": ["rule-id"],
    "ignorePaths": ["generated/", "fixtures/"],
    "ignores": [{"ruleId": "rule-id", "path": "src/example.tsx"}]
  }

Inline suppression:
  // gsap-react-audit-ignore rule-id
  // gsap-react-audit-ignore all

Compatibility:
  .motion-audit.json and // motion-audit-ignore comments are also honored for
  web-motion plugin consistency.
`;
}

function parseArgs(argv) {
  const args = { command: null, root: process.cwd(), format: 'markdown', output: null, maxFiles: 2000 };
  const rest = [...argv];
  while (rest.length) {
    const arg = rest.shift();
    if (arg === '--help' || arg === '-h') args.help = true;
    else if (arg === '--json') args.format = 'json';
    else if (arg === '--root') args.root = path.resolve(rest.shift() ?? '.');
    else if (arg === '--format') args.format = rest.shift() ?? 'markdown';
    else if (arg === '--output') args.output = path.resolve(rest.shift() ?? '');
    else if (arg === '--max-files') args.maxFiles = Number(rest.shift() ?? 2000);
    else if (!arg.startsWith('-') && args.command === null) args.command = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }
  args.command = args.command ?? 'scan';
  if (!['scan', 'doctor'].includes(args.command)) throw new Error(`Unknown command: ${args.command}`);
  if (!['markdown', 'json'].includes(args.format)) throw new Error(`Unknown format: ${args.format}`);
  if (!Number.isFinite(args.maxFiles) || args.maxFiles < 1) throw new Error('--max-files must be a positive number');
  return args;
}

function loadConfig(root) {
  const file = [
    path.join(root, '.gsap-react-audit.json'),
    path.join(root, '.motion-audit.json'),
  ].find((candidate) => fs.existsSync(candidate));
  if (!file) return { ignoreRules: [], ignorePaths: [], ignores: [] };
  try {
    const parsed = JSON.parse(fs.readFileSync(file, 'utf8'));
    return {
      ignoreRules: Array.isArray(parsed.ignoreRules) ? parsed.ignoreRules : [],
      ignorePaths: Array.isArray(parsed.ignorePaths) ? parsed.ignorePaths : [],
      ignores: Array.isArray(parsed.ignores) ? parsed.ignores : [],
    };
  } catch (error) {
    throw new Error(`Failed to parse .gsap-react-audit.json: ${error.message}`);
  }
}

function shouldSkipDir(relativePath) {
  return relativePath.split(path.sep).some((part) => skipDirs.has(part));
}

function readDirEntries(dir) {
  try {
    return fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }
}

function shouldSkipFile(relativePath, fileName, fullPath = '') {
  const normalized = relativePath.split(path.sep).join('/');
  const full = fullPath.split(path.sep).join('/');
  if (fullPath) {
    try {
      if (fs.readFileSync(fullPath, 'utf8').slice(0, 512).includes('motion-audit-skip-file')) return true;
    } catch {}
  }
  if (normalized === 'scripts/audit.mjs' || normalized.endsWith('/scripts/audit.mjs')) return true;
  if (/^skills\/[^/]+\/(?:agents|assets|examples|references|scripts|templates)(?:\/|$)/.test(normalized)) return true;
  if (path.basename(fileName) === 'SKILL.md' && /^skills\/[^/]+\/SKILL\.md$/.test(normalized)) return true;
  return path.extname(fileName) === '.json' && fileName !== 'package.json';
}

function listFiles(root, maxFiles) {
  const files = [];
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel)) walk(full);
      } else if (entry.isFile() && (fileExtensions.has(path.extname(entry.name)) || fileNames.has(entry.name))) {
        if (shouldSkipFile(rel, entry.name, full)) continue;
        files.push(full);
      }
      if (files.length >= maxFiles) return;
    }
  }
  walk(root);
  return files;
}

function listPackageFiles(root) {
  const files = [];
  function walk(dir) {
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel)) walk(full);
      } else if (entry.isFile() && entry.name === 'package.json') {
        files.push(full);
      }
    }
  }
  walk(root);
  return files;
}

function readPackage(root) {
  const file = path.join(root, 'package.json');
  if (!fs.existsSync(file)) return { exists: false, packages: new Set() };
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return { exists: true, packages: new Set(Object.keys(deps ?? {})), versions: deps ?? {}, scripts: pkg.scripts ?? {} };
  } catch {
    return { exists: true, packages: new Set(), versions: {}, scripts: {} };
  }
}

function readPackageFile(file) {
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return { packages: new Set(Object.keys(deps ?? {})), versions: deps ?? {} };
  } catch {
    return { packages: new Set(), versions: {} };
  }
}

function packageRecords(packageFiles, root) {
  return packageFiles
    .map((file) => ({
      file,
      relativePath: path.relative(root, file),
      dir: path.dirname(file),
      ...readPackageFile(file),
    }));
}

function nearestPackageRecord(file, records) {
  return records
    .filter((record) => {
      const relative = path.relative(record.dir, file);
      return relative === '' || (!relative.startsWith('..') && !path.isAbsolute(relative));
    })
    .sort((a, b) => b.dir.length - a.dir.length)[0] ?? null;
}

function lineForIndex(text, index) {
  return text.slice(0, index).split('\n').length;
}

function excerptForLine(lines, lineNumber) {
  return (lines[lineNumber - 1] ?? '').trim().slice(0, 240);
}

function isIgnored(config, ruleId, relativePath, lines, lineNumber) {
  if (config.ignoreRules.includes(ruleId)) return true;
  if (config.ignorePaths.some((ignored) => relativePath.includes(ignored))) return true;
  if (config.ignores.some((entry) => entry?.ruleId === ruleId && typeof entry.path === 'string' && entry.path.length > 0 && relativePath.includes(entry.path))) return true;
  const nearby = [lines[lineNumber - 1], lines[lineNumber - 2]].filter(Boolean).join('\n');
  return nearby.includes('gsap-react-audit-ignore all') ||
    nearby.includes(`gsap-react-audit-ignore ${ruleId}`) ||
    nearby.includes('motion-audit-ignore all') ||
    nearby.includes(`motion-audit-ignore ${ruleId}`);
}

function makeFinding(rule, relativePath, line, excerpt) {
  const safeExcerpt = rule.id === 'gsap.private-registry' ? '(redacted private registry/token config)' : excerpt;
  return {
    id: `${rule.id}:${relativePath}:${line}`,
    ruleId: rule.id,
    severity: rule.severity,
    confidence: rule.confidence,
    category: rule.category,
    file: relativePath,
    line,
    excerpt: safeExcerpt,
    rationale: rule.message,
    recommendation: rule.recommendation,
  };
}

function ruleRegex(pattern) {
  return new RegExp(pattern, 'gmi');
}

function analyzeCall(text, openParenIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  let topLevelCommas = 0;
  for (let index = openParenIndex; index < text.length; index += 1) {
    const char = text[index];
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === '\\') {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
      continue;
    }
    if (char === '(' || char === '[' || char === '{') {
      depth += 1;
      continue;
    }
    if (char === ')' || char === ']' || char === '}') {
      depth -= 1;
      if (depth === 0 && char === ')') {
        return { end: index, topLevelCommas };
      }
      continue;
    }
    if (char === ',' && depth === 1) {
      topLevelCommas += 1;
    }
  }
  return null;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  if (rule.pathPattern && !new RegExp(rule.pathPattern).test(relativePath)) {
    return [];
  }
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'contextSelectorWithoutScope') {
    if (!ruleRegex(rule.also).test(text)) return findings;
    const regex = ruleRegex(rule.include);
    for (const match of text.matchAll(regex)) {
      const openParen = text.indexOf('(', match.index ?? 0);
      const call = openParen === -1 ? null : analyzeCall(text, openParen);
      if (!call || call.topLevelCommas > 0) continue;
      const line = lineForIndex(text, match.index ?? 0);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'fileContainsWithout' || rule.kind === 'fileContainsBoth' || rule.kind === 'fileContainsBothWithout') {
    const includeMatch = ruleRegex(rule.include).exec(text);
    const alsoMatch = rule.also ? ruleRegex(rule.also).exec(text) : null;
    const withoutMatch = rule.without ? ruleRegex(rule.without).exec(text) : null;
    const matches =
      rule.kind === 'fileContainsBoth'
        ? includeMatch && alsoMatch
        : rule.kind === 'fileContainsBothWithout'
          ? includeMatch && alsoMatch && !withoutMatch
          : includeMatch && (!rule.also || alsoMatch) && !withoutMatch;
    if (matches) {
      const index = includeMatch.index;
      const line = lineForIndex(text, index);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'packageHasAny') return findings;
  const regex = ruleRegex(rule.pattern);
  for (const match of text.matchAll(regex)) {
    const line = lineForIndex(text, match.index ?? 0);
    if (!isIgnored(config, rule.id, relativePath, lines, line)) {
      findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
    }
  }
  return findings;
}

function scan(root, maxFiles) {
  if (!fs.existsSync(root)) throw new Error(`Root does not exist: ${root}`);
  const config = loadConfig(root);
  const files = listFiles(root, maxFiles);
  const packages = packageRecords(listPackageFiles(root), root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageHasAny') {
      for (const record of packages) {
        const matched = (rule.packages ?? []).filter((name) => record.packages.has(name));
        if (matched.length === 0) continue;
        if (isIgnored(config, rule.id, record.relativePath, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${record.relativePath}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: record.relativePath,
          line: 1,
          excerpt: `matched packages: ${matched.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageMissingWhenPackagePresent') {
      for (const record of packages) {
        if (!record.packages.has(rule.whenPackage) || record.packages.has(rule.requiredPackage)) continue;
        if (isIgnored(config, rule.id, record.relativePath, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${record.relativePath}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: record.relativePath,
          line: 1,
          excerpt: `${rule.whenPackage}: ${record.versions[rule.whenPackage] ?? '(declared)'}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageMissingWhenFileMatches') {
      for (const file of files) {
        let text;
        try {
          text = fs.readFileSync(file, 'utf8');
        } catch {
          continue;
        }
        const match = ruleRegex(rule.include).exec(text);
        if (!match) continue;
        const record = nearestPackageRecord(file, packages);
        const missing = (rule.packages ?? []).filter((name) => !record?.packages.has(name));
        if (missing.length === 0) continue;
        const relativePath = path.relative(root, file);
        const lines = text.split('\n');
        const line = lineForIndex(text, match.index);
        if (!isIgnored(config, rule.id, relativePath, lines, line)) {
          findings.push(makeFinding(rule, relativePath, line, `missing packages: ${missing.join(', ')}`));
        }
      }
      continue;
    }
    for (const file of files) {
      let text;
      try {
        text = fs.readFileSync(file, 'utf8');
      } catch {
        continue;
      }
      findings.push(...scanRule(rule, file, root, text, config));
    }
  }
  return {
    ok: !findings.some((finding) => finding.severity === 'high'),
    profile: profile.skillName,
    root,
    scannedFiles: files.length,
    rules: profile.rules.length,
    findings,
    summary: severities.reduce((acc, severity) => {
      acc[severity] = findings.filter((finding) => finding.severity === severity).length;
      return acc;
    }, {}),
  };
}

function doctor(root, maxFiles) {
  const pkg = readPackage(root);
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    configuredRules: profile.rules.length,
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.gsap-react-audit.json')) ||
      fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'use --format json for machine-readable output',
      'findings can be suppressed with .gsap-react-audit.json or inline gsap-react-audit-ignore comments',
      'legacy .motion-audit.json and motion-audit-ignore are supported for web-motion plugin consistency',
    ],
  };
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    return `# GSAP React Audit Doctor: ${result.profile}

- Root: ${result.root}
- Package JSON: ${result.packageJson ? 'yes' : 'no'}
- Config file: ${result.configFile ? 'yes' : 'no'}
- Configured rules: ${result.configuredRules}
- Sample file count: ${result.sampleFileCount}
- Status: ${result.ok ? 'ok' : 'failed'}
`;
  }
  const findings = result.findings
    .map((finding) => `## ${finding.severity.toUpperCase()} ${finding.ruleId}

- File: ${finding.file}:${finding.line}
- Confidence: ${finding.confidence}
- Category: ${finding.category}
- Evidence: ${finding.excerpt || '(file-level match)'}
- Rationale: ${finding.rationale}
- Recommendation: ${finding.recommendation}
`)
    .join('\n');
  return `# GSAP React Audit Report: ${result.profile}

- Root: ${result.root}
- Scanned files: ${result.scannedFiles}
- Rules: ${result.rules}
- Findings: ${result.findings.length}
- Severity summary: high=${result.summary.high}, medium=${result.summary.medium}, low=${result.summary.low}
- Status: ${result.ok ? 'no high-severity findings' : 'high-severity findings present'}

${findings || 'No findings.'}
`;
}

function emit(result, args) {
  const body = args.format === 'json' ? JSON.stringify(result, null, 2) + '\n' : renderMarkdown(result);
  if (args.output) {
    fs.mkdirSync(path.dirname(args.output), { recursive: true });
    fs.writeFileSync(args.output, body);
  } else {
    process.stdout.write(body);
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  if (args.help) {
    process.stdout.write(usage());
    process.exit(0);
  }
  const result = args.command === 'doctor' ? doctor(args.root, args.maxFiles) : scan(args.root, args.maxFiles);
  emit(result, args);
  process.exit(result.ok ? 0 : 2);
} catch (error) {
  const payload = { ok: false, profile: profile.skillName, error: error.message };
  const wantsJson = process.argv.includes('--json') || process.argv.includes('--format') && process.argv.includes('json');
  if (wantsJson) process.stdout.write(JSON.stringify(payload, null, 2) + '\n');
  else {
    console.error(error.message);
    console.error(usage());
  }
  process.exit(1);
}
