#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const currentScript = fs.realpathSync(fileURLToPath(import.meta.url));

const profile = {
  "skillName": "gsap-plugins",
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
      "message": "The project depends on gsap-trial even though current GSAP plugins are available from the public gsap package.",
      "recommendation": "Remove gsap-trial and import plugins from the public gsap package."
    },
    {
      "id": "gsap.trial-import",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "pattern": "from\\s+['\"]gsap-trial(?:/[^'\"]*)?['\"]|require\\(\\s*['\"]gsap-trial(?:/[^'\"]*)?['\"]\\s*\\)",
      "message": "Code imports GSAP from gsap-trial.",
      "recommendation": "Import GSAP and plugins from the public gsap package."
    },
    {
      "id": "gsap.motionpathhelper-plugin-name",
      "severity": "high",
      "confidence": "high",
      "category": "api",
      "pattern": "\\bMotionPathHelperPlugin\\b",
      "message": "MotionPathHelperPlugin appears in code, but current GSAP exports MotionPathHelper.",
      "recommendation": "Import and register MotionPathHelper from gsap/MotionPathHelper."
    },
    {
      "id": "gsap.motionpathhelper-old-signature",
      "severity": "high",
      "confidence": "high",
      "category": "api",
      "pattern": "MotionPathHelper\\.create\\([^,]+,\\s*['\"][^'\"]+['\"]\\s*,",
      "message": "MotionPathHelper.create appears to use the old target, path, vars signature.",
      "recommendation": "Use MotionPathHelper.create(target, vars) and pass the path inside the vars object."
    },
    {
      "id": "gsap.scrolltrigger-register",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "fileContainsWithout",
      "include": "ScrollTrigger",
      "without": "registerPlugin\\([^)]*ScrollTrigger",
      "message": "ScrollTrigger is referenced without an obvious gsap.registerPlugin(ScrollTrigger).",
      "recommendation": "Register ScrollTrigger once before creating triggers."
    },
    {
      "id": "gsap.scrollsmoother-scrolltrigger",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "fileContainsWithout",
      "include": "ScrollSmoother",
      "without": "ScrollTrigger",
      "message": "ScrollSmoother is referenced without an obvious ScrollTrigger import or registration in the same file.",
      "recommendation": "Register ScrollTrigger and ScrollSmoother before creating a ScrollSmoother instance."
    },
    {
      "id": "gsap.scrollto-plugin-reference",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "fileContainsWithout",
      "include": "scrollTo\\s*:",
      "without": "ScrollToPlugin|registerPlugin\\(",
      "message": "scrollTo tween vars are used without an obvious ScrollToPlugin reference in the same file.",
      "recommendation": "Import and register ScrollToPlugin before using scrollTo tween vars, or document central registration."
    },
    {
      "id": "gsap.splittext-autosplit-onsplit",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "SplitText\\.create",
      "also": "autoSplit\\s*:\\s*true",
      "without": "onSplit\\s*\\(",
      "message": "SplitText autoSplit is used without an obvious onSplit callback in the same file.",
      "recommendation": "Create and return SplitText animations inside onSplit() so re-splits target fresh elements and old animations are cleaned up."
    },
    {
      "id": "gsap.draggable-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "Draggable\\.create\\(",
      "without": "\\.kill\\(|gsap\\.context\\(|useGSAP\\(|ctx\\.revert\\(|contextSafe\\(",
      "message": "Draggable.create is used without obvious lifecycle cleanup in the same file.",
      "recommendation": "Keep the Draggable instance and call kill() on cleanup, or use gsap.context()/useGSAP lifecycle cleanup."
    },
    {
      "id": "gsap.observer-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "Observer\\.create\\(",
      "without": "\\.kill\\(|gsap\\.context\\(|useGSAP\\(|ctx\\.revert\\(|contextSafe\\(",
      "message": "Observer.create is used without obvious lifecycle cleanup in the same file.",
      "recommendation": "Keep the Observer instance and call kill() on cleanup, or use framework lifecycle cleanup."
    },
    {
      "id": "gsap.splittext-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "SplitText\\.create\\(",
      "without": "\\.revert\\(|\\.kill\\(|gsap\\.context\\(|useGSAP\\(|ctx\\.revert\\(",
      "message": "SplitText.create is used without an obvious revert/kill or GSAP context cleanup in the same file.",
      "recommendation": "Call split.revert()/split.kill() on cleanup, or create SplitText inside gsap.context()/useGSAP cleanup."
    },
    {
      "id": "gsap.scrollsmoother-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "ScrollSmoother\\.create\\(",
      "without": "ScrollSmoother\\.get\\(\\)\\?\\.kill\\(|\\.kill\\(|ctx\\.revert\\(|useGSAP\\(",
      "message": "ScrollSmoother.create is used without obvious cleanup in the same file.",
      "recommendation": "Kill the ScrollSmoother instance on route/component cleanup, or document centralized teardown."
    },
    {
      "id": "gsap.drawsvg-visible-stroke",
      "severity": "low",
      "confidence": "medium",
      "category": "svg",
      "kind": "fileContainsWithout",
      "include": "drawSVG\\s*:",
      "without": "stroke\\s*=|stroke-|strokeWidth|stroke-width",
      "message": "DrawSVG is used without an obvious visible stroke declaration in the same file.",
      "recommendation": "Ensure the target SVG elements have stroke and stroke-width in markup or CSS."
    },
    {
      "id": "gsap.gsdevtools-production",
      "severity": "medium",
      "confidence": "medium",
      "category": "dependency",
      "pattern": "from\\s+['\"]gsap/GSDevTools['\"]|GSDevTools\\.create\\(",
      "message": "GSDevTools appears in source code.",
      "recommendation": "Keep GSDevTools development-only or remove it from production bundles."
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
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), scripts: {}, versions: {}, packageFiles: [] };
  const packages = new Set();
  const scripts = {};
  const versions = {};
  const packageFiles = [];
  for (const file of listPackageFiles(root, 2000)) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
      const names = new Set(Object.keys(deps ?? {}));
      for (const name of names) packages.add(name);
      Object.assign(scripts, pkg.scripts ?? {});
      Object.assign(versions, deps ?? {});
      packageFiles.push({
        file: path.relative(root, file),
        packages: names,
        versions: deps ?? {},
        scripts: pkg.scripts ?? {},
      });
    } catch {
      continue;
    }
  }
  return { exists: packageFiles.length > 0, packages, scripts, versions, packageFiles };
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
    gsapVersion: pkg.versions?.gsap ?? null,
    gsapTrialPresent: pkg.packages.has('gsap-trial'),
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
- gsap version: ${result.gsapVersion ?? '(not declared)'}
- gsap-trial present: ${result.gsapTrialPresent ? 'yes' : 'no'}
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
