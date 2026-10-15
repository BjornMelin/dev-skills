#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const currentScript = fs.realpathSync(fileURLToPath(import.meta.url));

const profile = {
  "skillName": "gsap-performance",
  "rules": [
    {
      "id": "gsap.private-registry",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "pattern": "npm\\.greensock\\.com|GREEN.?SOCK.*TOKEN|greensock.*authToken|gsap.*license.?key",
      "message": "Outdated private GSAP registry/token guidance or config is present.",
      "recommendation": "Use the public gsap npm package; modern GSAP plugins do not require a private registry token."
    },
    {
      "id": "gsap.trial-package",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageHasAny",
      "packages": ["gsap-trial"],
      "message": "The trial GSAP package is installed in application dependencies.",
      "recommendation": "Use the public gsap package for production code unless the repo has a documented trial-only demo boundary."
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
      "id": "gsap.to-hot-path",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "gsap\\.to\\(",
      "also": "mousemove|pointermove|touchmove|scroll|resize|requestAnimationFrame|useFrame\\(|ticker\\.add",
      "message": "A tween may be created inside a high-frequency event or frame path.",
      "recommendation": "Create the tween once, then update progress/vars, or use quickSetter()/quickTo() for repeated value updates."
    },
    {
      "id": "gsap.quick-setter-transform-function",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "gsap\\.(quickSetter|quickTo)\\s*\\([^\\n]*,\\s*['\"](translate[XYZ]?|rotate|rotate[XYZ]|scale[XY]?|skew[XY]?)['\"]",
      "message": "quickSetter()/quickTo() is using a CSS transform function name instead of a GSAP transform alias.",
      "recommendation": "Use GSAP aliases like x, y, rotation, scale, scaleX, or skewX; aliases avoid transform-string parsing and match GSAP's optimized property path."
    },
    {
      "id": "gsap.ticker-missing-remove",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "gsap\\.ticker\\.add\\(",
      "without": "gsap\\.ticker\\.remove\\(",
      "message": "A GSAP ticker listener is added without an obvious paired remove.",
      "recommendation": "Store the listener function and remove it during unmount/teardown."
    },
    {
      "id": "gsap.force3d-overuse",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "\\bforce3D\\s*:\\s*true\\b|gsap\\.config\\s*\\([^)]*force3D\\s*:\\s*true",
      "message": "force3D is explicitly enabled and may create unnecessary layers.",
      "recommendation": "Prefer GSAP's default force3D:\"auto\" unless profiling proves a specific element needs explicit promotion."
    },
    {
      "id": "gsap.lazy-disabled",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "\\blazy\\s*:\\s*false\\b",
      "message": "GSAP lazy rendering is disabled for at least one tween/config block.",
      "recommendation": "Keep lazy rendering enabled unless measured first-render timing requires this exception."
    },
    {
      "id": "gsap.lag-smoothing-disabled",
      "severity": "medium",
      "confidence": "medium",
      "category": "timing",
      "pattern": "gsap\\.ticker\\.lagSmoothing\\s*\\(\\s*(0|false)\\s*[,)]",
      "message": "GSAP ticker lag smoothing appears to be disabled globally.",
      "recommendation": "Avoid disabling lagSmoothing() globally unless the app documents the timing tradeoff and accepts catch-up jumps after long frames."
    },
    {
      "id": "motion.transition-all",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "\\btransition\\s*:\\s*['\"]?all\\b|\\btransition-all\\b",
      "message": "Broad transition-all can animate future expensive properties unintentionally.",
      "recommendation": "Transition only the properties that should move, usually transform and opacity."
    },
    {
      "id": "motion.will-change-broad",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "\\bwill-change\\s*:\\s*(all|contents|scroll-position)\\b|\\bwillChange\\s*=\\s*['\"](all|contents|scroll-position)['\"]",
      "message": "Broad will-change can force wasteful layer or rendering preparation.",
      "recommendation": "Use will-change only for specific active properties, usually transform or opacity, and remove temporary hints after the animation."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "context": "(?:\\b(?:gsap\\.|ScrollTrigger\\.|animate\\(|transition)\\b|@keyframes\\b)",
      "radius": 420,
      "pattern": "\\b(width|height|top|left|right|bottom|margin|padding)\\s*:",
      "message": "Layout-affecting properties are being animated or configured near motion code.",
      "recommendation": "Prefer transform and opacity in hot paths; measure before keeping layout animation."
    },
    {
      "id": "scrolltrigger.refresh-hot-path",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "ScrollTrigger\\.refresh\\(",
      "also": "mousemove|pointermove|touchmove|scroll|resize|requestAnimationFrame|ticker\\.add|onUpdate",
      "message": "ScrollTrigger.refresh() appears near high-frequency event or frame code.",
      "recommendation": "Call refresh only after layout changes, debounce app-level refreshes, and use ScrollTrigger.refresh(true) when layout needs a render tick before measurement."
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
      } else if (
        entry.isFile() &&
        !shouldSkipFile(rel, entry.name, full) &&
        (fileExtensions.has(path.extname(entry.name)) || fileNames.has(entry.name)) &&
        fs.realpathSync(full) !== currentScript
      ) {
        files.push(full);
      }
      if (files.length >= maxFiles) return;
    }
  }
  walk(root);
  return files;
}

function listPackageFiles(root, maxFiles) {
  const files = [];
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel)) walk(full);
      } else if (entry.isFile() && entry.name === 'package.json') {
        files.push(full);
      }
      if (files.length >= maxFiles) return;
    }
  }
  walk(root);
  return files;
}

function readPackage(root) {
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), packageVersions: {}, scripts: {}, packageFiles: [] };
  const packages = new Set();
  const packageVersions = {};
  const scripts = {};
  const packageFiles = [];
  for (const file of listPackageFiles(root, 2000)) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
      const names = new Set(Object.keys(deps ?? {}));
      for (const name of names) packages.add(name);
      Object.assign(packageVersions, deps ?? {});
      Object.assign(scripts, pkg.scripts ?? {});
      packageFiles.push({
        file: path.relative(root, file),
        packages: names,
        packageVersions: deps ?? {},
        scripts: pkg.scripts ?? {},
      });
    } catch {
      continue;
    }
  }
  return { exists: packageFiles.length > 0, packages, packageVersions, scripts, packageFiles };
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

function matchingCloseBrace(text, openIndex) {
  let depth = 0;
  for (let cursor = openIndex; cursor < text.length; cursor += 1) {
    if (text[cursor] === '{') depth += 1;
    else if (text[cursor] === '}') {
      depth -= 1;
      if (depth === 0) return cursor;
    }
  }
  return -1;
}

function cssContextAround(text, index) {
  const stack = openBraceStackAt(text, index);
  const innerOpen = stack.at(-1);
  const innerClose = innerOpen == null ? -1 : matchingCloseBrace(text, innerOpen);
  if (innerOpen == null || innerClose < index) {
    const lineStart = text.lastIndexOf('\n', index) + 1;
    const lineEnd = text.indexOf('\n', index);
    return text.slice(lineStart, lineEnd >= 0 ? lineEnd : text.length);
  }
  let context = text.slice(innerOpen, innerClose + 1);
  for (const openIndex of stack.slice().reverse()) {
    const previousOpen = text.lastIndexOf('{', openIndex - 1);
    const previousClose = text.lastIndexOf('}', openIndex - 1);
    const preludeStart = Math.max(previousOpen, previousClose) + 1;
    const prelude = text.slice(preludeStart, openIndex);
    if (/@keyframes\b/i.test(prelude)) {
      context = prelude + context;
      break;
    }
  }
  return context;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.requires && !ruleRegex(rule.requires).test(text)) return findings;
  if (rule.context) {
    const regex = ruleRegex(rule.pattern);
    const contextRegex = ruleRegex(rule.context);
    const radius = Number.isFinite(rule.radius) ? rule.radius : 420;
    const seenLines = new Set();
    for (const match of text.matchAll(regex)) {
      const index = match.index ?? 0;
      const isCssLike = /\.(css|scss|sass)$/.test(file);
      const nearby = isCssLike
        ? cssContextAround(text, index)
        : text.slice(Math.max(0, index - radius), Math.min(text.length, index + radius));
      contextRegex.lastIndex = 0;
      if (!contextRegex.test(nearby)) continue;
      const line = lineForIndex(text, index);
      if (seenLines.has(line)) continue;
      seenLines.add(line);
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
      for (const packageFile of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageFile.packages.has(name));
        if (matched.length === 0) continue;
        if (isIgnored(config, rule.id, packageFile.file, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packageFile.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageFile.file,
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
    packages: {
      gsap: pkg.packageVersions?.gsap ?? null,
      gsapTrial: pkg.packageVersions?.['gsap-trial'] ?? null,
    },
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
    packages: {
      gsap: pkg.packageVersions?.gsap ?? null,
      gsapTrial: pkg.packageVersions?.['gsap-trial'] ?? null,
    },
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
- GSAP package: ${result.packages.gsap ?? 'not found'}
- GSAP trial package: ${result.packages.gsapTrial ?? 'not found'}
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
- GSAP package: ${result.packages.gsap ?? 'not found'}
- GSAP trial package: ${result.packages.gsapTrial ?? 'not found'}
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
