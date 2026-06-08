#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "gsap-scrolltrigger",
  "rules": [
    {
      "id": "gsap.private-registry",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "pattern": [
        "npm\\.green" + "sock\\.com",
        "GREEN.?SO" + "CK.*TOKEN",
        "green" + "sock.*auth" + "Token",
        "gsap.*lic" + "ense.?key"
      ].join("|"),
      "message": "Outdated private GSAP registry/token guidance or config is present.",
      "recommendation": "Use the public gsap npm package; modern GSAP plugins do not require a private registry token."
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
      "id": "gsap.scrolltrigger-markers-production",
      "severity": "medium",
      "confidence": "medium",
      "category": "debug",
      "pattern": "markers\\s*:\\s*true",
      "message": "ScrollTrigger debug markers are enabled.",
      "recommendation": "Use markers only while developing and remove or gate them before production."
    },
    {
      "id": "gsap.scrolltrigger-scrub-toggleactions",
      "severity": "medium",
      "confidence": "medium",
      "category": "behavior",
      "kind": "fileContainsBoth",
      "include": "scrub\\s*:",
      "also": "toggleActions\\s*:",
      "message": "A file configures both scrub and toggleActions around ScrollTrigger usage.",
      "recommendation": "Choose scrub for scroll-linked progress or toggleActions for discrete play/reverse behavior; do not mix them on the same trigger."
    },
    {
      "id": "gsap.scrolltrigger-container-ease",
      "severity": "medium",
      "confidence": "medium",
      "category": "behavior",
      "kind": "fileContainsBothWithout",
      "include": "containerAnimation\\s*:",
      "also": "scrollTrigger\\s*:",
      "without": "ease\\s*:\\s*['\"]none['\"]",
      "message": "containerAnimation is used without an obvious linear ease in the same file.",
      "recommendation": "Ensure the tween or timeline passed as containerAnimation uses ease: \"none\" so scroll position and animation progress stay aligned."
    },
    {
      "id": "gsap.scrolltrigger-deprecated-matchmedia",
      "severity": "low",
      "confidence": "high",
      "category": "api",
      "pattern": "ScrollTrigger\\.match" + "Media\\s*\\(",
      "message": "Deprecated ScrollTrigger responsive media-query helper usage is present.",
      "recommendation": "Use gsap.matchMedia() for new responsive ScrollTrigger setup; it tracks contexts and reverts animations and ScrollTriggers when queries stop matching."
    },
    {
      "id": "gsap.scrolltrigger-batch-animation-vars",
      "severity": "low",
      "confidence": "medium",
      "category": "api",
      "pattern": "ScrollTrigger\\.batch\\s*\\([\\s\\S]{0,900}\\b(trigger|scrub|snap|toggleActions|animation|onScrubComplete|onSnapComplete)\\s*:",
      "message": "ScrollTrigger.batch() appears to receive trigger or animation-linked vars.",
      "recommendation": "Batch targets are already the triggers; keep batch vars callback-oriented and avoid trigger, scrub, snap, toggleActions, animation, onScrubComplete, and onSnapComplete."
    },
    {
      "id": "gsap.scrolltrigger-child-tween",
      "severity": "medium",
      "confidence": "low",
      "category": "structure",
      "pattern": "gsap\\.timeline\\s*\\([^)]*\\)\\s*\\.[\\s\\S]{0,500}?\\b(to|from|fromTo)\\s*\\([^)]*scrollTrigger\\s*:",
      "message": "A timeline chain may put scrollTrigger on a child tween.",
      "recommendation": "Put ScrollTrigger on the parent timeline or a top-level tween, not on child tweens inside a timeline."
    },
    {
      "id": "gsap.scrolltrigger-refresh-hot-path",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "\\b(scroll|resize|pointermove|mousemove|touchmove|requestAnimationFrame|ticker\\.add|onscroll|onwheel)\\b[\\s\\S]{0,700}ScrollTrigger\\.refresh\\s*\\(",
      "message": "ScrollTrigger.refresh() appears near a high-frequency event or frame loop.",
      "recommendation": "Call refresh after real layout changes, not inside scroll/pointer/frame hot paths; debounce app-level refreshes."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "nearbyPattern",
      "pattern": "(?<![-\\w])(width|height|top|left|right|bottom|margin|padding)\\s*:",
      "context": "\\b(gsap\\.|scrollTrigger\\s*:|ScrollTrigger\\.|animate\\(|transition|@keyframes)\\b",
      "radius": 420,
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
const skipFiles = new Set(['scripts/audit.mjs']);
const fileExtensions = new Set([
  '.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.css', '.scss', '.sass',
  '.html', '.vue', '.svelte',
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
      } else if (entry.isFile() && !skipFiles.has(rel) && !shouldSkipFile(rel, entry.name, full) && (fileExtensions.has(path.extname(entry.name)) || fileNames.has(entry.name))) {
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
    return { exists: false, packages: new Set(), versions: {}, scripts: {}, installed: {} };
  }
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return {
      exists: true,
      packages: new Set(Object.keys(deps ?? {})),
      versions: deps ?? {},
      scripts: pkg.scripts ?? {},
      installed: {
        gsap: readInstalledPackageVersion(root, 'gsap'),
        '@gsap/react': readInstalledPackageVersion(root, '@gsap/react'),
      },
    };
  } catch {
    return { exists: true, packages: new Set(), versions: {}, scripts: {}, installed: {} };
  }
}

function readInstalledPackageVersion(root, packageName) {
  const file = path.join(root, 'node_modules', ...packageName.split('/'), 'package.json');
  if (!fs.existsSync(file)) return null;
  try {
    return JSON.parse(fs.readFileSync(file, 'utf8')).version ?? null;
  } catch {
    return null;
  }
}

function packageSummary(pkg) {
  return {
    packageJson: pkg.exists,
    gsapDependency: pkg.versions.gsap ?? null,
    gsapInstalledVersion: pkg.installed.gsap ?? null,
    gsapReactDependency: pkg.versions['@gsap/react'] ?? null,
    gsapReactInstalledVersion: pkg.installed['@gsap/react'] ?? null,
  };
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
  if (rule.kind === 'nearbyPattern') {
    const regex = ruleRegex(rule.pattern);
    const contextRegex = ruleRegex(rule.context);
    const radius = Number.isFinite(rule.radius) ? rule.radius : 360;
    for (const match of text.matchAll(regex)) {
      const index = match.index ?? 0;
      const nearby = text.slice(Math.max(0, index - radius), Math.min(text.length, index + radius));
      contextRegex.lastIndex = 0;
      if (!contextRegex.test(nearby)) continue;
      const line = lineForIndex(text, index);
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
    package: packageSummary(pkg),
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
    package: packageSummary(pkg),
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
- Package JSON: ${result.package.packageJson ? 'yes' : 'no'}
- GSAP dependency: ${result.package.gsapDependency ?? '(not declared)'}
- GSAP installed: ${result.package.gsapInstalledVersion ?? '(not installed)'}
- @gsap/react dependency: ${result.package.gsapReactDependency ?? '(not declared)'}
- @gsap/react installed: ${result.package.gsapReactInstalledVersion ?? '(not installed)'}
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
- GSAP dependency: ${result.package.gsapDependency ?? '(not declared)'}
- GSAP installed: ${result.package.gsapInstalledVersion ?? '(not installed)'}
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
