#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const profile = {
  "skillName": "native-controls-transitions",
  "rules": [
    {
      "id": "native.package-needs-doctor",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "kind": "packageHasAny",
      "packages": [
        "expo-router",
        "@expo/ui",
        "react-native-screens",
        "@react-navigation/native-stack",
        "@gorhom/bottom-sheet",
        "@react-native-segmented-control/segmented-control",
        "@react-native-community/slider",
        "@react-native-picker/picker",
        "react-native-reanimated",
        "react-native-gesture-handler"
      ],
      "message": "Native navigation, control, sheet, or motion dependencies are present and should be validated with platform-specific checks.",
      "recommendation": "Run the repo doctor/typecheck/native smoke commands and record iOS/Android/web proof appropriate to the touched surface."
    },
    {
      "id": "native.stale-animation-enabled",
      "severity": "medium",
      "confidence": "high",
      "category": "api",
      "kind": "pattern",
      "pattern": "\\banimationEnabled\\s*:",
      "message": "A stale native-stack animationEnabled option was found.",
      "recommendation": "Use native-stack animation options such as animation: 'none' when installed types support them, and verify the route on target platforms."
    },
    {
      "id": "native.bottom-toolbar-in-layout",
      "severity": "medium",
      "confidence": "medium",
      "category": "architecture",
      "kind": "filePathContains",
      "pathPattern": "(^|/)_layout\\.(tsx|jsx)$",
      "include": "<Stack\\.Toolbar\\b(?![^>]*placement=[\"'](?:left|right)[\"'])",
      "message": "A Stack.Toolbar without explicit left/right placement was found in a layout file.",
      "recommendation": "Move bottom/default toolbar placement into the page component, or set and verify a supported header placement."
    },
    {
      "id": "native.toolbar-icon-needs-label",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "<Stack\\.Toolbar\\.(Button|Menu)\\b[^>]*(icon|sf|src)=",
      "without": "accessibilityLabel|aria-label|accessibilityHint",
      "message": "A native toolbar icon action was found without an obvious accessible label in the same file.",
      "recommendation": "Add a platform-supported accessibility label/hint for icon-only toolbar actions or document why visible text already labels the action."
    },
    {
      "id": "native.menu-action-title-prop",
      "severity": "medium",
      "confidence": "high",
      "category": "api",
      "kind": "pattern",
      "pattern": "<Link\\.MenuAction\\b[^>]*\\btitle\\s*=",
      "message": "Link.MenuAction was found with a title prop.",
      "recommendation": "Put the menu action label in children and keep platform labels/accessibility explicit."
    },
    {
      "id": "native.stack-transition-needs-owner",
      "severity": "low",
      "confidence": "medium",
      "category": "architecture",
      "kind": "fileContainsBoth",
      "include": "\\b(animation|presentation|sheetAllowedDetents|headerSearchBarOptions|Stack\\.SearchBar|Stack\\.Toolbar)\\b",
      "also": "\\b(withTiming|withSpring|entering=|exiting=|layout=|useAnimatedStyle)\\b",
      "message": "Navigator/native-control transition configuration and component Reanimated motion appear in the same file.",
      "recommendation": "Confirm there is one transition owner per element: navigator/native control for screen chrome, Reanimated for in-screen product content."
    },
    {
      "id": "native.expo-ui-host-scroll-risk",
      "severity": "medium",
      "confidence": "medium",
      "category": "layout",
      "kind": "fileContainsBoth",
      "include": "<Host\\b[^>]*\\bmatchContents\\b",
      "also": "<(ScrollView|FlatList|SectionList|FlashList)\\b",
      "message": "An Expo UI Host with matchContents appears in the same file as scrollable content.",
      "recommendation": "Verify matchContents is not used on the same axis as the scroll container; give scrollable hosts finite size."
    },
    {
      "id": "native.expo-ui-universal-needs-host",
      "severity": "low",
      "confidence": "medium",
      "category": "architecture",
      "kind": "fileContainsWithout",
      "include": "from\\s+['\"]@expo/ui['\"]",
      "without": "\\bHost\\b|from\\s+['\"]@expo/ui/(drop-in-replacements|swift-ui|jetpack-compose)",
      "message": "Universal @expo/ui components were imported without an obvious Host in the same file.",
      "recommendation": "Wrap universal Expo UI subtrees in Host or confirm the imported component does not require one in the installed package."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\b(withRepeat\\(|withTiming\\(|withSpring\\(|entering=|exiting=|layout=|itemLayoutAnimation|Layout\\.)\\b",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|AccessibilityInfo|reduceMotion",
      "message": "App-owned Reanimated motion was found without an obvious reduced-motion branch in the same file.",
      "recommendation": "Add reduced-motion behavior for app-owned movement or document why the effect is native/platform-owned and already handled elsewhere."
    }
  ]
};
const nativePackageNames = Array.from(new Set(profile.rules.flatMap((rule) => rule.packages ?? [])));

const skipDirs = new Set([
  '.git', 'node_modules', '.next', '.nuxt', 'dist', 'build', 'coverage',
  '.expo', '.turbo', '.vercel', '.cache', '.codex', '.agents',
  'output', 'tmp', 'temp', 'vendor', 'playwright-report', 'storybook-static',
]);
const selfFile = fileURLToPath(import.meta.url);
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

function requireValue(rest, flag) {
  const value = rest.shift();
  if (!value || value.startsWith("-")) throw new Error(`${flag} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = { command: null, root: process.cwd(), format: 'markdown', output: null, maxFiles: 2000 };
  const rest = [...argv];
  while (rest.length) {
    const arg = rest.shift();
    if (arg === '--help' || arg === '-h') args.help = true;
    else if (arg === '--json') args.format = 'json';
    else if (arg === '--root') args.root = path.resolve(requireValue(rest, arg));
    else if (arg === '--format') args.format = requireValue(rest, arg);
    else if (arg === '--output') args.output = path.resolve(requireValue(rest, arg));
    else if (arg === '--max-files') args.maxFiles = Number(requireValue(rest, arg));
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

function uniqueFindings(findings) {
  return [...new Map(findings.map((finding) => [finding.id, finding])).values()];
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

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'filePathContains') {
    if (!ruleRegex(rule.pathPattern).test(relativePath)) return findings;
    const includeMatch = ruleRegex(rule.include).exec(text);
    if (includeMatch) {
      const line = lineForIndex(text, includeMatch.index);
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
      for (const packageInfo of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageInfo.packages.has(name));
        if (matched.length === 0) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        const versions = matched.map((name) => `${name}@${packageInfo.packageVersions[name] ?? 'unknown'}`);
        findings.push({
          id: `${rule.id}:${packageInfo.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageInfo.file,
          line: 1,
          excerpt: `matched packages: ${versions.join(', ')}`,
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
      if (path.resolve(file) === selfFile) {
        continue;
      }
      findings.push(...scanRule(rule, file, root, text, config));
    }
  }
  const unique = uniqueFindings(findings);
  return {
    ok: !unique.some((finding) => finding.severity === 'high'),
    profile: profile.skillName,
    root,
    scannedFiles: files.length,
    scannedPackageFiles: pkg.packageFiles.length,
    rules: profile.rules.length,
    findings: unique,
    summary: severities.reduce((acc, severity) => {
      acc[severity] = unique.filter((finding) => finding.severity === severity).length;
      return acc;
    }, {}),
  };
}

function doctor(root, maxFiles) {
  const pkg = readPackage(root);
  const nativePackageMatches = nativePackageNames
    .flatMap((name) =>
      pkg.packageFiles
        .filter((packageInfo) => packageInfo.packages.has(name))
        .map((packageInfo) => ({ name, version: packageInfo.packageVersions[name] ?? 'unknown', file: packageInfo.file })),
    );
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    packageFiles: pkg.packageFiles.length,
    nativePackageMatches,
    configuredRules: profile.rules.length,
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'use --format json for machine-readable output',
      'package matches are signals only; verify installed types before API-sensitive edits',
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
- Native package matches: ${(result.nativePackageMatches ?? []).map((item) => `${item.name}@${item.version}`).join(', ') || 'none'}
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
