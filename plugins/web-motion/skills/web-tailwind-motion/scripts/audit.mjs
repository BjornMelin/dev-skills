#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "web-tailwind-motion",
  "rules": [
    {
      "id": "tailwind.dynamic-class",
      "severity": "medium",
      "confidence": "medium",
      "category": "build",
      "pattern": "class(Name)?\\s*=\\s*{[\\s\\S]{0,400}(\\$\\{|join\\(|clsx\\([\\s\\S]{0,240}\\$\\{|cn\\([\\s\\S]{0,240}\\$\\{|[\"'`](?:(?:[A-Za-z0-9_-]+:)*)(?:duration|delay|ease|animate|transition)-[\"'`]\\s*\\+|\\+\\s*[\"'`](?:(?:[A-Za-z0-9_-]+:)*)(?:duration|delay|ease|animate|transition)-)|(?:clsx|cn)\\([\\s\\S]{0,300}([\"'`](?:(?:[A-Za-z0-9_-]+:)*)(?:duration|delay|ease|animate|transition)-[\"'`]\\s*\\+|\\+\\s*[\"'`](?:(?:[A-Za-z0-9_-]+:)*)(?:duration|delay|ease|animate|transition)-)|\\b(?:const|let|var)\\s+[A-Za-z_$][\\w$]*(?:\\s*:\\s*[^=\\n;]+)?\\s*=\\s*(?:[\"'`](?:(?:[A-Za-z0-9_-]+:)*)(?:duration|delay|ease|animate|transition)-[\"'`]\\s*\\+|[^;\\n]{1,160}\\+\\s*[\"'`](?:(?:[A-Za-z0-9_-]+:)*)(?:duration|delay|ease|animate|transition)-)|class\\s*=\\s*[\"'][^\"']*\\$\\{",
      "message": "Dynamic class construction can hide Tailwind classes from extraction.",
      "recommendation": "Use explicit class maps or @source inline() for a finite generated class set."
    },
    {
      "id": "tailwind.dynamic-motion-token",
      "severity": "medium",
      "confidence": "high",
      "category": "build",
      "pattern": "`[^`]*(duration|delay|ease|animate|transition)-\\$\\{[^`]+\\}[^`]*`",
      "message": "Motion utility names are interpolated inside a template literal.",
      "recommendation": "Map variants to complete classes or register the finite set with @source inline()."
    },
    {
      "id": "tailwind.v3-config-safelist-review",
      "severity": "low",
      "confidence": "medium",
      "category": "build",
      "pattern": "\\b(content|safelist)\\s*:",
      "message": "A v3-style Tailwind content/safelist boundary was found.",
      "recommendation": "If the project is Tailwind v4, prefer stylesheet-owned @source, @source not, source(none), and @source inline() boundaries; keep config content/safelist only for v3 or documented compat boundaries."
    },
    {
      "id": "tailwind.arbitrary-motion-value",
      "severity": "low",
      "confidence": "medium",
      "category": "maintainability",
      "pattern": "(?<!-)\\b(?:animate|transition|duration|delay|ease)-\\[[^\\]\\n]+\\]",
      "message": "Arbitrary Tailwind motion values can become a parallel token system.",
      "recommendation": "Keep one-offs local; promote repeated values to @theme tokens or component CSS."
    },
    {
      "id": "tailwind.animate-token-outside-theme",
      "severity": "medium",
      "confidence": "medium",
      "category": "build",
      "kind": "animateTokenOutsideTheme",
      "include": "--animate-[A-Za-z0-9_-]+\\s*:",
      "without": "@theme\\b",
      "message": "Animation utility-looking tokens appear outside an @theme boundary.",
      "recommendation": "Define reusable animate-* utilities with top-level @theme --animate-*; use :root only for ordinary CSS variables that should not generate utilities."
    },
    {
      "id": "motion.transition-all",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "\\btransition\\s*:\\s*['\"]?all\\b|\\bclass(Name)?\\s*=\\s*([\"'][^\"'\\n]*|{[^}\\n]*)(?<![\\w.-])transition-all(?![\\w-])",
      "message": "Broad transition-all can animate future expensive properties unintentionally.",
      "recommendation": "Transition only the properties that should move, usually transform and opacity."
    },
    {
      "id": "tailwind.infinite-animation-without-motion-variant",
      "severity": "low",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsBothWithout",
      "include": "\\banimate-(spin|ping|pulse|bounce)\\b",
      "also": "\\b(class|className)\\b",
      "without": "motion-safe|motion-reduce|prefers-reduced-motion",
      "message": "Built-in looping animation appears without a reduced-motion variant in the same file.",
      "recommendation": "Use motion-safe: for decorative loops and motion-reduce:animate-none or a non-spatial substitute."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\b(gsap\\.|motion\\.|<motion\\.|withRepeat\\(|withTiming\\(|withSpring\\(|useFrame\\(|lottie|Rive|Skia|animate\\(|@keyframes)\\b",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|AccessibilityInfo|reduceMotion",
      "message": "Motion code was found without an obvious reduced-motion branch in the same file.",
      "recommendation": "Add reduced-motion behavior or document why this effect is essential and already handled elsewhere."
    }
  ]
};

const skipDirs = new Set([
  '.git', 'node_modules', '.next', '.nuxt', 'dist', 'build', 'coverage',
  '.expo', '.turbo', '.vercel', '.cache', '.codex', '.agents',
  'output', 'tmp', 'temp', 'vendor', 'playwright-report', 'storybook-static',
]);
const fileExtensions = new Set([
  '.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.mts', '.cts', '.css',
  '.scss', '.sass', '.html', '.vue', '.svelte', '.json',
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

function listFiles(root, maxFiles) {
  const files = [];
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel)) walk(full);
      } else if (entry.isFile() && fileExtensions.has(path.extname(entry.name))) {
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
  if (!fs.existsSync(file)) {
    return { exists: false, packages: new Set(), versions: {}, scripts: {} };
  }
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return {
      exists: true,
      packages: new Set(Object.keys(deps ?? {})),
      versions: deps ?? {},
      scripts: pkg.scripts ?? {},
    };
  } catch {
    return { exists: true, packages: new Set(), versions: {}, scripts: {} };
  }
}

function findCssEntrypoints(root, maxFiles) {
  const candidates = listFiles(root, maxFiles)
    .filter((file) => ['.css', '.scss', '.sass'].includes(path.extname(file)))
    .map((file) => {
      let text = '';
      try {
        text = fs.readFileSync(file, 'utf8');
      } catch {
        return null;
      }
      return {
        file: path.relative(root, file),
        importsTailwind: text.includes('@import "tailwindcss"') || text.includes("@import 'tailwindcss'"),
        hasTheme: /@theme\b/.test(text),
        hasSource: /@source\b|source\(none\)/.test(text),
        hasInlineSource: /@source\s+(?:not\s+)?inline\(/.test(text),
        hasReducedMotionMedia: /prefers-reduced-motion/.test(text),
        hasAnimateTheme: /@theme[\s\S]*--animate-[A-Za-z0-9_-]+\s*:/.test(text),
        hasDefaultTransitionTheme: /--default-transition-(duration|timing-function)\s*:/.test(text),
      };
    })
    .filter(Boolean)
    .filter((entry) => entry.importsTailwind || entry.hasTheme || entry.hasSource);
  return candidates.slice(0, 20);
}

function findTailwindConfigFiles(root, maxFiles) {
  return listFiles(root, maxFiles)
    .filter((file) => /^tailwind\.config\.(js|cjs|mjs|ts|cts|mts)$/.test(path.basename(file)))
    .map((file) => {
      let text = '';
      try {
        text = fs.readFileSync(file, 'utf8');
      } catch {
        return null;
      }
      return {
        file: path.relative(root, file),
        hasContent: /\bcontent\s*:/.test(text),
        hasSafelist: /\bsafelist\s*:/.test(text),
        hasAnimationExtend: /\banimation\s*:|keyframes\s*:/.test(text),
        hasTransitionExtend: /transition(Property|Duration|TimingFunction)|transition\s*:/.test(text),
      };
    })
    .filter(Boolean)
    .slice(0, 20);
}

function tailwindPackageInfo(pkg) {
  return Object.fromEntries(
    Object.entries(pkg.versions ?? {})
      .filter(([name]) => name === 'tailwindcss' || name.startsWith('@tailwindcss/'))
      .sort(([left], [right]) => left.localeCompare(right)),
  );
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
  return {
    id: `${rule.id}:${relativePath}:${line}`,
    ruleId: rule.id,
    severity: rule.severity,
    confidence: rule.confidence,
    category: rule.category,
    file: relativePath,
    line,
    excerpt,
    rationale: rule.message,
    recommendation: rule.recommendation,
  };
}

function ruleRegex(pattern) {
  return new RegExp(pattern, 'gms');
}

function matchingBrace(text, openIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  for (let index = openIndex; index < text.length; index += 1) {
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
    if (char === '{') depth += 1;
    else if (char === '}') {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function openBraceStackAt(text, index) {
  const stack = [];
  let quote = null;
  let escaped = false;
  for (let cursor = 0; cursor < index; cursor += 1) {
    const char = text[cursor];
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
    if (char === '{') stack.push(cursor);
    else if (char === '}') stack.pop();
  }
  return stack;
}

function stripCommentsPreserveLength(text) {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, (match) => match.replace(/[^\n\r]/g, ' '))
    .replace(/(^|[\s;{}])\/\/[^\n\r]*/g, (match, prefix) => prefix + ' '.repeat(match.length - prefix.length));
}

function isInsideThemeBlock(text, index) {
  const cleanText = stripCommentsPreserveLength(text);
  const stack = openBraceStackAt(cleanText, index);
  for (const open of stack.slice().reverse()) {
    const close = matchingBrace(cleanText, open);
    if (close !== -1 && index > close) continue;
    const previousOpen = cleanText.lastIndexOf('{', open - 1);
    const previousClose = cleanText.lastIndexOf('}', open - 1);
    const prelude = cleanText
      .slice(Math.max(previousOpen, previousClose) + 1, open)
      .replace(/(^|[\s;{}])\/\/[^\n\r]*/g, '$1 ');
    if (/@theme\b/.test(prelude)) return stack[0] === open;
  }
  return false;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'animateTokenOutsideTheme') {
    const regex = ruleRegex(rule.include);
    const cleanText = stripCommentsPreserveLength(text);
    for (const match of cleanText.matchAll(regex)) {
      const index = match.index ?? 0;
      if (isInsideThemeBlock(text, index)) continue;
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
    tailwindPackages: tailwindPackageInfo(pkg),
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
  const cssEntrypoints = fs.existsSync(root) ? findCssEntrypoints(root, maxFiles) : [];
  const tailwindConfigFiles = fs.existsSync(root) ? findTailwindConfigFiles(root, maxFiles) : [];
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    tailwindPackages: tailwindPackageInfo(pkg),
    cssEntrypoints,
    tailwindConfigFiles,
    configuredRules: profile.rules.length,
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'use --format json for machine-readable output',
      'Tailwind v4 source generation should prefer complete class names or @source inline() for finite sets',
      'findings can be suppressed with .motion-audit.json or inline motion-audit-ignore comments',
    ],
  };
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    const packages = Object.entries(result.tailwindPackages)
      .map(([name, version]) => `${name}@${version}`)
      .join(', ') || 'none found';
    const entrypoints = result.cssEntrypoints
      .map((entry) => `  - ${entry.file} (tailwind=${entry.importsTailwind ? 'yes' : 'no'}, theme=${entry.hasTheme ? 'yes' : 'no'}, animate-theme=${entry.hasAnimateTheme ? 'yes' : 'no'}, source=${entry.hasSource ? 'yes' : 'no'}, inline-source=${entry.hasInlineSource ? 'yes' : 'no'}, reduced-motion=${entry.hasReducedMotionMedia ? 'yes' : 'no'})`)
      .join('\n') || '  - none found';
    const configs = (result.tailwindConfigFiles ?? [])
      .map((entry) => `  - ${entry.file} (content=${entry.hasContent ? 'yes' : 'no'}, safelist=${entry.hasSafelist ? 'yes' : 'no'}, animation=${entry.hasAnimationExtend ? 'yes' : 'no'}, transition=${entry.hasTransitionExtend ? 'yes' : 'no'})`)
      .join('\n') || '  - none found';
    return `# Motion Audit Doctor: ${result.profile}

- Root: ${result.root}
- Package JSON: ${result.packageJson ? 'yes' : 'no'}
- Tailwind packages: ${packages}
- Config file: ${result.configFile ? 'yes' : 'no'}
- Configured rules: ${result.configuredRules}
- Sample file count: ${result.sampleFileCount}
- Status: ${result.ok ? 'ok' : 'failed'}

## CSS Entrypoints

${entrypoints}

## Tailwind Config Files

${configs}
`;
  }
  const packages = Object.entries(result.tailwindPackages)
    .map(([name, version]) => `${name}@${version}`)
    .join(', ') || 'none found';
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
- Tailwind packages: ${packages}
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
