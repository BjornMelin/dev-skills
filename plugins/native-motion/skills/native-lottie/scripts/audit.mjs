#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "native-lottie",
  "rules": [
    {
      "id": "lottie.remote-asset",
      "severity": "medium",
      "confidence": "medium",
      "category": "asset",
      "pattern": "(?:path|src|source|uri|url)\\s*[:=]\\s*{?\\s*['\"]https?://[^'\"]*(?:lottie|\\.json|\\.lottie)",
      "message": "A Lottie asset appears to load from a remote URL.",
      "recommendation": "Prefer local versioned assets or document loading, cache, offline, privacy, and failure behavior for the remote dependency."
    },
    {
      "id": "lottie.loop-autoplay-no-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsBothWithout",
      "include": "LottieView|DotLottie|lottie-react-native|dotlottie-react-native",
      "also": "\\b(?:loop|autoPlay|autoplay)\\s*(?:=|:)\\s*{?\\s*true\\b|<[^>]*\\s(?:loop|autoPlay|autoplay)(?!\\s*=)(?=\\s|>|/)",
      "without": "reduced|ReduceMotion|useReducedMotion|motion-reduce|AccessibilityInfo|poster|fallback|finalFrame|static",
      "message": "Looping/autoplay Lottie usage lacks an obvious reduced-motion or poster fallback.",
      "recommendation": "Pause, replace, or simplify decorative asset playback under reduced motion."
    },
    {
      "id": "lottie.imperative-no-unmount-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "LottieView|DotLottie|lottie-react-native|dotlottie-react-native",
      "also": "\\.current\\?\\.(?:play|resume)\\(",
      "without": "return\\s*\\(?.*(?:reset|pause|stop|freeze|animation\\.stop|subscription\\.remove)|\\.current\\?\\.(?:reset|pause|stop|freeze)\\(",
      "message": "Imperative Lottie playback lacks obvious unmount/interruption cleanup.",
      "recommendation": "Verify the owning component resets, pauses, stops, freezes, or stops Animated progress when it unmounts or is interrupted."
    },
    {
      "id": "lottie.progress-mixed-control",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsBoth",
      "include": "progress\\s*=",
      "also": "\\.current\\?\\.(?:play|pause|reset|resume)\\(",
      "message": "A Lottie component appears to mix controlled progress and imperative playback.",
      "recommendation": "Use one playback owner per component: progress, imperative ref, marker/segment, state machine, or declarative autoplay."
    },
    {
      "id": "lottie.progress-native-driver",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "pattern": "progress[\\s\\S]{0,2000}Animated\\.timing[\\s\\S]{0,800}useNativeDriver\\s*:\\s*true|Animated\\.timing[\\s\\S]{0,800}useNativeDriver\\s*:\\s*true[\\s\\S]{0,2000}progress",
      "message": "Lottie progress appears to be driven by React Native Animated with useNativeDriver: true.",
      "recommendation": "Use useNativeDriver: false for Lottie progress ownership unless the repo has a verified alternative."
    },
    {
      "id": "lottie.colorfilters-contract",
      "severity": "low",
      "confidence": "medium",
      "category": "asset",
      "kind": "fileContainsBoth",
      "include": "LottieView|lottie-react-native",
      "also": "colorFilters|textFiltersAndroid|textFiltersIOS",
      "message": "Lottie color or text filters depend on designer-authored asset keypaths.",
      "recommendation": "Centralize keypaths/text targets and verify iOS/Android rendering after every designer export."
    },
    {
      "id": "lottie.dotlottie-needs-metro",
      "severity": "high",
      "confidence": "high",
      "category": "asset",
      "kind": "dotLottieNeedsMetro",
      "message": ".lottie assets are used without an obvious Metro assetExts entry.",
      "recommendation": "Add 'lottie' to Metro resolver.assetExts before importing .lottie files."
    },
    {
      "id": "lottie.dotlottie-needs-test-mock",
      "severity": "low",
      "confidence": "medium",
      "category": "test",
      "kind": "dotLottieNeedsTestMock",
      "message": ".lottie assets are used in a repo with JS tests but no obvious test module mapper or mock.",
      "recommendation": "Add a stable .lottie test stub for Jest/Vitest if tests import animation modules."
    },
    {
      "id": "dotlottie.state-machine-contract",
      "severity": "low",
      "confidence": "medium",
      "category": "asset",
      "kind": "fileContainsBoth",
      "include": "DotLottie|dotlottie-react-native",
      "also": "stateMachineId|stateMachineStart|stateMachineLoad|stateMachineSet|stateMachineFire",
      "message": "dotLottie state-machine usage depends on asset-authored IDs and inputs.",
      "recommendation": "Centralize state-machine IDs/input names and validate each transition/input on iOS and Android."
    },
    {
      "id": "dotlottie.native-runtime-needs-build",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "kind": "packageHasAny",
      "packages": [
        "@lottiefiles/dotlottie-react-native"
      ],
      "message": "dotLottie React Native is installed and includes native runtime code.",
      "recommendation": "Use prebuild/development/EAS builds for proof; Expo Go is not sufficient for dotLottie native validation."
    },
    {
      "id": "lottie.package-needs-validation",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "kind": "packageHasAny",
      "packages": [
        "lottie-react-native",
        "@lottiefiles/dotlottie-react-native"
      ],
      "message": "Native Lottie dependencies are present and should be validated with focused package and platform checks.",
      "recommendation": "Run the repo's package alignment, typecheck/test, and iOS/Android playback checks; include rebuild proof after native or asset changes."
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

function listAllFiles(root, maxFiles) {
  const files = [];
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel)) walk(full);
      } else if (entry.isFile()) {
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
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), packageVersions: {}, packageFiles: [], scripts: {} };
  const files = listPackageFiles(root);
  const packages = new Set();
  const packageVersions = {};
  const packageFiles = [];
  const scripts = {};
  for (const file of files) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      Object.assign(scripts, pkg.scripts ?? {});
      const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
      const packageNames = new Set(Object.keys(deps ?? {}));
      Object.assign(packageVersions, deps ?? {});
      for (const name of packageNames) packages.add(name);
      packageFiles.push({ file: path.relative(root, file), packages: packageNames, packageVersions: deps ?? {} });
    } catch {
      continue;
    }
  }
  return { exists: files.length > 0, packages, packageVersions, packageFiles, scripts };
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
  return new RegExp(pattern, 'gims');
}

function dotLottieEvidence(root, textFiles) {
  const evidence = [];
  for (const file of textFiles) {
    let text;
    try {
      text = fs.readFileSync(file, 'utf8');
    } catch {
      continue;
    }
    for (const match of text.matchAll(/(?:require\(\s*['"`][^'"`\n]+\.lottie['"`]\s*\)|import\s+(?:[^'"`;]+?\s+from\s+)?['"`][^'"`\n]+\.lottie['"`])/g)) {
      const line = lineForIndex(text, match.index ?? 0);
      evidence.push({
        file: path.relative(root, file),
        line,
        excerpt: excerptForLine(text.split('\n'), line),
      });
    }
  }
  return evidence;
}

function hasMetroDotLottieSupport(allFiles) {
  const names = [
    'metro.config.js',
    'metro.config.cjs',
    'metro.config.mjs',
    'metro.config.ts',
  ];
  return allFiles.some((file) => {
    if (!names.includes(path.basename(file))) return false;
    const text = fs.readFileSync(file, 'utf8');
    return assetExtsContains(text, 'lottie');
  });
}

function assetExtsContains(text, extension) {
  const escaped = extension.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`assetExts\\.(?:push|unshift)\\(\\s*['"\`]${escaped}['"\`]`, 'i').test(text)
    || new RegExp(`assetExts\\s*[:=]\\s*\\[[^\\]]*['"\`]${escaped}['"\`][^\\]]*\\]`, 'i').test(text);
}

function hasJsTestSurface(pkg, allFiles) {
  const scriptText = Object.entries(pkg.scripts ?? {})
    .map(([name, value]) => `${name}:${value}`)
    .join('\n');
  if (/\b(jest|vitest)\b/.test(scriptText)) return true;
  return allFiles.some((file) => {
    const name = path.basename(file);
    return (
      /^jest\.config\.[cm]?[jt]s$/.test(name) ||
      /^vitest\.config\.[cm]?[jt]s$/.test(name)
    );
  });
}

function hasDotLottieTestMock(root, allFiles) {
  const configNames = new Set([
    'jest.config.js',
    'jest.config.cjs',
    'jest.config.mjs',
    'jest.config.ts',
    'vitest.config.js',
    'vitest.config.cjs',
    'vitest.config.mjs',
    'vitest.config.ts',
  ]);
  for (const file of allFiles) {
    const name = path.basename(file);
    if (!configNames.has(name) && !file.includes(`${path.sep}__mocks__${path.sep}`)) {
      continue;
    }
    let text;
    try {
      text = fs.readFileSync(file, 'utf8');
    } catch {
      continue;
    }
    if (/\\\.\(lottie\)|\\\.lottie|\.lottie|dotLottieMock|lottieMock|lottie-test-file-stub/i.test(text)) {
      return true;
    }
  }
  return false;
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
  const allFiles = listAllFiles(root, maxFiles);
  const pkg = readPackage(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageHasAny') {
      for (const packageFile of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageFile.packages.has(name));
        if (matched.length === 0) continue;
        if (isIgnored(config, rule.id, packageFile.file, [''], 1)) continue;
        const matchedWithVersions = matched.map((name) => {
          const version = packageFile.packageVersions?.[name];
          return version ? `${name}@${version}` : name;
        });
        findings.push({
          id: `${rule.id}:${packageFile.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageFile.file,
          line: 1,
          excerpt: `matched packages: ${matchedWithVersions.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'dotLottieNeedsMetro') {
      const evidence = dotLottieEvidence(root, files)
        .find((entry) => !isIgnored(config, rule.id, entry.file, [''], entry.line));
      if (evidence && !hasMetroDotLottieSupport(allFiles)) {
        findings.push(makeFinding(rule, evidence.file, evidence.line, evidence.excerpt));
      }
      continue;
    }
    if (rule.kind === 'dotLottieNeedsTestMock') {
      const evidence = dotLottieEvidence(root, files)
        .find((entry) => !isIgnored(config, rule.id, entry.file, [''], entry.line));
      if (
        evidence &&
        hasJsTestSurface(pkg, allFiles) &&
        !hasDotLottieTestMock(root, allFiles)
      ) {
        findings.push(makeFinding(rule, evidence.file, evidence.line, evidence.excerpt));
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
  const lottiePackages = [
    'lottie-react-native',
    '@lottiefiles/dotlottie-react-native',
  ]
    .filter((name) => pkg.packages.has(name))
    .map((name) => `${name}@${pkg.packageVersions?.[name] ?? 'unknown'}`);
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    lottiePackages,
    configuredRules: profile.rules.length,
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'Lottie findings are triage; verify against real code and asset contracts',
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
- Lottie packages: ${result.lottiePackages.length ? result.lottiePackages.join(', ') : 'none detected'}
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
