#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "gsap-utils",
  "rules": [
    {
      "id": "gsap-utils.unitize-raw-value",
      "severity": "high",
      "confidence": "high",
      "category": "api-correctness",
      "pattern": "\\bgsap\\.utils\\.unitize\\s*\\(\\s*(?:[-+]?\\d|['\"`])",
      "message": "gsap.utils.unitize() appears to receive a raw value instead of a function.",
      "recommendation": "Pass a configured numeric function first, for example gsap.utils.unitize(gsap.utils.wrap(0, width), 'px')."
    },
    {
      "id": "gsap-utils.to-array-unscoped-selector",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "pattern": "\\bgsap\\.utils\\.toArray\\s*\\(\\s*['\"`](?:[.#\\[]|[A-Za-z][A-Za-z0-9_-]*(?=['\"`\\s>+~.#:\\[]))",
      "message": "gsap.utils.toArray() is called with selector text and no obvious scope in the same call.",
      "recommendation": "Scope selector text with the scope argument, gsap.utils.selector(root), or a local gsap.context(); verify this code is client-side."
    },
    {
      "id": "gsap-utils.selector-missing-scope",
      "severity": "medium",
      "confidence": "high",
      "category": "lifecycle",
      "pattern": "\\bgsap\\.utils\\.selector\\s*\\(\\s*\\)",
      "message": "gsap.utils.selector() is called without a scope.",
      "recommendation": "Pass a component root, DOM element, React ref, Angular ElementRef, or selector string that resolves to a local scope."
    },
    {
      "id": "gsap-utils.shuffle-mutates",
      "severity": "medium",
      "confidence": "high",
      "category": "state",
      "pattern": "\\bgsap\\.utils\\.shuffle\\s*\\(",
      "message": "gsap.utils.shuffle() mutates and returns the same array.",
      "recommendation": "Confirm mutation is intended; otherwise shuffle a copy such as gsap.utils.shuffle([...items])."
    },
    {
      "id": "gsap-utils.random-nondeterministic",
      "severity": "low",
      "confidence": "medium",
      "category": "determinism",
      "pattern": "\\bgsap\\.utils\\.random\\s*\\(",
      "message": "gsap.utils.random() is nondeterministic.",
      "recommendation": "Verify nondeterminism is acceptable for tests, SSR/hydration, replay, and visual regression; inject fixed values when deterministic proof is required."
    },
    {
      "id": "gsap-utils.value-helper-unit-string",
      "severity": "medium",
      "confidence": "medium",
      "category": "api-correctness",
      "pattern": "\\bgsap\\.utils\\.(?:clamp|mapRange|normalize)\\s*\\([^)]*['\"][^'\"]*(?:px|%|rem|em|deg|vh|vw)",
      "message": "Numeric gsap.utils helpers appear to receive CSS unit strings.",
      "recommendation": "Keep clamp(), mapRange(), and normalize() numeric; use getUnit(), unitize(), or a unit-preserving snap()/modifier flow when units matter."
    },
    {
      "id": "gsap-utils.snap-object-without-radius",
      "severity": "medium",
      "confidence": "medium",
      "category": "api-correctness",
      "pattern": "\\bgsap\\.utils\\.snap\\s*\\(\\s*\\{(?=[\\s\\S]{0,240}\\b(?:values|increment)\\s*:)(?![\\s\\S]{0,240}\\bradius\\s*:)",
      "message": "snap() object config with values/increment has no obvious radius.",
      "recommendation": "Use the simple increment/array form when every value should snap, or include radius deliberately for magnetic snapping."
    },
    {
      "id": "gsap-utils.distribute-auto-grid",
      "severity": "low",
      "confidence": "medium",
      "category": "runtime-boundary",
      "pattern": "\\bgsap\\.utils\\.distribute\\s*\\([\\s\\S]{0,500}grid\\s*:\\s*['\"]auto['\"]",
      "message": "distribute({ grid: 'auto' }) measures DOM layout.",
      "recommendation": "Run only where DOM layout is available and stable; avoid SSR/server code and avoid rebuilding the distributor in hot callbacks."
    },
    {
      "id": "gsap-utils.check-prefix-needs-fallback",
      "severity": "low",
      "confidence": "medium",
      "category": "compatibility",
      "pattern": "\\bgsap\\.utils\\.checkPrefix\\s*\\(",
      "message": "checkPrefix() can return undefined for unsupported properties.",
      "recommendation": "Verify callers handle an unsupported property before using the returned value as a string key."
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
const fileNames = new Set();
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
  scripts/audit.mjs scan --root . --format json --output gsap-utils-audit.json

Config:
  Optional .motion-audit.json at --root supports:
  {
    "ignoreRules": ["rule-id"],
    "ignorePaths": ["generated/", "fixtures/"],
    "ignores": [{"ruleId": "rule-id", "path": "src/example.tsx"}]
  }

Inline suppression:
  // motion-audit-ignore rule-id
  // motion-audit-ignore all
`;
}

function readOption(rest, flag) {
  const value = rest.shift();
  if (!value || value.startsWith('-')) throw new Error(`${flag} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = { command: null, root: process.cwd(), format: 'markdown', output: null, maxFiles: 2000 };
  const rest = [...argv];
  while (rest.length) {
    const arg = rest.shift();
    if (arg === '--help' || arg === '-h') args.help = true;
    else if (arg === '--json') args.format = 'json';
    else if (arg === '--root') args.root = path.resolve(readOption(rest, '--root'));
    else if (arg === '--format') args.format = readOption(rest, '--format');
    else if (arg === '--output') args.output = path.resolve(readOption(rest, '--output'));
    else if (arg === '--max-files') args.maxFiles = Number(readOption(rest, '--max-files'));
    else if (!arg.startsWith('-') && args.command === null) args.command = arg;
    else throw new Error(`Unknown argument: ${arg}`);
  }
  args.command = args.command ?? 'scan';
  if (!['scan', 'doctor'].includes(args.command)) throw new Error(`Unknown command: ${args.command}`);
  if (!['markdown', 'json'].includes(args.format)) throw new Error(`Unknown format: ${args.format}`);
  if (!Number.isInteger(args.maxFiles) || args.maxFiles < 1) throw new Error('--max-files must be a positive integer');
  return args;
}

function loadConfig(root) {
  const file = path.join(root, '.motion-audit.json');
  if (!fs.existsSync(file)) return { ignoreRules: [], ignorePaths: [], ignores: [] };
  try {
    const parsed = JSON.parse(fs.readFileSync(file, 'utf8'));
    return {
      ignoreRules: Array.isArray(parsed.ignoreRules) ? parsed.ignoreRules : [],
      ignorePaths: Array.isArray(parsed.ignorePaths) ? parsed.ignorePaths : [],
      ignores: Array.isArray(parsed.ignores) ? parsed.ignores : [],
    };
  } catch (error) {
    throw new Error(`Failed to parse .motion-audit.json: ${error.message}`);
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
  const extension = path.extname(fileName);
  if (normalized === 'scripts/audit.mjs' || normalized.endsWith('/scripts/audit.mjs')) return true;
  if (normalized.startsWith('evals/')) return true;
  if (/^skills\/[^/]+\/(?:agents|assets|examples|references|scripts|templates)(?:\/|$)/.test(normalized)) return true;
  if (path.basename(fileName) === 'SKILL.md' && /^skills\/[^/]+\/SKILL\.md$/.test(normalized)) return true;
  if (extension === '.json' && fileName !== 'package.json') return true;
  if (fullPath && (fileExtensions.has(extension) || fileNames.has(fileName))) {
    try {
      return fs.readFileSync(fullPath, 'utf8').slice(0, 512).includes('motion-audit-skip-file');
    } catch {
      return true;
    }
  }
  return false;
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
      } else if (entry.isFile() && !shouldSkipFile(rel, entry.name, full) && fileExtensions.has(path.extname(entry.name))) {
        files.push(full);
      }
      if (files.length >= maxFiles) return;
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
    return { exists: true, packages: new Set(Object.keys(deps ?? {})), scripts: pkg.scripts ?? {} };
  } catch {
    return { exists: true, packages: new Set(), scripts: {} };
  }
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
  return nearby.includes('motion-audit-ignore all') || nearby.includes(`motion-audit-ignore ${ruleId}`);
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

function hasTopLevelSecondArgument(text, openParenIndex) {
  let depth = 1;
  let quote = null;
  let escaped = false;
  for (let index = openParenIndex + 1; index < text.length; index += 1) {
    const char = text[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
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
      if (depth === 0 && char === ')') return false;
      continue;
    }
    if (char === ',' && depth === 1) return true;
  }
  return false;
}

function isLikelySelectorText(value) {
  const selector = value.trim();
  return /^[.#\[]/.test(selector) || /^[A-Za-z][A-Za-z0-9_-]*(?:$|[\s>+~.#:[\]])/.test(selector);
}

function selectorIdentifiers(text) {
  const names = new Set();
  for (const match of text.matchAll(/\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*(['"`])([^'"`\n]+)\2/g)) {
    if (isLikelySelectorText(match[3] ?? '')) names.add(match[1]);
  }
  return names;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
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
  if (rule.id === 'gsap-utils.to-array-unscoped-selector') {
    const selectorNames = selectorIdentifiers(text);
    const regex = /\bgsap\.utils\.toArray\s*\(\s*(?:(['"`])([^'"`\n]+)\1|([A-Za-z_$][\w$]*))/g;
    for (const match of text.matchAll(regex)) {
      const literalSelector = match[2];
      const selectorName = match[3];
      if (literalSelector && !isLikelySelectorText(literalSelector)) continue;
      if (selectorName && !selectorNames.has(selectorName)) continue;
      const openParenIndex = (match.index ?? 0) + match[0].lastIndexOf('(');
      if (hasTopLevelSecondArgument(text, openParenIndex)) continue;
      const line = lineForIndex(text, match.index ?? 0);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
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
  const pkg = readPackage(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageHasAny') {
      const matched = (rule.packages ?? []).filter((name) => pkg.packages.has(name));
      if (matched.length > 0) {
        if (isIgnored(config, rule.id, 'package.json', [''], 1)) continue;
        findings.push({
          id: `${rule.id}:package.json:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: 'package.json',
          line: 1,
          excerpt: `matched packages: ${matched.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
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
    gsapDependency: pkg.packages.has('gsap'),
    configuredRules: profile.rules.length,
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'use --format json for machine-readable output',
      'rules are heuristics; verify each finding against current code before editing',
      'findings can be suppressed with .motion-audit.json or inline motion-audit-ignore comments',
    ],
  };
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    return `# Motion Audit Doctor: ${result.profile}

- Root: ${result.root}
- Package JSON: ${result.packageJson ? 'yes' : 'no'}
- GSAP dependency: ${result.gsapDependency ? 'yes' : 'no'}
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
  return `# Motion Audit Report: ${result.profile}

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
