#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "gsap-timeline",
  "rules": [
    {
      "id": "gsap.private-registry",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      // motion-audit-ignore gsap.private-registry
      "pattern": "npm\\.greensock\\.com|GREEN.?SOCK.*TOKEN|greensock.*authToken|gsap.*license.?key",
      "message": "Outdated private GSAP registry/token guidance or config is present.",
      "recommendation": "Use the public gsap npm package; modern GSAP plugins do not require a private registry token."
    },
    {
      "id": "gsap.timeline-per-frame",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "gsap\\.timeline\\(",
      "also": "requestAnimationFrame|useFrame\\(|ticker\\.add",
      "message": "A GSAP timeline may be created from a per-frame loop.",
      "recommendation": "Create timelines during setup and update progress or values inside frame callbacks."
    },
    {
      "id": "gsap.timeline-delay-sequencing",
      "severity": "low",
      "confidence": "medium",
      "category": "maintainability",
      "kind": "fileContainsBoth",
      "include": "gsap\\.timeline\\(",
      "also": "\\bdelay\\s*:",
      "message": "Timeline code uses delay vars near a GSAP timeline.",
      "recommendation": "Use timeline position parameters for sequencing; keep delay only for an intentional whole-timeline or independent child start delay."
    },
    {
      "id": "gsap.timeline-duration-vars",
      "severity": "low",
      "confidence": "medium",
      "category": "correctness",
      "kind": "timelineConstructorTopLevelKey",
      "key": "duration",
      "message": "A timeline constructor appears to include duration.",
      "recommendation": "Timeline duration is derived from child animations; put shared child duration in defaults or tween vars."
    },
    {
      "id": "gsap.deprecated-duration-signature",
      "severity": "medium",
      "confidence": "medium",
      "category": "correctness",
      "requires": "gsap\\.|TimelineMax|TimelineLite",
      "pattern": "\\.(to|from|fromTo)\\s*\\(\\s*[^,\\n]+,\\s*\\d+(?:\\.\\d+)?\\s*,",
      "message": "Timeline code may be using a deprecated GSAP 2 duration argument signature.",
      "recommendation": "Use GSAP 3 signatures and put duration inside vars, for example tl.to(target, { duration: 1, x: 100 }, position)."
    },
    {
      "id": "gsap.scrolltrigger-child-tween",
      "severity": "medium",
      "confidence": "medium",
      "category": "correctness",
      "kind": "fileContainsBoth",
      "include": "gsap\\.timeline\\(",
      "also": "\\.(to|from|fromTo)\\([\\s\\S]{0,500}?scrollTrigger\\s*:",
      "message": "A tween inside timeline-looking code may define its own ScrollTrigger.",
      "recommendation": "Put ScrollTrigger on the top-level timeline or top-level tween; do not nest ScrollTriggered child tweens inside a parent timeline."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "gsap\\.|ScrollTrigger",
      "also": "\\b(width|height|top|left|right|bottom|margin|padding)\\s*:",
      "message": "Layout-affecting properties are being animated or configured near motion code.",
      "recommendation": "Prefer transform and opacity in hot paths; measure before keeping layout animation."
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
  scripts/audit.mjs scan --root . --format json --output motion-audit.json

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
      } else if (entry.isFile() && !shouldSkipFile(rel, entry.name, full) && (fileExtensions.has(path.extname(entry.name)) || fileNames.has(entry.name))) {
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

function nextCodeIndex(text, index) {
  let i = index;
  while (i < text.length && /\s/.test(text[i])) i += 1;
  return i;
}

function findBalancedEnd(text, openIndex, openChar, closeChar) {
  let depth = 1;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let i = openIndex + 1; i < text.length; i += 1) {
    const char = text[i];
    const next = text[i + 1];
    if (lineComment) {
      if (char === '\n') lineComment = false;
      continue;
    }
    if (blockComment) {
      if (char === '*' && next === '/') {
        blockComment = false;
        i += 1;
      }
      continue;
    }
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
    if (char === '/' && next === '/') {
      lineComment = true;
      i += 1;
      continue;
    }
    if (char === '/' && next === '*') {
      blockComment = true;
      i += 1;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
      continue;
    }
    if (char === openChar) depth += 1;
    else if (char === closeChar) {
      depth -= 1;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function findStringEnd(text, quoteIndex) {
  const quote = text[quoteIndex];
  let escaped = false;
  for (let i = quoteIndex + 1; i < text.length; i += 1) {
    const char = text[i];
    if (escaped) escaped = false;
    else if (char === '\\') escaped = true;
    else if (char === quote) return i;
  }
  return -1;
}

function findTopLevelObjectKey(text, objectStart, objectEnd, key) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let i = objectStart + 1; i < objectEnd; i += 1) {
    const char = text[i];
    const next = text[i + 1];
    if (lineComment) {
      if (char === '\n') lineComment = false;
      continue;
    }
    if (blockComment) {
      if (char === '*' && next === '/') {
        blockComment = false;
        i += 1;
      }
      continue;
    }
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
    if (char === '/' && next === '/') {
      lineComment = true;
      i += 1;
      continue;
    }
    if (char === '/' && next === '*') {
      blockComment = true;
      i += 1;
      continue;
    }
    if (char === '"' || char === "'") {
      if (depth === 0) {
        const keyStart = i + 1;
        const end = findStringEnd(text, i);
        if (end !== -1) {
          const candidate = text.slice(keyStart, end);
          const colonIndex = nextCodeIndex(text, end + 1);
          if (candidate === key && text[colonIndex] === ':') return i;
          i = end;
          continue;
        }
      }
      quote = char;
      continue;
    }
    if (char === '`') {
      quote = char;
      continue;
    }
    if (char === '{' || char === '[' || char === '(') {
      depth += 1;
      continue;
    }
    if (char === '}' || char === ']' || char === ')') {
      depth -= 1;
      continue;
    }
    if (depth === 0 && /[A-Za-z_$]/.test(char)) {
      const match = text.slice(i).match(/^[A-Za-z_$][A-Za-z0-9_$]*/);
      if (match) {
        const candidate = match[0];
        const colonIndex = nextCodeIndex(text, i + candidate.length);
        if (candidate === key && text[colonIndex] === ':') return i;
        i += candidate.length - 1;
      }
    }
  }
  return -1;
}

function findTimelineConstructorTopLevelKeys(text, key) {
  const matches = [];
  const timelineCall = /gsap\s*\.\s*timeline\s*\(/g;
  for (const match of text.matchAll(timelineCall)) {
    const openParen = text.indexOf('(', match.index ?? 0);
    const argStart = nextCodeIndex(text, openParen + 1);
    if (text[argStart] !== '{') continue;
    const objectEnd = findBalancedEnd(text, argStart, '{', '}');
    if (objectEnd === -1) continue;
    const keyIndex = findTopLevelObjectKey(text, argStart, objectEnd, key);
    if (keyIndex !== -1) matches.push(keyIndex);
  }
  return matches;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.requires && !ruleRegex(rule.requires).test(text)) return findings;
  if (rule.kind === 'timelineConstructorTopLevelKey') {
    for (const index of findTimelineConstructorTopLevelKeys(text, rule.key)) {
      const line = lineForIndex(text, index);
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
  const pkg = readPackage(root);
  const findings = [];
  const fileTexts = new Map();
  for (const file of files) {
    try {
      fileTexts.set(file, fs.readFileSync(file, 'utf8'));
    } catch {}
  }
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
    for (const [file, text] of fileTexts) {
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
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'use --format json for machine-readable output',
      'findings can be suppressed with .motion-audit.json or inline motion-audit-ignore comments',
    ],
  };
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    return `# Motion Audit Doctor: ${result.profile}

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
