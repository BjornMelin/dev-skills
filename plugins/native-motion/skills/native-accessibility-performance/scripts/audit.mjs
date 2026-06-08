#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "native-accessibility-performance",
  "rules": [
    {
      "id": "native.runonjs-reanimated-import",
      "severity": "medium",
      "confidence": "high",
      "category": "compatibility",
      "pattern": "import\\s+[^;]*\\brunOnJS\\b[^;]*from\\s+['\"]react-native-reanimated['\"]",
      "message": "runOnJS is imported through Reanimated; Reanimated 4 source marks that re-export deprecated.",
      "recommendation": "Verify installed versions. Prefer scheduleOnRN from react-native-worklets when available, or import runOnJS directly from react-native-worklets."
    },
    {
      "id": "native.motion-missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\b(withRepeat|withTiming|withSpring|withDecay|withSequence|LayoutAnimation|Animated\\.(timing|spring|decay|loop|sequence|parallel)|useFrameCallback|lottie|Lottie|Moti|Rive|Skia)\\b",
      "without": "prefers-reduced-motion|motion-reduce|useReducedMotion|ReducedMotion|ReduceMotion|AccessibilityInfo|reduceMotion",
      "message": "Motion code was found without an obvious reduced-motion branch in the same file.",
      "recommendation": "Add reduced-motion behavior or document where the effect is already reduced at a higher level."
    },
    {
      "id": "native.repeat-no-cancel",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "\\b(withRepeat|Animated\\.loop)\\(",
      "without": "cancelAnimation|useReducedMotion|ReducedMotion|AccessibilityInfo|cleanup|return\\s*\\(\\s*\\)\\s*=>",
      "message": "Repeating native animation lacks an obvious cancel or reduced-motion branch.",
      "recommendation": "Cancel repeaters on unmount/state change and reduce decorative loops for reduced motion."
    },
    {
      "id": "native.haptics-high-frequency",
      "severity": "medium",
      "confidence": "low",
      "category": "accessibility",
      "kind": "fileContainsBothWithout",
      "include": "\\bHaptics\\.",
      "also": "\\b(withRepeat|useFrameCallback|useAnimatedReaction|onScroll|setInterval|autoplay|loop)\\b",
      "without": "debounce|throttle|onPress|onLongPress|selectionAsync|performAndroidHapticsAsync|reduced|reduceMotion|AccessibilityInfo",
      "message": "Haptics appear near repeated or high-frequency motion code.",
      "recommendation": "Tie haptics to intentional user action, avoid repeated tactile output, and test on real devices."
    },
    {
      "id": "native.assertive-live-region-motion",
      "severity": "medium",
      "confidence": "low",
      "category": "accessibility",
      "kind": "fileContainsBoth",
      "include": "accessibilityLiveRegion\\s*=\\s*['\"{]?assertive",
      "also": "\\b(withRepeat|withTiming|withSpring|useFrameCallback|Animated\\.|setInterval|onScroll)\\b",
      "message": "Assertive live region appears near animated or repeated updates.",
      "recommendation": "Verify TalkBack output. Prefer polite or settled announcements unless the update is urgent."
    },
    {
      "id": "native.global-reduced-motion-never",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "pattern": "ReducedMotionConfig[^\\n]*mode=\\{?ReduceMotion\\.Never",
      "message": "Global ReducedMotionConfig disables reduced-motion adaptation.",
      "recommendation": "Confirm this is an explicit app-wide accessibility decision; prefer local exceptions for essential motion."
    },
    {
      "id": "native.local-reduced-motion-never",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "pattern": "(reduceMotion\\s*:\\s*ReduceMotion\\.Never|\\.reduceMotion\\(\\s*ReduceMotion\\.Never)",
      "message": "A local animation opts out of reduced-motion adaptation.",
      "recommendation": "Verify this motion is essential and provide a static or lower-motion equivalent where possible."
    },
    {
      "id": "native.accessibilityinfo-listener-no-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsBothWithout",
      "include": "AccessibilityInfo\\.addEventListener\\(",
      "also": "(reduceMotionChanged|screenReaderChanged|accessibilityServiceChanged|announcementFinished)",
      "without": "\\.remove\\(|removeEventListener|return\\s*\\(\\s*\\)\\s*=>",
      "message": "AccessibilityInfo listener lacks an obvious cleanup path.",
      "recommendation": "Remove subscriptions on unmount and verify rapid navigation does not leave stale accessibility handlers."
    },
    {
      "id": "native.deprecated-set-accessibility-focus",
      "severity": "medium",
      "confidence": "high",
      "category": "compatibility",
      "pattern": "AccessibilityInfo\\.setAccessibilityFocus\\(",
      "message": "React Native 0.85 marks setAccessibilityFocus as deprecated.",
      "recommendation": "Prefer sendAccessibilityEvent(ref, 'focus') when the installed React Native version supports it."
    },
    {
      "id": "native.announce-from-motion-loop",
      "severity": "medium",
      "confidence": "low",
      "category": "accessibility",
      "kind": "fileContainsBoth",
      "include": "AccessibilityInfo\\.announceForAccessibility",
      "also": "\\b(withRepeat|useFrameCallback|useAnimatedReaction|onScroll|setInterval|requestAnimationFrame|Animated\\.)\\b",
      "message": "Screen-reader announcements appear near animated or high-frequency update code.",
      "recommendation": "Announce a settled state instead of every frame/tick and verify VoiceOver/TalkBack output."
    },
    {
      "id": "native.shared-value-js-read",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "pattern": "[^A-Za-z0-9_]([A-Za-z0-9_]+)\\.value",
      "message": "Shared value reads on the JS thread can block or desynchronize if used outside worklets.",
      "recommendation": "Confirm this read is inside a worklet; otherwise derive and consume on the UI thread."
    },
    {
      "id": "native.animated-native-driver-false",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "pattern": "Animated\\.(timing|spring|decay)\\([\\s\\S]{0,500}useNativeDriver\\s*:\\s*false",
      "message": "Animated API is configured without the native driver in a nearby animation config.",
      "recommendation": "Verify this is required for unsupported props. Prefer native/UI-thread animation for hot interaction paths."
    },
    {
      "id": "native.animated-native-driver-missing",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "kind": "fileContainsWithout",
      "include": "Animated\\.(timing|spring|decay)\\(",
      "without": "useNativeDriver\\s*:",
      "message": "Animated API usage lacks an obvious useNativeDriver decision in the same file.",
      "recommendation": "Set useNativeDriver when supported, or document why JS-driven animation is required for unsupported props."
    },
    {
      "id": "native.animated-loop-list-interaction",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "kind": "fileContainsBothWithout",
      "include": "Animated\\.loop\\(",
      "also": "\\b(FlatList|VirtualizedList|SectionList|FlashList)\\b",
      "without": "isInteraction\\s*:\\s*false",
      "message": "A long or looping Animated animation appears in a list surface without isInteraction false.",
      "recommendation": "Verify list row rendering while the animation runs; for non-interaction loops, consider isInteraction: false."
    },
    {
      "id": "native.layout-prop-animation",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "\\b(width|height|top|left|right|bottom|margin|marginTop|marginBottom|marginLeft|marginRight|padding|paddingTop|paddingBottom|paddingLeft|paddingRight)\\s*:\\s*(withTiming|withSpring|withDecay)\\(",
      "message": "A layout-affecting style is animated directly.",
      "recommendation": "Prefer transform/opacity or measure the layout cost on representative devices before accepting this path."
    },
    {
      "id": "native.frame-callback-not-memoized",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "kind": "fileContainsWithout",
      "include": "useFrameCallback\\(",
      "without": "useCallback\\(|React Compiler|reactCompiler|compiler",
      "message": "useFrameCallback appears without an obvious stable callback.",
      "recommendation": "Memoize frame callbacks unless React Compiler or local patterns prove stable identity."
    },
    {
      "id": "native.list-gesture-not-memoized",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "kind": "fileContainsBothWithout",
      "include": "\\bGesture\\.[A-Za-z]+\\(",
      "also": "\\b(FlatList|FlashList|VirtualizedList|SectionList|renderItem)\\b",
      "without": "useMemo\\(|React Compiler|reactCompiler|compiler",
      "message": "Gesture objects appear in a list/render path without obvious memoization.",
      "recommendation": "Memoize gesture objects in list rows unless compiler support or local profiling shows no reattachment cost."
    },
    {
      "id": "native.inline-worklet-rn-scheduler",
      "severity": "medium",
      "confidence": "medium",
      "category": "compatibility",
      "pattern": "(scheduleOnRN|runOnJS)\\(\\s*\\([^)]*\\)\\s*=>",
      "message": "A UI-runtime-to-RN/JS scheduler receives an inline function.",
      "recommendation": "Define scheduled functions in RN/JS scope, then pass the function reference from the worklet."
    },
    {
      "id": "native.runonjs-invoked-argument",
      "severity": "medium",
      "confidence": "medium",
      "category": "compatibility",
      "pattern": "runOnJS\\(\\s*[A-Za-z_$][A-Za-z0-9_$]*\\s*\\(",
      "message": "runOnJS appears to receive the result of a function call.",
      "recommendation": "Pass the function reference to runOnJS, then provide arguments to the returned callable."
    },
    {
      "id": "native.reanimated-static-flags",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "pattern": "\"staticFeatureFlags\"\\s*:",
      "message": "Reanimated static feature flags are configured.",
      "recommendation": "Verify every flag exists in the installed Reanimated version, then record native rebuild and iOS/Android runtime proof; Expo Go cannot change these flags."
    },
    {
      "id": "native.package-needs-doctor",
      "severity": "medium",
      "confidence": "medium",
      "category": "validation",
      "kind": "packageHasAny",
      "packages": [
        "react-native-reanimated",
        "react-native-worklets",
        "react-native-gesture-handler",
        "nativewind",
        "expo-haptics",
        "@shopify/flash-list",
        "@shopify/react-native-skia",
        "@rive-app/react-native",
        "lottie-react-native",
        "expo-gl"
      ],
      "message": "Native motion dependencies are present and should be validated with platform-specific checks.",
      "recommendation": "Run the repo doctor/typecheck/native smoke commands and record iOS/Android proof."
    }
  ]
};

const skipDirs = new Set([
  '.git', 'node_modules', '.next', '.nuxt', 'dist', 'build', 'coverage',
  '.expo', '.turbo', '.vercel', '.cache', '.codex', '.agents',
  'output', 'tmp', 'temp', 'vendor', 'playwright-report', 'storybook-static',
]);
const skillResourceDirs = new Set(['agents', 'evals', 'references', 'templates']);
const fileExtensions = new Set([
  '.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.css', '.scss', '.sass',
  '.html', '.vue', '.svelte', '.json',
]);
const severities = ['low', 'medium', 'high'];
const relevantPackages = [
  'expo',
  'react',
  'react-native',
  'react-native-reanimated',
  'react-native-worklets',
  'react-native-gesture-handler',
  'expo-haptics',
  '@expo/ui',
  '@shopify/flash-list',
  '@shopify/react-native-skia',
  '@rive-app/react-native',
  'lottie-react-native',
  'expo-gl',
];

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
  const scanningSkillRoot = fs.existsSync(path.join(root, 'SKILL.md'));
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (scanningSkillRoot && skillResourceDirs.has(rel.split(path.sep)[0])) continue;
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
  const scanningSkillRoot = fs.existsSync(path.join(root, 'SKILL.md'));
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (scanningSkillRoot && skillResourceDirs.has(rel.split(path.sep)[0])) continue;
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
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), versions: {}, scripts: {}, packageFiles: [] };
  const packages = new Set();
  const versions = {};
  const scripts = {};
  const packageFiles = [];
  for (const file of listPackageFiles(root, 2000)) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
      const names = new Set(Object.keys(deps ?? {}));
      for (const name of names) packages.add(name);
      Object.assign(versions, deps ?? {});
      Object.assign(scripts, pkg.scripts ?? {});
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
  return { exists: packageFiles.length > 0, packages, versions, scripts, packageFiles };
}

function relevantPackageVersions(pkg) {
  return Object.fromEntries(
    relevantPackages
      .filter((name) => Object.prototype.hasOwnProperty.call(pkg.versions, name))
      .map((name) => [name, pkg.versions[name]])
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

function uniqueFindings(findings) {
  return [...new Map(findings.map((finding) => [finding.id, finding])).values()];
}

function ruleRegex(pattern) {
  return new RegExp(pattern, 'gms');
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
      for (const packageInfo of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageInfo.packages.has(name));
        if (matched.length === 0) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        const matchedWithVersions = matched.map((name) => `${name}@${packageInfo.versions[name] ?? 'unknown'}`);
        findings.push({
          id: `${rule.id}:${packageInfo.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageInfo.file,
          line: 1,
          excerpt: `matched packages: ${matchedWithVersions.join(', ')}`,
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
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    packageFiles: pkg.packageFiles.length,
    relevantPackageVersions: relevantPackageVersions(pkg),
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
    const versions = Object.entries(result.relevantPackageVersions ?? {})
      .map(([name, version]) => `  - ${name}: ${version}`)
      .join('\n');
    return `# Motion Audit Doctor: ${result.profile}

- Root: ${result.root}
- Package JSON: ${result.packageJson ? 'yes' : 'no'}
- Config file: ${result.configFile ? 'yes' : 'no'}
- Configured rules: ${result.configuredRules}
- Sample file count: ${result.sampleFileCount}
- Relevant package versions:
${versions || '  - (none found)'}
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
