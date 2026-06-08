#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "native-styling-boundaries",
  "rules": [
    {
      "id": "tailwind.dynamic-class",
      "severity": "medium",
      "confidence": "medium",
      "category": "build",
      "pattern": "className\\s*=\\s*{[^}\\n]*(\\$\\{|join\\(|clsx\\([^)]*\\$\\{)|class\\s*=\\s*[\"'][^\"']*\\$\\{",
      "message": "Dynamic class construction can hide Tailwind classes from extraction.",
      "recommendation": "Use explicit class maps or safelist the finite set of generated classes."
    },
    {
      "id": "tailwind.template-interpolation",
      "severity": "medium",
      "confidence": "high",
      "category": "build",
      "pattern": "className\\s*=\\s*{\\s*`[^`]*\\$\\{",
      "message": "Tailwind cannot detect class names assembled with template interpolation.",
      "recommendation": "Map inputs to complete class strings that are present in source."
    },
    {
      "id": "tailwind.v4-old-directives",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageVersionFileContains",
      "package": "tailwindcss",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*4\\.|\\b(?:latest|next)\\b",
      "include": "@tailwind\\s+(base|components|utilities)",
      "message": "Tailwind v4 packages are present but CSS still uses Tailwind v3 directives.",
      "recommendation": "Use Tailwind v4 CSS imports and directives, or confirm the package is intentionally pinned to a v3-compatible lane."
    },
    {
      "id": "tailwind.v4-broad-source",
      "severity": "medium",
      "confidence": "medium",
      "category": "build",
      "pattern": "@source\\s+(?:inline\\()?['\\\"][^'\\\"]*(?:\\.\\.\\/\\.\\.|node_modules|vendor|dist|build|coverage|\\*)",
      "message": "Tailwind v4 source detection appears to scan a broad/generated/vendor path.",
      "recommendation": "Use narrow @source paths for app/shared-package code and @source inline() only for a finite generated utility set."
    },
    {
      "id": "nativewind.v5-missing-peer-packages",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageVersionMissingPackages",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "requiredPackages": ["tailwindcss", "@tailwindcss/postcss", "react-native-css", "react-native-reanimated", "react-native-safe-area-context"],
      "message": "NativeWind v5 preview setup is missing one or more required peer/runtime packages.",
      "recommendation": "Install the Expo-compatible NativeWind v5 peer set, or stay on the existing NativeWind v4 lane."
    },
    {
      "id": "nativewind.v5-tailwind-not-v4",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageVersionRequiresPackageVersion",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "requiredPackage": "tailwindcss",
      "requiredVersionPattern": "^(?:[~^]|>=?|<=?)?\\s*4\\.|\\b(?:latest|next)\\b",
      "message": "NativeWind v5 preview expects Tailwind CSS v4.",
      "recommendation": "Pin Tailwind to the supported v4 lane for NativeWind v5, or keep the app on NativeWind v4/Tailwind v3."
    },
    {
      "id": "nativewind.v5-reanimated-not-v4",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageVersionRequiresPackageVersion",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "requiredPackage": "react-native-reanimated",
      "requiredVersionPattern": "^(?:[~^]|>=?|<=?)?\\s*4\\.|\\b(?:latest|next)\\b",
      "message": "NativeWind v5 preview expects Reanimated 4-era CSS animation support.",
      "recommendation": "Use the Expo-compatible Reanimated 4 lane before relying on NativeWind v5 native CSS animation behavior."
    },
    {
      "id": "nativewind.v5-missing-with-nativewind",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "kind": "packageVersionMissingFileContains",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "include": "withNativewind\\s*\\(",
      "message": "NativeWind v5 preview is installed but no withNativewind Metro wrapper was detected.",
      "recommendation": "Verify Metro is wrapped once with nativewind/metro and composed in the repo's expected order."
    },
    {
      "id": "nativewind.v5-legacy-babel",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "kind": "packageVersionFileContains",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "include": "nativewind/babel",
      "message": "NativeWind v5 setup should not carry the old NativeWind Babel preset by default.",
      "recommendation": "Verify whether Babel is still needed for another tool; otherwise use the v5 Metro/react-native-css setup and clear Metro cache."
    },
    {
      "id": "nativewind.v5-legacy-env-types",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageVersionFileContains",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "include": "nativewind/types",
      "message": "NativeWind v5 setup should not keep the v4 nativewind/types env reference by default.",
      "recommendation": "Use the react-native-css/types or generated v5 env typing lane for the target setup."
    },
    {
      "id": "nativewind.v5-css-missing-nativewind-theme",
      "severity": "low",
      "confidence": "medium",
      "category": "tokens",
      "kind": "packageVersionFileContainsWithout",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*5\\.|\\b(?:preview|next)\\b",
      "include": "@import\\s+['\\\"]tailwindcss(?:/theme\\.css)?['\\\"]",
      "without": "nativewind/theme",
      "message": "NativeWind v5 CSS imports Tailwind but does not import nativewind/theme in the same file.",
      "recommendation": "If the app relies on NativeWind RN theme values or platform variants, import nativewind/theme in global.css."
    },
    {
      "id": "nativewind.v4-tailwind-v4-mix",
      "severity": "medium",
      "confidence": "high",
      "category": "migration",
      "kind": "packageVersionIncompatiblePackageVersion",
      "package": "nativewind",
      "versionPattern": "^(?:[~^]|>=?|<=?)?\\s*4\\.|\\blatest\\b",
      "incompatiblePackage": "tailwindcss",
      "incompatibleVersionPattern": "^(?:[~^]|>=?|<=?)?\\s*4\\.|\\b(?:latest|next)\\b",
      "message": "NativeWind stable v4 is installed with a Tailwind v4 package lane.",
      "recommendation": "Keep NativeWind v4 on the Tailwind v3-compatible setup, or perform an explicit NativeWind v5 preview migration."
    },
    {
      "id": "expo.tailwind-web-only-native",
      "severity": "low",
      "confidence": "medium",
      "category": "platform",
      "kind": "packageHasAnyWithoutAny",
      "packages": ["expo"],
      "alsoPackages": ["tailwindcss"],
      "missingAnyPackages": ["nativewind", "uniwind", "react-native-css"],
      "message": "Expo plus Tailwind without a native compatibility layer is web-only by default.",
      "recommendation": "If Android/iOS screens use Tailwind-style classes, add/verify NativeWind or Uniwind; otherwise keep this lane web-only or use DOM components intentionally."
    },
    {
      "id": "nativewind.platform-select-class",
      "severity": "low",
      "confidence": "medium",
      "category": "platform",
      "pattern": "className\\s*=\\s*{[^}\\n]*Platform\\.select",
      "message": "className is selected at runtime with Platform.select, which can hide complete Tailwind classes from extraction.",
      "recommendation": "Prefer static class maps or NativeWind platform variants such as ios:, android:, native:, and web:."
    },
    {
      "id": "nativewind.component-mapping-registration",
      "severity": "low",
      "confidence": "medium",
      "category": "interop",
      "pattern": "\\b(cssInterop|remapProps|useCssElement|styled)\\s*\\(",
      "message": "NativeWind component mapping API detected.",
      "recommendation": "Verify the mapping API matches the installed NativeWind lane and is centralized in typed wrappers instead of screen-local registration."
    },
    {
      "id": "expo.dom-component-boundary",
      "severity": "low",
      "confidence": "medium",
      "category": "platform",
      "pattern": "['\\\"]use dom['\\\"]",
      "message": "Expo DOM component boundary detected.",
      "recommendation": "Verify this is an intentional DOM/WebView island with local CSS imports, serializable props, and native accessibility/gesture proof."
    },
    {
      "id": "nativewind.animation-utility",
      "severity": "low",
      "confidence": "medium",
      "category": "motion",
      "pattern": "className\\s*=\\s*(?:[\"'`][^\"'`\\n]*\\banimate-|{[^}\\n]*[\"'`][^\"'`\\n]*\\banimate-)",
      "message": "NativeWind animation utilities rely on native animation support that may be experimental or limited.",
      "recommendation": "Verify reduced motion, interruption, remount behavior, and target-platform support; use Reanimated directly for complex motion."
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

function dependencyMap(pkg) {
  return Object.assign(
    {},
    pkg.dependencies,
    pkg.devDependencies,
    pkg.peerDependencies,
    pkg.optionalDependencies,
  );
}

function readPackage(root) {
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), versions: {}, scripts: {}, packageFiles: [] };
  const packageFiles = [];
  const packages = new Set();
  const versions = {};
  const scripts = {};
  for (const file of listPackageFiles(root, 2000)) {
    try {
      const raw = JSON.parse(fs.readFileSync(file, 'utf8'));
      const deps = dependencyMap(raw);
      const names = new Set(Object.keys(deps ?? {}));
      for (const name of names) packages.add(name);
      Object.assign(versions, deps ?? {});
      Object.assign(scripts, raw.scripts ?? {});
      packageFiles.push({
        file: path.relative(root, file),
        packages: names,
        versions: deps ?? {},
        scripts: raw.scripts ?? {},
      });
    } catch {
      continue;
    }
  }
  return { exists: packageFiles.length > 0, packages, versions, scripts, packageFiles };
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

function maskJsComments(text) {
  let result = '';
  let quote = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    const next = text[index + 1];
    if (lineComment) {
      if (char === '\n') {
        lineComment = false;
        result += char;
      } else {
        result += ' ';
      }
      continue;
    }
    if (blockComment) {
      if (char === '*' && next === '/') {
        blockComment = false;
        result += '  ';
        index += 1;
      } else {
        result += char === '\n' ? char : ' ';
      }
      continue;
    }
    if (quote) {
      result += char;
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '/' && next === '/') {
      lineComment = true;
      result += '  ';
      index += 1;
      continue;
    }
    if (char === '/' && next === '*') {
      blockComment = true;
      result += '  ';
      index += 1;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') quote = char;
    result += char;
  }
  return result;
}

function isMetroConfigFile(file) {
  return /^metro\.config\.(?:js|cjs|mjs|ts)$/.test(path.basename(file));
}

function packageVersion(pkg, name) {
  const version = pkg.versions?.[name];
  return typeof version === 'string' ? version : null;
}

function packageVersionMatches(pkg, rule) {
  const version = packageVersion(pkg, rule.package);
  return Boolean(version && new RegExp(rule.versionPattern).test(version));
}

function packageVersionDoesNotMatch(pkg, name, pattern) {
  const version = packageVersion(pkg, name);
  return !version || !new RegExp(pattern).test(version);
}

function packageFinding(rule, excerpt, file = 'package.json') {
  return {
    id: `${rule.id}:${file}:1`,
    ruleId: rule.id,
    severity: rule.severity,
    confidence: rule.confidence,
    category: rule.category,
    file,
    line: 1,
    excerpt,
    rationale: rule.message,
    recommendation: rule.recommendation,
  };
}

function uniqueFindings(findings) {
  return [...new Map(findings.map((finding) => [finding.id, finding])).values()];
}

function packageDir(packageInfo) {
  const dir = path.dirname(packageInfo.file);
  return dir === '.' ? '' : dir;
}

function isUnderPackageDir(relativePath, packageInfo) {
  const dir = packageDir(packageInfo);
  return dir === '' || relativePath === dir || relativePath.startsWith(`${dir}/`);
}

function owningPackage(relativePath, packageFiles) {
  return packageFiles
    .filter((packageInfo) => isUnderPackageDir(relativePath, packageInfo))
    .sort((a, b) => packageDir(b).length - packageDir(a).length)[0] ?? null;
}

function filesOwnedByPackage(files, root, packageInfo, packageFiles) {
  return files.filter((file) => owningPackage(path.relative(root, file), packageFiles)?.file === packageInfo.file);
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'fileContainsWithout' || rule.kind === 'fileContainsBoth' || rule.kind === 'fileContainsBothWithout' || rule.kind === 'packageVersionFileContains' || rule.kind === 'packageVersionFileContainsWithout') {
    const includeMatch = ruleRegex(rule.include).exec(text);
    const alsoMatch = rule.also ? ruleRegex(rule.also).exec(text) : null;
    const withoutMatch = rule.without ? ruleRegex(rule.without).exec(text) : null;
    const matches =
      rule.kind === 'packageVersionFileContains'
        ? includeMatch
        : rule.kind === 'packageVersionFileContainsWithout'
          ? includeMatch && !withoutMatch
        : rule.kind === 'fileContainsBoth'
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
  const packageFiles = pkg.packageFiles.length > 0 ? pkg.packageFiles : [{ file: 'package.json', packages: pkg.packages, versions: pkg.versions }];
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageHasAny') {
      for (const packageInfo of packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageInfo.packages.has(name));
        if (matched.length === 0) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        findings.push(packageFinding(rule, `matched packages: ${matched.join(', ')}`, packageInfo.file));
      }
      continue;
    }
    if (rule.kind === 'packageHasAnyWithoutAny') {
      for (const packageInfo of packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageInfo.packages.has(name));
        const alsoMatched = (rule.alsoPackages ?? []).every((name) => packageInfo.packages.has(name));
        const missingAlternatives = !(rule.missingAnyPackages ?? []).some((name) => packageInfo.packages.has(name));
        if (matched.length === 0 || !alsoMatched || !missingAlternatives) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        findings.push(packageFinding(rule, `matched packages: ${[...matched, ...(rule.alsoPackages ?? [])].join(', ')}; missing one of: ${(rule.missingAnyPackages ?? []).join(', ')}`, packageInfo.file));
      }
      continue;
    }
    if (rule.kind === 'packageVersionMissingPackages') {
      for (const packageInfo of packageFiles) {
        if (!packageVersionMatches(packageInfo, rule)) continue;
        const missing = (rule.requiredPackages ?? []).filter((name) => !packageInfo.packages.has(name));
        if (missing.length === 0) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        findings.push(packageFinding(rule, `${rule.package}@${packageVersion(packageInfo, rule.package)}; missing: ${missing.join(', ')}`, packageInfo.file));
      }
      continue;
    }
    if (rule.kind === 'packageVersionRequiresPackageVersion') {
      for (const packageInfo of packageFiles) {
        if (!packageVersionMatches(packageInfo, rule) || !packageVersionDoesNotMatch(packageInfo, rule.requiredPackage, rule.requiredVersionPattern)) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        const installed = packageVersion(packageInfo, rule.requiredPackage) ?? '<missing>';
        findings.push(packageFinding(rule, `${rule.package}@${packageVersion(packageInfo, rule.package)}; ${rule.requiredPackage}@${installed}`, packageInfo.file));
      }
      continue;
    }
    if (rule.kind === 'packageVersionIncompatiblePackageVersion') {
      for (const packageInfo of packageFiles) {
        const version = packageVersion(packageInfo, rule.incompatiblePackage);
        if (!packageVersionMatches(packageInfo, rule) || !version || !new RegExp(rule.incompatibleVersionPattern).test(version)) continue;
        if (isIgnored(config, rule.id, packageInfo.file, [''], 1)) continue;
        findings.push(packageFinding(rule, `${rule.package}@${packageVersion(packageInfo, rule.package)}; ${rule.incompatiblePackage}@${version}`, packageInfo.file));
      }
      continue;
    }
    if (rule.kind === 'packageVersionMissingFileContains') {
      for (const packageInfo of packageFiles) {
        if (!packageVersionMatches(packageInfo, rule)) continue;
        const scopedFiles = filesOwnedByPackage(files, root, packageInfo, packageFiles);
        const evidenceFiles = rule.id === 'nativewind.v5-missing-with-nativewind'
          ? scopedFiles.filter(isMetroConfigFile)
          : scopedFiles;
        let matched = false;
        for (const file of evidenceFiles) {
          let text;
          try {
            text = fs.readFileSync(file, 'utf8');
          } catch {
            continue;
          }
          const searchableText = rule.id === 'nativewind.v5-missing-with-nativewind'
            ? maskJsComments(text)
            : text;
          if (ruleRegex(rule.include).test(searchableText)) {
            matched = true;
            break;
          }
        }
        if (!matched && !isIgnored(config, rule.id, packageInfo.file, [''], 1)) {
          findings.push(packageFinding(rule, `${rule.package}@${packageVersion(packageInfo, rule.package)}; missing source pattern: ${rule.include}`, packageInfo.file));
        }
      }
      continue;
    }
    if (rule.kind === 'packageVersionFileContains') {
      for (const packageInfo of packageFiles) {
        if (!packageVersionMatches(packageInfo, rule)) continue;
        for (const file of filesOwnedByPackage(files, root, packageInfo, packageFiles)) {
          let text;
          try {
            text = fs.readFileSync(file, 'utf8');
          } catch {
            continue;
          }
          findings.push(...scanRule(rule, file, root, text, config));
        }
      }
      continue;
    }
    if (rule.kind === 'packageVersionFileContainsWithout') {
      for (const packageInfo of packageFiles) {
        if (!packageVersionMatches(packageInfo, rule)) continue;
        for (const file of filesOwnedByPackage(files, root, packageInfo, packageFiles)) {
          let text;
          try {
            text = fs.readFileSync(file, 'utf8');
          } catch {
            continue;
          }
          findings.push(...scanRule(rule, file, root, text, config));
        }
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
    scannedPackageFiles: packageFiles.length,
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
  const watchedPackages = [
    'expo',
    'react-native',
    'nativewind',
    'uniwind',
    'tailwindcss',
    'react-native-css',
    'react-native-reanimated',
    'react-native-safe-area-context',
    '@tailwindcss/postcss',
  ];
  const packageVersions = Object.fromEntries(
    watchedPackages
      .map((name) => [name, packageVersion(pkg, name)])
      .filter(([, version]) => version !== null),
  );
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    packageFiles: pkg.packageFiles.length,
    packageVersions,
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
- Package versions: ${Object.keys(result.packageVersions ?? {}).length > 0 ? JSON.stringify(result.packageVersions) : '(none detected)'}
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
