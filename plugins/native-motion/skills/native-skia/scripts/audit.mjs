#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "native-skia",
  "rules": [
    {
      "id": "skia.canvas-size",
      "severity": "medium",
      "confidence": "low",
      "category": "rendering",
      // motion-audit-ignore skia.canvas-size
      "pattern": "<Canvas(?![^>]*(style|height|width|flex|onSize))",
      "message": "Skia Canvas appears without obvious sizing.",
      "recommendation": "Give canvases stable dimensions and test blank-canvas failure modes."
    },
    {
      "id": "skia.canvas-onlayout",
      "severity": "low",
      "confidence": "low",
      "category": "rendering",
      "kind": "fileContainsBoth",
      "include": "<Canvas",
      "also": "onLayout\\s*=",
      "message": "Skia Canvas uses onLayout for measurement.",
      "recommendation": "Prefer onSize for UI-thread dimensions or useCanvasSize() for JS-thread dimensions; verify Fabric compatibility."
    },
    {
      "id": "skia.reanimated-interpolate-color",
      "severity": "medium",
      "confidence": "medium",
      "category": "rendering",
      "kind": "fileContainsBothWithout",
      "include": "@shopify/react-native-skia",
      "also": "interpolateColor\\s*\\(",
      "without": "interpolateColors\\s*\\(",
      "message": "Skia code appears to use Reanimated interpolateColor.",
      "recommendation": "Use interpolateColors from @shopify/react-native-skia for Skia color values."
    },
    {
      "id": "skia.animated-component-wrapper",
      "severity": "medium",
      "confidence": "medium",
      "category": "rendering",
      "kind": "fileContainsBoth",
      "include": "@shopify/react-native-skia",
      "also": "createAnimatedComponent\\s*\\(\\s*Canvas\\s*\\)|useAnimatedProps\\s*\\(",
      "message": "Skia components appear to be wrapped with generic Reanimated animated props.",
      "recommendation": "Pass Reanimated shared and derived values directly to Skia props unless the installed API specifically requires an animated wrapper."
    },
    {
      "id": "skia.runtime-effect-null",
      "severity": "medium",
      "confidence": "low",
      "category": "rendering",
      "kind": "fileContainsWithout",
      "include": "Skia\\.RuntimeEffect\\.Make\\(",
      "without": "RuntimeEffect\\.Make\\([\\s\\S]{0,500}(if\\s*\\(|throw new Error|return null|invariant|assert)",
      "message": "Runtime shader compilation may not handle a null RuntimeEffect.",
      "recommendation": "Check RuntimeEffect.Make() for null and render a fallback or fail with a clear error."
    },
    {
      "id": "skia.runtime-effect-non-null-assertion",
      "severity": "medium",
      "confidence": "medium",
      "category": "rendering",
      "pattern": "Skia\\.RuntimeEffect\\.Make\\([\\s\\S]{0,500}\\)!",
      "message": "RuntimeEffect.Make() result is force-unwrapped.",
      "recommendation": "Replace the non-null assertion with an explicit null check and a fallback or clear compile error."
    },
    {
      "id": "skia.image-loading-state",
      "severity": "medium",
      "confidence": "low",
      "category": "assets",
      "kind": "fileContainsBothWithout",
      "include": "useImage\\s*\\(",
      "also": "<Image\\b",
      "without": "if\\s*\\([^\\)]*image|image\\s*\\?|image\\s*={\\s*[^}]+\\?\\s*|image\\s*===?\\s*null|return\\s+null|fallback|placeholder|onError|isLoaded",
      "message": "Skia image code may render before useImage() finishes loading.",
      "recommendation": "Render a deliberate loading/error/fallback state before passing image data to Skia drawing components."
    },
    {
      "id": "skia.expo-56-bundled-version",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "kind": "expoBundledSkiaMismatch",
      "expoMajor": 56,
      "package": "@shopify/react-native-skia",
      "expected": "2.6.2",
      "message": "Expo SDK 56 project appears to use a Skia version different from Expo's bundled native module.",
      "recommendation": "Use Expo-compatible installation or prove a custom dev/native build before moving past @shopify/react-native-skia@2.6.2."
    },
    {
      "id": "skia.bun-trusted-dependency",
      "severity": "low",
      "confidence": "medium",
      "category": "validation",
      "kind": "packageMissingTrustedDependency",
      "package": "@shopify/react-native-skia",
      "message": "Skia is installed in a likely Bun project without an obvious trustedDependencies entry.",
      "recommendation": "If the repo uses Bun, add @shopify/react-native-skia to trustedDependencies so native binaries can be copied during postinstall."
    },
    {
      "id": "native.package-needs-doctor",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "kind": "packageHasAny",
      "packages": [
        "react-native-reanimated",
        "nativewind",
        "@shopify/react-native-skia",
        "@rive-app/react-native",
        "lottie-react-native",
        "expo-gl"
      ],
      "message": "Native motion dependencies are present and should be validated with platform-specific checks.",
      "recommendation": "Run the repo doctor/typecheck/native smoke commands and record iOS/Android proof."
    },
    {
      "id": "native.shared-value-js-read",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "requires": ["@shopify/react-native-skia"],
      "pattern": "[^A-Za-z0-9_]([A-Za-z0-9_]+)\\.value",
      "message": "Shared value reads on the JS thread can block or desynchronize if used outside worklets.",
      "recommendation": "Confirm this read is inside a worklet; otherwise derive and consume on the UI thread."
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
  '.html', '.vue', '.svelte',
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
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), dependencyVersions: {}, packageFiles: [], scripts: {} };
  const files = listPackageFiles(root);
  const packages = new Set();
  const dependencyVersions = {};
  const packageFiles = [];
  const scripts = {};
  const rootBunLock = fs.existsSync(path.join(root, 'bun.lock')) || fs.existsSync(path.join(root, 'bun.lockb'));
  for (const file of files) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      Object.assign(scripts, pkg.scripts ?? {});
      const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
      const packageNames = new Set(Object.keys(deps ?? {}));
      Object.assign(dependencyVersions, deps ?? {});
      for (const name of packageNames) packages.add(name);
      packageFiles.push({
        file: path.relative(root, file),
        packages: packageNames,
        dependencyVersions: deps ?? {},
        trustedDependencies: Array.isArray(pkg.trustedDependencies) ? new Set(pkg.trustedDependencies) : new Set(),
        packageManager: typeof pkg.packageManager === 'string' ? pkg.packageManager : '',
        bunLock: rootBunLock || fs.existsSync(path.join(path.dirname(file), 'bun.lock')) || fs.existsSync(path.join(path.dirname(file), 'bun.lockb')),
      });
    } catch {
      continue;
    }
  }
  return { exists: files.length > 0, packages, dependencyVersions, packageFiles, scripts };
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

const workletHookArgumentCounts = new Map([
  ['useAnimatedStyle', 1],
  ['useDerivedValue', 1],
  ['useAnimatedReaction', 2],
  ['useAnimatedProps', 1],
  ['useAnimatedScrollHandler', 1],
  ['useFrameCallback', 1],
  ['useAnimatedGestureHandler', 1],
]);

const gestureWorkletCallbackPattern =
  /\.(?:onBegin|onStart|onUpdate|onChange|onEnd|onFinalize|onTouchesDown|onTouchesMove|onTouchesUp|onTouchesCancelled)\s*\(/g;

function topLevelArguments(text, openParenIndex) {
  const args = [];
  let start = openParenIndex + 1;
  let depth = 1;
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let index = openParenIndex + 1; index < text.length; index += 1) {
    const char = text[index];
    const next = text[index + 1];
    if (lineComment) {
      if (char === '\n') lineComment = false;
      continue;
    }
    if (blockComment) {
      if (char === '*' && next === '/') {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '/' && next === '/') {
      lineComment = true;
      index += 1;
      continue;
    }
    if (char === '/' && next === '*') {
      blockComment = true;
      index += 1;
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
        args.push({ start, end: index });
        break;
      }
      continue;
    }
    if (char === ',' && depth === 1) {
      args.push({ start, end: index });
      start = index + 1;
    }
  }
  return args;
}

function isSharedValueWorkletRead(text, index) {
  for (const [hookName, workletArgumentCount] of workletHookArgumentCounts) {
    const hookRegex = new RegExp(`\\b${hookName}\\s*\\(`, 'g');
    for (const match of text.matchAll(hookRegex)) {
      const openParenIndex = match.index + match[0].lastIndexOf('(');
      if (openParenIndex > index) break;
      const args = topLevelArguments(text, openParenIndex).slice(0, workletArgumentCount);
      for (const arg of args) {
        if (index < arg.start || index >= arg.end) continue;
        const argumentText = text.slice(arg.start, arg.end);
        if (/(?:=>|\bfunction\b|['"]worklet['"])/.test(argumentText)) return true;
      }
    }
  }
  for (const match of text.matchAll(gestureWorkletCallbackPattern)) {
    const openParenIndex = match.index + match[0].lastIndexOf('(');
    if (openParenIndex > index) break;
    const [callbackArg] = topLevelArguments(text, openParenIndex);
    if (!callbackArg || index < callbackArg.start || index >= callbackArg.end) continue;
    const argumentText = text.slice(callbackArg.start, callbackArg.end);
    if (/(?:=>|\bfunction\b|['"]worklet['"])/.test(argumentText)) return true;
  }
  return false;
}

function isSharedValueWrite(text, match) {
  const fullMatch = match[0] ?? '';
  const matchIndex = match.index ?? 0;
  const valueOffset = fullMatch.lastIndexOf('.value');
  if (valueOffset < 0) return false;
  const identifierOffset = fullMatch.search(/[A-Za-z0-9_]+\.value/);
  const identifierIndex = matchIndex + (identifierOffset < 0 ? valueOffset : identifierOffset);
  const afterValue = text.slice(matchIndex + valueOffset + '.value'.length).trimStart();
  if (/^(?:\+\+|--|[+\-*/%]?=(?!=))/.test(afterValue)) return true;
  const beforeIdentifier = text.slice(0, identifierIndex).trimEnd();
  return beforeIdentifier.endsWith('++') || beforeIdentifier.endsWith('--');
}

function scanRule(rule, file, root, text, config) {
  if (Array.isArray(rule.requires) && rule.requires.some((pattern) => !ruleRegex(pattern).test(text))) {
    return [];
  }
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
    if (
      rule.id === 'native.shared-value-js-read' &&
      (isSharedValueWorkletRead(text, match.index ?? 0) || isSharedValueWrite(text, match))
    ) {
      continue;
    }
    const line = lineForIndex(text, match.index ?? 0);
    if (!isIgnored(config, rule.id, relativePath, lines, line)) {
      findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
    }
  }
  return findings;
}

function packageSpecMentionsMajor(spec, major) {
  return new RegExp(`(^|[^0-9])${major}(\\.|$)`).test(String(spec ?? ''));
}

function packageSpecMentionsVersion(spec, version) {
  const escaped = version.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`(^|[^0-9])${escaped}([^0-9]|$)`).test(String(spec ?? ''));
}

function scan(root, maxFiles) {
  if (!fs.existsSync(root)) throw new Error(`Root does not exist: ${root}`);
  const config = loadConfig(root);
  const files = listFiles(root, maxFiles);
  const pkg = readPackage(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'expoBundledSkiaMismatch') {
      const packageName = rule.package;
      for (const packageFile of pkg.packageFiles) {
        const expoSpec = packageFile.dependencyVersions?.expo;
        const packageSpec = packageFile.dependencyVersions?.[packageName];
        if (!(
          packageSpec &&
          packageSpecMentionsMajor(expoSpec, rule.expoMajor) &&
          !packageSpecMentionsVersion(packageSpec, rule.expected)
        )) continue;
        if (isIgnored(config, rule.id, packageFile.file, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packageFile.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageFile.file,
          line: 1,
          excerpt: `expo=${expoSpec}, ${packageName}=${packageSpec}, expected bundled ${rule.expected}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageMissingTrustedDependency') {
      const packageName = rule.package;
      for (const packageFile of pkg.packageFiles) {
        const likelyBun = packageFile.bunLock || packageFile.packageManager?.startsWith('bun@');
        if (!(
          packageFile.packages.has(packageName) &&
          likelyBun &&
          !packageFile.trustedDependencies?.has(packageName)
        )) continue;
        if (isIgnored(config, rule.id, packageFile.file, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packageFile.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageFile.file,
          line: 1,
          excerpt: `missing trustedDependencies entry: ${packageName}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
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
    configuredRules: profile.rules.length,
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    packageSpecs: {
      expo: pkg.dependencyVersions?.expo ?? null,
      skia: pkg.dependencyVersions?.['@shopify/react-native-skia'] ?? null,
      react: pkg.dependencyVersions?.react ?? null,
      reactNative: pkg.dependencyVersions?.['react-native'] ?? null,
      reanimated: pkg.dependencyVersions?.['react-native-reanimated'] ?? null,
    },
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
- Package specs: expo=${result.packageSpecs?.expo ?? 'n/a'}, skia=${result.packageSpecs?.skia ?? 'n/a'}, react=${result.packageSpecs?.react ?? 'n/a'}, react-native=${result.packageSpecs?.reactNative ?? 'n/a'}, reanimated=${result.packageSpecs?.reanimated ?? 'n/a'}
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
