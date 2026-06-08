#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "native-rive",
  "rules": [
    {
      "id": "rive.new-runtime-missing-nitro",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageRequiresAny",
      "whenPackage": "@rive-app/react-native",
      "requiresAny": ["react-native-nitro-modules"],
      "message": "@rive-app/react-native is installed without react-native-nitro-modules.",
      "recommendation": "Install the Nitro peer dependency using the repo package policy, then rebuild and run native smoke checks."
    },
    {
      "id": "rive.expo-needs-dev-build-proof",
      "severity": "medium",
      "confidence": "medium",
      "category": "native-build",
      "kind": "packagePairRequiresAny",
      "whenAll": ["expo", "@rive-app/react-native"],
      "requiresAny": ["expo-dev-client"],
      "message": "Expo and @rive-app/react-native are present, but expo-dev-client was not found.",
      "recommendation": "Verify the app has an equivalent development-build/prebuild/EAS lane. Expo Go is not runtime proof for Rive's native Nitro runtime."
    },
    {
      "id": "rive.expo53-android-build-props",
      "severity": "low",
      "confidence": "medium",
      "category": "native-build",
      "kind": "packageVersionRequiresAll",
      "whenPackage": "expo",
      "versionPattern": "^[~^]?53\\.",
      "alsoPackage": "@rive-app/react-native",
      "requiresAll": ["expo-build-properties", "expo-custom-agp"],
      "message": "Expo SDK 53 with @rive-app/react-native may need Android compile SDK / AGP overrides.",
      "recommendation": "If Android builds fail, add the repo-approved equivalent of expo-build-properties compileSdkVersion 36 and expo-custom-agp 8.9.1+ per Rive Expo docs, then prebuild/rebuild."
    },
    {
      "id": "rive.legacy-runtime-present",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "kind": "packageHasAny",
      "packages": ["rive-react-native"],
      "message": "The legacy rive-react-native package is present.",
      "recommendation": "Keep only if deliberately pinned; otherwise migrate to @rive-app/react-native plus react-native-nitro-modules and update call sites."
    },
    {
      "id": "rive.mixed-new-and-legacy-runtime",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageHasAll",
      "whenAll": ["@rive-app/react-native", "rive-react-native"],
      "message": "Both new and legacy Rive React Native runtimes are installed.",
      "recommendation": "Keep both only with an explicit migration plan; otherwise remove the unused runtime and migrate call sites to one canonical implementation."
    },
    {
      "id": "rive.raw-input-name",
      "severity": "medium",
      "confidence": "low",
      "category": "contract",
      "pattern": "(useRive(Number|Boolean|String|Color|Enum|List|Trigger)|artboardName=|stateMachineName=|dataBind=\\{\\{\\s*byName:)[^\\n]*['\"][A-Za-z0-9 _.:-]+['\"]",
      "message": "Rive asset contract names appear inline.",
      "recommendation": "Prefer a typed local mapping for artboards, state machines, view-model instances, property paths, and triggers."
    },
    {
      "id": "rive.deprecated-input-api",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "pattern": "\\.(setNumberInputValue|getNumberInputValue|setBooleanInputValue|getBooleanInputValue|triggerInput|setTextRunValue|getTextRunValue|onEventListener|removeEventListeners)\\(",
      "message": "Deprecated legacy state-machine input/text/event API is used.",
      "recommendation": "Prefer data binding and ViewModelInstance property hooks; if this is legacy migration, centralize names and add cleanup."
    },
    {
      "id": "rive.new-runtime-legacy-databinding-prop",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "fileContainsBoth",
      "include": "from\\s+['\"]@rive-app/react-native['\"]",
      "also": "dataBinding\\s*=",
      "message": "New-runtime imports appear with the legacy dataBinding prop.",
      "recommendation": "Use the new-runtime RiveView `dataBind` prop. `dataBinding` belongs to legacy examples and should not be mixed into @rive-app/react-native wrappers."
    },
    {
      "id": "rive.view-model-instance-not-bound",
      "severity": "medium",
      "confidence": "medium",
      "category": "contract",
      "kind": "fileContainsBothWithout",
      "include": "useViewModelInstance\\(",
      "also": "<RiveView\\b",
      "without": "dataBind\\s*=",
      "message": "A ViewModelInstance is created in a file rendering RiveView, but no dataBind prop was found.",
      "recommendation": "Bind the explicit ViewModelInstance through RiveView `dataBind`, or suppress with a reason if binding happens in another wrapper."
    },
    {
      "id": "rive.riveview-missing-onerror",
      "severity": "medium",
      "confidence": "medium",
      "category": "fallback",
      "kind": "fileContainsBothWithout",
      "include": "from\\s+['\"]@rive-app/react-native['\"]",
      "also": "<RiveView\\b",
      "without": "onError\\s*=",
      "message": "A new-runtime RiveView is rendered without an onError handler in the same file.",
      "recommendation": "Handle file, artboard, state-machine, view-model, and asset contract failures at the wrapper boundary."
    },
    {
      "id": "rive.riveview-missing-style",
      "severity": "low",
      "confidence": "medium",
      "category": "layout",
      "kind": "fileContainsBothWithout",
      "include": "from\\s+['\"]@rive-app/react-native['\"]",
      "also": "<RiveView\\b",
      "without": "style\\s*=",
      "message": "A new-runtime RiveView is rendered without a local style prop.",
      "recommendation": "Verify stable dimensions are supplied directly or through a wrapper. Blank Rive output is often layout-related."
    },
    {
      "id": "rive.remote-riv",
      "severity": "low",
      "confidence": "medium",
      "category": "asset",
      "pattern": "useRiveFile\\([^\\n]*(https?:\\/\\/[^'\"\\s]+\\.riv)",
      "message": "A remote .riv URL is loaded at runtime.",
      "recommendation": "Confirm caching, offline behavior, privacy/integrity, loading UI, and failure fallback; prefer local assets unless remote delivery is required."
    },
    {
      "id": "rive.native-resource-riv",
      "severity": "low",
      "confidence": "medium",
      "category": "asset",
      "pattern": "useRiveFile\\(\\s*['\"][A-Za-z0-9_][A-Za-z0-9_-]*['\"]\\s*\\)",
      "message": "A string literal Rive file source may be a native resource name.",
      "recommendation": "If this is native resource loading, verify iOS bundle resources, Android res/raw placement, and native rebuild proof. If it is a URL/config indirection, suppress with the policy reason."
    },
    {
      "id": "rive.dynamic-assets-need-policy",
      "severity": "low",
      "confidence": "medium",
      "category": "asset",
      "pattern": "(referencedAssets\\s*:|RiveImages\\.loadFromURLAsync\\(|\\.imageProperty\\()",
      "message": "Dynamic or referenced Rive assets are used.",
      "recommendation": "Verify exported asset keys, loading UI, cache/privacy/offline policy, missing asset fallback, and playIfNeeded behavior when replacing assets at runtime."
    },
    {
      "id": "rive.local-riv-needs-metro",
      "severity": "medium",
      "confidence": "medium",
      "category": "asset",
      "kind": "fileContainsWithoutRepo",
      "include": "(require\\([^\\n]*\\.riv['\"]\\)|import\\s+(?:[^'\";]+?\\s+from\\s+)?['\"][^'\"\\n]+\\.riv['\"])",
      "without": "assetExts\\.(push|unshift)\\(\\s*['\"]riv['\"]\\)|assetExts\\s*[:=]\\s*\\[[^\\]]*['\"]riv['\"][^\\]]*\\]",
      "message": "A local .riv asset is referenced, but repo-level Metro .riv support was not found.",
      "recommendation": "Add `riv` to Metro resolver.assetExts or verify the existing repo-specific asset pipeline."
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
        "rive-react-native",
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
  if (!fs.existsSync(root)) {
    return {
      exists: false,
      packageFiles: [],
      scripts: {},
    };
  }
  const files = listPackageFiles(root);
  const packageFiles = [];
  const scripts = {};
  for (const file of files) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      Object.assign(scripts, pkg.scripts ?? {});
      const deps = Object.assign(
        {},
        pkg.dependencies,
        pkg.devDependencies,
        pkg.peerDependencies,
        pkg.optionalDependencies,
      );
      const packageNames = new Set();
      const packageVersions = new Map();
      for (const [name, version] of Object.entries(deps ?? {})) {
        packageNames.add(name);
        if (typeof version === 'string') packageVersions.set(name, version);
      }
      packageFiles.push({
        file: path.relative(root, file),
        packages: packageNames,
        versions: packageVersions,
      });
    } catch {
      continue;
    }
  }
  return { exists: files.length > 0, packageFiles, scripts };
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

function makePackageRecordFinding(rule, record, excerpt) {
  return makeFinding(rule, record.file, 1, excerpt);
}

function packageRecordsWith(pkg, packageName) {
  return pkg.packageFiles.filter((record) => record.packages.has(packageName));
}

function isPackageRuleIgnored(config, ruleId, file) {
  return isIgnored(config, ruleId, file, [''], 1);
}

function ruleRegex(pattern) {
  return new RegExp(pattern, 'gms');
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (
    rule.kind === 'fileContainsWithout' ||
    rule.kind === 'fileContainsBoth' ||
    rule.kind === 'fileContainsBothWithout'
  ) {
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
  if (
    rule.kind === 'packageHasAny' ||
    rule.kind === 'packageRequiresAny' ||
    rule.kind === 'packageHasAll' ||
    rule.kind === 'packagePairRequiresAny' ||
    rule.kind === 'packageVersionRequiresAll' ||
    rule.kind === 'fileContainsWithoutRepo'
  ) {
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

function scanRepoContainsWithout(rule, files, root, config) {
  const includeRegex = ruleRegex(rule.include);
  const withoutRegex = ruleRegex(rule.without);
  let firstMatch = null;
  let hasWithout = false;

  for (const file of files) {
    let text;
    try {
      text = fs.readFileSync(file, 'utf8');
    } catch {
      continue;
    }
    if (!hasWithout && isMetroConfigFile(file) && withoutRegex.test(text)) {
      hasWithout = true;
    }
    if (!firstMatch) {
      includeRegex.lastIndex = 0;
      const match = includeRegex.exec(text);
      if (match) {
        const relativePath = path.relative(root, file);
        const lines = text.split('\n');
        const line = lineForIndex(text, match.index);
        if (!isIgnored(config, rule.id, relativePath, lines, line)) {
          firstMatch = { relativePath, line, excerpt: excerptForLine(lines, line), lines };
        }
      }
    }
    includeRegex.lastIndex = 0;
    withoutRegex.lastIndex = 0;
  }

  if (!firstMatch || hasWithout) return [];
  return [makeFinding(rule, firstMatch.relativePath, firstMatch.line, firstMatch.excerpt)];
}

function isMetroConfigFile(file) {
  return /^metro\.config\.(js|cjs|mjs|ts)$/.test(path.basename(file));
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
      for (const record of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => record.packages.has(name));
        if (matched.length > 0) {
          if (isPackageRuleIgnored(config, rule.id, record.file)) continue;
          findings.push(makePackageRecordFinding(rule, record, `matched packages: ${matched.join(', ')}`));
        }
      }
      continue;
    }
    if (rule.kind === 'packageRequiresAny') {
      for (const record of packageRecordsWith(pkg, rule.whenPackage)) {
        const matched = (rule.requiresAny ?? []).filter((name) => record.packages.has(name));
        if (matched.length === 0) {
          if (isPackageRuleIgnored(config, rule.id, record.file)) continue;
          findings.push(
            makePackageRecordFinding(
              rule,
              record,
              `package ${rule.whenPackage} requires one of: ${(rule.requiresAny ?? []).join(', ')}`,
            ),
          );
        }
      }
      continue;
    }
    if (rule.kind === 'packageHasAll') {
      const whenAll = rule.whenAll ?? [];
      for (const record of pkg.packageFiles) {
        if (whenAll.every((name) => record.packages.has(name))) {
          if (isPackageRuleIgnored(config, rule.id, record.file)) continue;
          findings.push(
            makePackageRecordFinding(
              rule,
              record,
              `matched packages: ${whenAll.join(', ')}`,
            ),
          );
        }
      }
      continue;
    }
    if (rule.kind === 'packagePairRequiresAny') {
      const whenAll = rule.whenAll ?? [];
      for (const record of pkg.packageFiles) {
        if (!whenAll.every((name) => record.packages.has(name))) continue;
        const matched = (rule.requiresAny ?? []).filter((name) => record.packages.has(name));
        if (matched.length === 0) {
          if (isPackageRuleIgnored(config, rule.id, record.file)) continue;
          findings.push(
            makePackageRecordFinding(
              rule,
              record,
              `packages ${whenAll.join(', ')} require one of: ${(rule.requiresAny ?? []).join(', ')}`,
            ),
          );
        }
      }
      continue;
    }
    if (rule.kind === 'packageVersionRequiresAll') {
      for (const record of packageRecordsWith(pkg, rule.whenPackage)) {
        const version = record.versions.get(rule.whenPackage) ?? '';
        const versionMatches = version && ruleRegex(rule.versionPattern).test(version);
        const alsoMatches = !rule.alsoPackage || record.packages.has(rule.alsoPackage);
        if (!versionMatches || !alsoMatches) continue;
        const missing = (rule.requiresAll ?? []).filter((name) => !record.packages.has(name));
        if (missing.length > 0) {
          if (isPackageRuleIgnored(config, rule.id, record.file)) continue;
          findings.push(
            makePackageRecordFinding(
              rule,
              record,
              `package ${rule.whenPackage}@${version} is missing: ${missing.join(', ')}`,
            ),
          );
        }
      }
      continue;
    }
    if (rule.kind === 'fileContainsWithoutRepo') {
      findings.push(...scanRepoContainsWithout(rule, files, root, config));
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
      'package checks scan workspace package.json files outside ignored directories',
      'regex findings are triage; verify them against the .riv contract and runtime behavior',
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
