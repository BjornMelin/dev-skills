#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "web-motion-react",
  "rules": [
    {
      "id": "motion.package-missing",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageMissingWhenFileMatches",
      "include": "from\\s+['\"]motion\\/(react|react-client|react-m|react-mini)['\"]|require\\(['\"]motion\\/(react|react-client|react-m|react-mini)['\"]\\)",
      "packages": ["motion"],
      "message": "Source imports Motion React from the motion package, but package.json does not declare motion.",
      "recommendation": "Add motion with the target repo's package manager, or remove the stale import."
    },
    {
      "id": "motion.framer-motion-package",
      "severity": "medium",
      "confidence": "medium",
      "category": "dependency",
      "kind": "packageHasAny",
      "packages": ["framer-motion"],
      "message": "The target repo still declares framer-motion directly.",
      "recommendation": "For new Motion React work, prefer the motion package. Treat direct framer-motion as intentional legacy unless the task includes migration."
    },
    {
      "id": "motion.animatepresence-keys",
      "severity": "medium",
      "confidence": "low",
      "category": "correctness",
      "kind": "fileContainsBoth",
      "include": "AnimatePresence",
      "also": "\\? .*<motion\\.|&&\\s*<motion\\.",
      "message": "AnimatePresence is used with conditional motion children; stable keys may be required for exits.",
      "recommendation": "Verify leaving children have stable keys and AnimatePresence itself stays mounted."
    },
    {
      "id": "motion.animatepresence-index-key",
      "severity": "medium",
      "confidence": "high",
      "category": "correctness",
      "kind": "fileContainsBoth",
      "include": "AnimatePresence",
      "also": "key=\\{\\s*(index|i)\\s*\\}",
      "message": "AnimatePresence appears to use an array index as a direct or nearby key.",
      "recommendation": "Use a stable item ID so exits and reorders preserve presence identity."
    },
    {
      "id": "motion.animatepresence-conditionally-mounted",
      "severity": "medium",
      "confidence": "low",
      "category": "correctness",
      "pattern": "(\\?\\s*<AnimatePresence\\b|&&\\s*<AnimatePresence\\b)",
      "message": "AnimatePresence itself may be conditionally mounted, which prevents exit animations from controlling removed children.",
      "recommendation": "Keep AnimatePresence mounted and put the conditional child inside it."
    },
    {
      "id": "motion.animatepresence-poplayout-custom-ref",
      "severity": "medium",
      "confidence": "low",
      "category": "correctness",
      "kind": "fileContainsBothWithout",
      "include": "mode\\s*=\\s*['\"]popLayout['\"]",
      "also": "<[A-Z][A-Za-z0-9]*(\\s|>)",
      "without": "forwardRef\\s*\\(",
      "message": "AnimatePresence popLayout is used near custom components without an obvious forwardRef.",
      "recommendation": "When a custom component is an immediate popLayout child, wrap it in forwardRef and forward the ref to the DOM node."
    },
    {
      "id": "motion.legacy-framer-motion-import",
      "severity": "medium",
      "confidence": "high",
      "category": "maintenance",
      "pattern": "from\\s+['\"]framer-motion['\"]|require\\(['\"]framer-motion['\"]\\)",
      "message": "The legacy framer-motion package import was found in Motion React code.",
      "recommendation": "Prefer the current motion package import path, usually motion/react, unless the target repo intentionally remains on framer-motion."
    },
    {
      "id": "motion.lazymotion-mixed-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "LazyMotion",
      "also": "(<motion\\.|\\bimport\\s+\\{[^}]*\\bmotion\\b[^}]*\\}\\s+from\\s+['\"]motion/react['\"])",
      "message": "LazyMotion is present while the regular motion component may also be loaded.",
      "recommendation": "Use m components from motion/react-m inside LazyMotion subtrees; consider LazyMotion strict when bundle size is the goal."
    },
    {
      "id": "motion.lazymotion-missing-features",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsWithout",
      "include": "<LazyMotion\\b",
      "without": "\\bfeatures\\s*=",
      "message": "LazyMotion is rendered without an obvious features bundle.",
      "recommendation": "Pass domAnimation, domMax, or an async feature loader so the lazy boundary is explicit."
    },
    {
      "id": "motion.next-app-import-client-boundary",
      "severity": "medium",
      "confidence": "medium",
      "category": "ssr",
      "kind": "fileContainsWithout",
      "include": "from\\s+['\"]motion/react['\"]",
      "without": "['\"]use client['\"]|from\\s+['\"]motion/react-client['\"]",
      "pathPattern": "(^|[/\\\\])app[/\\\\].*\\.(js|jsx|ts|tsx)$",
      "message": "A Next-style app route file imports motion/react without an obvious client boundary.",
      "recommendation": "Move Motion hooks/components into a small client component, add a use client directive, or use motion/react-client for server-compatible animated DOM wrappers."
    },
    {
      "id": "motion.react-client-hooks",
      "severity": "high",
      "confidence": "high",
      "category": "ssr",
      "kind": "fileContainsBoth",
      "include": "from\\s+['\"]motion/react-client['\"]",
      "also": "\\b(useScroll|useAnimate|useReducedMotion|useInView|useMotionValueEvent|useSpring|useTransform|useAnimationFrame|useDragControls|useMotionValue|usePresence|usePresenceData|useIsPresent)\\b",
      "message": "motion/react-client is imported in a file that also references Motion hooks.",
      "recommendation": "Use motion/react inside a client component for hooks, refs, scroll, presence state, gestures, or layout measurement."
    },
    {
      "id": "motion.hooks-missing-client-boundary",
      "severity": "medium",
      "confidence": "medium",
      "category": "ssr",
      "kind": "fileContainsBothWithout",
      "pathPattern": "(^|[/\\\\])app[/\\\\].*\\.(js|jsx|ts|tsx)$",
      "include": "from\\s+['\"]motion/react['\"]",
      "also": "\\b(useScroll|useAnimate|useReducedMotion|useInView|useMotionValueEvent|useSpring|useTransform|useAnimationFrame|useDragControls|useMotionValue|usePresence|usePresenceData|useIsPresent)\\b",
      "without": "['\"]use client['\"]",
      "message": "A Next App Router file appears to use Motion hooks without a client-component directive.",
      "recommendation": "Move the hook code into a focused client component and keep the parent server component server-rendered."
    },
    {
      "id": "motion.broad-transition",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "\\btransition\\s*:\\s*['\"]?all\\b|\\btransition" + "-all\\b",
      "message": "Broad transition declarations can animate future expensive properties unintentionally.",
      "recommendation": "Transition only the properties that should move, usually transform and opacity."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "(from\\s+['\"]motion/react['\"]|from\\s+['\"]motion/react-client['\"]|<motion\\.|\\bmotion\\.)",
      "also": "\\b(width|height|top|left|right|bottom|margin|padding)\\s*:",
      "message": "Layout-affecting properties are being animated or configured near motion code.",
      "recommendation": "Prefer transform and opacity in hot paths; measure before keeping layout animation."
    },
    {
      "id": "motion.layout-scroll-container",
      "severity": "medium",
      "confidence": "low",
      "category": "correctness",
      "kind": "fileContainsBothWithout",
      "include": "\\blayout\\b",
      "also": "overflow(?:X|Y)?\\s*:\\s*['\"]?(auto|scroll)",
      "without": "\\blayoutScroll\\b",
      "message": "A layout animation appears inside or near a scrollable container without layoutScroll.",
      "recommendation": "Add layoutScroll to scrollable layout containers so Motion accounts for scroll offset during measurement."
    },
    {
      "id": "motion.layout-fixed-container",
      "severity": "medium",
      "confidence": "low",
      "category": "correctness",
      "kind": "fileContainsBothWithout",
      "include": "\\blayout\\b",
      "also": "position\\s*:\\s*['\"]?fixed",
      "without": "\\blayoutRoot\\b",
      "message": "A layout animation appears inside or near a fixed container without layoutRoot.",
      "recommendation": "Add layoutRoot to fixed layout containers so Motion accounts for page scroll offset."
    },
    {
      "id": "motion.scroll-react-state-write",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "\\buseScroll\\s*\\(",
      "also": "\\bset[A-Z][A-Za-z0-9_]*\\s*\\(",
      "message": "useScroll is used near React state writes.",
      "recommendation": "Avoid writing React state on every scroll frame; bind Motion values directly or compose them with useTransform/useSpring."
    },
    {
      "id": "motion.infinite-repeat-without-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "repeat\\s*:\\s*(Infinity|-1)|repeatType\\s*:",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|reduceMotion",
      "message": "A repeating Motion animation has no obvious reduced-motion branch in the same file.",
      "recommendation": "Disable or simplify nonessential loops for reduced-motion users."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "(from\\s+['\"]motion/react|from\\s+['\"]motion/react-client|<motion\\.|\\bmotion\\.)",
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
  Optional .web-motion-react-audit.json or .motion-audit.json at --root supports:
  {
    "ignoreRules": ["rule-id"],
    "ignorePaths": ["generated/", "fixtures/"],
    "ignores": [{"ruleId": "rule-id", "path": "src/example.tsx"}]
  }

Inline suppression:
  // web-motion-react-audit-ignore rule-id
  // web-motion-react-audit-ignore all
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
  const file = [
    path.join(root, '.web-motion-react-audit.json'),
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
    throw new Error(`Failed to parse ${path.basename(file)}: ${error.message}`);
  }
}

const skillArtifactDirs = new Set(['agents', 'assets', 'examples', 'references', 'scripts', 'templates']);

function shouldSkipDir(relativePath, rootIsSkillDir = false) {
  const normalized = relativePath.split(path.sep).join('/');
  if (rootIsSkillDir && skillArtifactDirs.has(normalized.split('/')[0])) return true;
  if (/^skills\/[^/]+\/(?:agents|assets|examples|references|scripts|templates)(?:\/|$)/.test(normalized)) return true;
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
  if (fullPath) {
    try {
      if (fs.readFileSync(fullPath, 'utf8').slice(0, 512).includes('motion-audit-skip-file')) return true;
    } catch {}
  }
  if (normalized === 'scripts/audit.mjs' || normalized.endsWith('/scripts/audit.mjs')) return true;
  if (/^skills\/[^/]+\/SKILL\.md$/.test(normalized)) return true;
  return path.extname(fileName) === '.json' && fileName !== 'package.json';
}

function listFiles(root, maxFiles) {
  const files = [];
  const rootIsSkillDir = fs.existsSync(path.join(root, 'SKILL.md')) && fs.existsSync(path.join(root, 'references'));
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel, rootIsSkillDir)) walk(full);
      } else if (entry.isFile() && !shouldSkipFile(rel, entry.name, full) && (fileExtensions.has(path.extname(entry.name)) || fileNames.has(entry.name))) {
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
  const rootIsSkillDir = fs.existsSync(path.join(root, 'SKILL.md')) && fs.existsSync(path.join(root, 'references'));
  function walk(dir) {
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel, rootIsSkillDir)) walk(full);
      } else if (entry.isFile() && entry.name === 'package.json') {
        files.push(full);
      }
    }
  }
  walk(root);
  return files;
}

function readPackageFile(file) {
  if (!fs.existsSync(file)) return { exists: false, packages: new Set() };
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return { exists: true, file, dir: path.dirname(file), packages: new Set(Object.keys(deps ?? {})), versions: deps ?? {}, scripts: pkg.scripts ?? {} };
  } catch {
    return { exists: true, file, dir: path.dirname(file), packages: new Set(), versions: {}, scripts: {} };
  }
}

function readPackage(root) {
  return readPackageFile(path.join(root, 'package.json'));
}

function packageRecords(root) {
  return listPackageFiles(root).map(readPackageFile);
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
  return nearby.includes('web-motion-react-audit-ignore all') ||
    nearby.includes(`web-motion-react-audit-ignore ${ruleId}`) ||
    nearby.includes('motion-audit-ignore all') ||
    nearby.includes(`motion-audit-ignore ${ruleId}`);
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

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  if (rule.pathPattern && !new RegExp(rule.pathPattern).test(relativePath)) {
    return [];
  }
  const lines = text.split('\n');
  const findings = [];
  const normalizedPath = `/${relativePath.split(path.sep).join('/')}`;
  if (rule.pathIncludes && !normalizedPath.includes(rule.pathIncludes)) return findings;
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
  if (rule.kind === 'packageHasAny' || rule.kind === 'packageMissingWhenFileMatches') return findings;
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
  const packages = packageRecords(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageHasAny') {
      for (const pkg of packages) {
        const matched = (rule.packages ?? []).filter((name) => pkg.packages.has(name));
        if (matched.length === 0) continue;
        const packagePath = path.relative(root, pkg.file);
        if (isIgnored(config, rule.id, packagePath, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packagePath}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packagePath,
          line: 1,
          excerpt: `matched packages: ${matched.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageMissingWhenFileMatches') {
      const matchesByPackage = new Map();
      for (const file of files) {
        const pkg = nearestPackageRecord(file, packages);
        if (!pkg?.exists) continue;
        const hasRequired = (rule.packages ?? []).some((name) => pkg.packages.has(name));
        if (hasRequired) continue;
        let text;
        try {
          text = fs.readFileSync(file, 'utf8');
        } catch {
          continue;
        }
        const match = ruleRegex(rule.include).exec(text);
        if (match) {
          const relativePath = path.relative(root, file);
          const lines = text.split('\n');
          const line = lineForIndex(text, match.index ?? 0);
          if (!isIgnored(config, rule.id, relativePath, lines, line)) {
            const packagePath = path.relative(root, pkg.file);
            if (!matchesByPackage.has(packagePath)) {
              matchesByPackage.set(packagePath, { file: relativePath, line, excerpt: excerptForLine(lines, line), packagePath });
            }
          }
        }
      }
      for (const match of matchesByPackage.values()) {
        findings.push({
          id: `${rule.id}:${match.file}:${match.line}`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: match.file,
          line: match.line,
          excerpt: match.excerpt,
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
