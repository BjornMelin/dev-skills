#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const nativeMotionPackages = [
  'react-native-reanimated',
  'react-native-worklets',
  'nativewind',
  'react-native-css',
  '@shopify/react-native-skia',
  '@rive-app/react-native',
  'lottie-react-native',
  '@lottiefiles/dotlottie-react-native',
  'expo-gl',
  'expo-dev-client',
  'expo-build-properties',
];

const profile = {
  skillName: 'native-validation',
  rules: [
    {
      id: 'native.package-needs-doctor',
      severity: 'medium',
      confidence: 'medium',
      category: 'validation',
      kind: 'packageHasAny',
      packages: nativeMotionPackages,
      message: 'Native motion dependencies are present and should be validated with platform-specific checks.',
      recommendation: 'Run the repo equivalent of expo install --check, Expo Doctor when applicable, focused tests, and iOS/Android native proof.',
    },
    {
      id: 'native.runonjs-reanimated4',
      severity: 'medium',
      confidence: 'medium',
      category: 'compatibility',
      whenReanimatedMajor: 4,
      pattern: '\\brunOnJS\\s*\\(',
      message: 'runOnJS is present; Reanimated 4 keeps compatibility exports but worklets APIs may be the current owner.',
      recommendation: 'Verify installed Reanimated/worklets version and import source; prefer current react-native-worklets threading APIs for new Reanimated 4 code.',
    },
    {
      id: 'native.expo-config-needs-prebuild-proof',
      severity: 'medium',
      confidence: 'medium',
      category: 'validation',
      kind: 'fileContainsAny',
      files: [
        'app.json',
        'app.config.js',
        'app.config.cjs',
        'app.config.mjs',
        'app.config.ts',
      ],
      include: '\\b(plugins|ios|android|permissions|entitlements|newArchEnabled|runtimeVersion|splash|icon)\\b',
      message: 'Expo native configuration is present and may affect generated native output.',
      recommendation: 'For changes to this file, run Expo Doctor and inspect prebuild config or native build output per repo policy.',
    },
  ],
};

const skipDirs = new Set([
  '.git',
  'node_modules',
  '.next',
  '.nuxt',
  'dist',
  'build',
  'coverage',
  '.expo',
  '.turbo',
  '.vercel',
  '.cache',
  '.codex',
  '.agents',
  'output',
  'tmp',
  'temp',
  'vendor',
  'playwright-report',
  'storybook-static',
]);
const fileExtensions = new Set([
  '.js',
  '.jsx',
  '.ts',
  '.tsx',
  '.mjs',
  '.cjs',
  '.css',
  '.scss',
  '.sass',
  '.html',
  '.vue',
  '.svelte',
  '.json',
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
  scripts/audit.mjs doctor --root . --format json
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
  const args = {
    command: null,
    root: process.cwd(),
    format: 'markdown',
    output: null,
    maxFiles: 2000,
  };
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
  if (!['scan', 'doctor'].includes(args.command)) {
    throw new Error(`Unknown command: ${args.command}`);
  }
  if (!['markdown', 'json'].includes(args.format)) {
    throw new Error(`Unknown format: ${args.format}`);
  }
  if (!Number.isFinite(args.maxFiles) || args.maxFiles < 1) {
    throw new Error('--max-files must be a positive number');
  }
  return args;
}

function loadConfig(root) {
  const file = path.join(root, '.motion-audit.json');
  if (!fs.existsSync(file)) {
    return { ignoreRules: [], ignorePaths: [], ignores: [] };
  }
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

function readTextIfExists(file) {
  return fs.existsSync(file) ? fs.readFileSync(file, 'utf8') : '';
}

function readJsonIfExists(file) {
  const text = readTextIfExists(file);
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
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
      raw: {},
      packages: new Set(),
      dependencies: {},
      scripts: {},
      packageManager: null,
      packageFiles: [],
    };
  }
  const files = listPackageFiles(root);
  const packages = new Set();
  const dependencies = {};
  const scripts = {};
  const packageFiles = [];
  let packageManager = null;
  let rawRoot = {};
  for (const file of files) {
    try {
      const raw = JSON.parse(fs.readFileSync(file, 'utf8'));
      if (path.relative(root, file) === 'package.json') rawRoot = raw;
      const deps = Object.assign({}, raw.dependencies, raw.devDependencies, raw.peerDependencies, raw.optionalDependencies);
      Object.assign(dependencies, deps ?? {});
      Object.assign(scripts, raw.scripts ?? {});
      if (!packageManager && raw.packageManager) packageManager = raw.packageManager;
      const packageNames = new Set(Object.keys(deps ?? {}));
      for (const name of packageNames) packages.add(name);
      packageFiles.push({
        file: path.relative(root, file),
        raw,
        packages: packageNames,
        dependencies: deps ?? {},
        scripts: raw.scripts ?? {},
        packageManager: raw.packageManager ?? null,
      });
    } catch {
      continue;
    }
  }
  return { exists: files.length > 0, raw: rawRoot, packages, dependencies, scripts, packageManager, packageFiles };
}

function scopedPackage(pkg, packageFile) {
  return {
    exists: true,
    raw: packageFile.raw ?? {},
    packages: packageFile.packages ?? new Set(),
    dependencies: packageFile.dependencies ?? {},
    scripts: packageFile.scripts ?? {},
    packageManager: packageFile.packageManager ?? pkg.packageManager,
    packageFiles: [packageFile],
    allPackageFiles: pkg.packageFiles,
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

function lineForIndex(text, index) {
  return text.slice(0, index).split('\n').length;
}

function excerptForLine(lines, lineNumber) {
  return (lines[lineNumber - 1] ?? '').trim().slice(0, 240);
}

function isIgnored(config, ruleId, relativePath, lines, lineNumber) {
  if (config.ignoreRules.includes(ruleId)) return true;
  if (config.ignorePaths.some((ignored) => relativePath.includes(ignored))) {
    return true;
  }
  if (
    config.ignores.some(
      (entry) =>
        entry?.ruleId === ruleId &&
        typeof entry.path === 'string' &&
        entry.path.length > 0 &&
        relativePath.includes(entry.path),
    )
  ) {
    return true;
  }
  const nearby = [lines[lineNumber - 1], lines[lineNumber - 2]]
    .filter(Boolean)
    .join('\n');
  return nearby.includes('motion-audit-ignore all') ||
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

function makeFileFinding({
  ruleId,
  severity,
  confidence,
  category,
  file,
  line = 1,
  excerpt,
  rationale,
  recommendation,
}) {
  return {
    id: `${ruleId}:${file}:${line}`,
    ruleId,
    severity,
    confidence,
    category,
    file,
    line,
    excerpt,
    rationale,
    recommendation,
  };
}

function makePackageFinding(ruleId, severity, confidence, rationale, recommendation, excerpt, file = 'package.json') {
  return makeFileFinding({
    ruleId,
    severity,
    confidence,
    category: 'compatibility',
    file,
    excerpt,
    rationale,
    recommendation,
  });
}

function ruleRegex(pattern) {
  return new RegExp(pattern, 'gms');
}

function ruleAppliesToFile(rule, relativePath) {
  if (!Array.isArray(rule.files) || rule.files.length === 0) return true;
  return rule.files.some((fileName) =>
    relativePath === fileName ||
    path.basename(relativePath) === fileName ||
    relativePath.endsWith(`/${fileName}`),
  );
}

function scanRule(rule, file, root, text, config, packageFiles = []) {
  const relativePath = path.relative(root, file);
  if (Number.isFinite(rule.whenReanimatedMajor)) {
    const owner = owningPackage(relativePath, packageFiles);
    const reanimatedVersion = owner?.dependencies?.['react-native-reanimated'];
    if (dependencyMajor(reanimatedVersion) !== rule.whenReanimatedMajor) return [];
  }
  if (!ruleAppliesToFile(rule, relativePath)) return [];
  const lines = text.split('\n');
  const findings = [];
  if (
    rule.kind === 'fileContainsAny' ||
    rule.kind === 'fileContainsWithout' ||
    rule.kind === 'fileContainsBoth' ||
    rule.kind === 'fileContainsBothWithout'
  ) {
    const includeMatch = ruleRegex(rule.include).exec(text);
    const alsoMatch = rule.also ? ruleRegex(rule.also).exec(text) : null;
    const withoutMatch = rule.without ? ruleRegex(rule.without).exec(text) : null;
    const matches =
      rule.kind === 'fileContainsAny'
        ? includeMatch
        : rule.kind === 'fileContainsBoth'
          ? includeMatch && alsoMatch
          : rule.kind === 'fileContainsBothWithout'
            ? includeMatch && alsoMatch && !withoutMatch
            : includeMatch && (!rule.also || alsoMatch) && !withoutMatch;
    if (matches) {
      const line = lineForIndex(text, includeMatch.index);
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

function packageVersion(pkg, name) {
  const value = pkg.dependencies?.[name];
  return typeof value === 'string' ? value : null;
}

function dependencyMajor(version) {
  if (!version) return null;
  const match = version.match(/(?:npm:)?[~^<>=\s]*(\d+)/);
  return match ? Number(match[1]) : null;
}

function dependencyMajorMinor(version) {
  if (!version) return null;
  const match = version.match(/(?:npm:)?[~^<>=\s]*(\d+)\.(\d+)(?:\.|$)/);
  if (!match) return null;
  return `${Number(match[1])}.${Number(match[2])}`;
}

const reanimatedWorkletsCompatibility = {
  '4.0': new Set(['0.4']),
  '4.1': new Set(['0.5', '0.6', '0.7', '0.8']),
  '4.2': new Set(['0.7', '0.8']),
  '4.3': new Set(['0.8']),
  '4.4': new Set(['0.9']),
  '4.5': new Set(['0.10']),
};

function workletsCompatibility(reanimatedVersion, workletsVersion) {
  if (dependencyMajor(reanimatedVersion) !== 4) return 'not-applicable';
  if (!workletsVersion) return 'missing';
  const reanimated = dependencyMajorMinor(reanimatedVersion);
  const worklets = dependencyMajorMinor(workletsVersion);
  const allowed = reanimatedWorkletsCompatibility[reanimated];
  if (!allowed || !worklets) return 'unknown';
  return allowed.has(worklets) ? 'compatible' : 'mismatch';
}

function packageManager(root, pkg) {
  return pkg.packageManager ??
    (fs.existsSync(path.join(root, 'bun.lock')) || fs.existsSync(path.join(root, 'bun.lockb'))
      ? 'bun'
      : fs.existsSync(path.join(root, 'pnpm-lock.yaml'))
        ? 'pnpm'
        : fs.existsSync(path.join(root, 'yarn.lock'))
          ? 'yarn'
          : fs.existsSync(path.join(root, 'package-lock.json'))
            ? 'npm'
            : 'unknown');
}

function fileNames(root, names) {
  return names.filter((name) => fs.existsSync(path.join(root, name)));
}

const nativeConfigNames = [
  'app.json',
  'app.config.js',
  'app.config.cjs',
  'app.config.mjs',
  'app.config.ts',
  'eas.json',
];

const appConfigNames = new Set([
  'app.json',
  'app.config.js',
  'app.config.cjs',
  'app.config.mjs',
  'app.config.ts',
]);

function relativeFilesByName(files, root, names) {
  const wanted = new Set(names);
  return files
    .map((file) => path.relative(root, file))
    .filter((relativePath) => wanted.has(path.basename(relativePath)));
}

function configFiles(root, files = []) {
  if (files.length > 0) return relativeFilesByName(files, root, nativeConfigNames);
  return fileNames(root, nativeConfigNames);
}

const babelConfigNames = [
  'babel.config.js',
  'babel.config.cjs',
  'babel.config.mjs',
];

const jestConfigNames = [
  'jest.config.js',
  'jest.config.cjs',
  'jest.config.mjs',
  'jest.config.ts',
  'jest.setup.js',
  'jest.setup.ts',
  'jest-setup.js',
  'jest-setup.ts',
];

function filesOwnedByPackage(files, root, pkg) {
  if (!pkg?.allPackageFiles && (pkg?.packageFiles?.length ?? 0) > 1) return files;
  const currentPackageFile = pkg?.packageFiles?.[0]?.file;
  const allPackageFiles = pkg?.allPackageFiles ?? pkg?.packageFiles ?? [];
  if (!currentPackageFile || allPackageFiles.length === 0) return files;
  return files.filter((file) => owningPackage(path.relative(root, file), allPackageFiles)?.file === currentPackageFile);
}

function babelFiles(root, files = [], pkg = null) {
  if (files.length > 0) {
    return [...new Set([
      ...fileNames(root, babelConfigNames),
      ...relativeFilesByName(filesOwnedByPackage(files, root, pkg), root, babelConfigNames),
    ])];
  }
  return fileNames(root, babelConfigNames);
}

function jestFiles(root, files = [], pkg = null) {
  if (files.length > 0) return relativeFilesByName(filesOwnedByPackage(files, root, pkg), root, jestConfigNames);
  return fileNames(root, jestConfigNames);
}

function jestSetupFiles(root, files = [], pkg = null) {
  return jestFiles(root, files, pkg).filter((file) =>
    /(?:^|[.-])setup\.(?:js|ts|mjs|cjs)$/.test(path.basename(file)) ||
    /^jest-setup\.(?:js|ts|mjs|cjs)$/.test(path.basename(file)),
  );
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
      if (escaped) {
        escaped = false;
      } else if (char === '\\') {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
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

function containsReanimatedSetUpTests(text) {
  const activeText = maskJsComments(text);
  return /require\(['"]react-native-reanimated['"]\)\.setUpTests\(/.test(activeText) ||
    /from ['"]react-native-reanimated['"][\s\S]{0,240}\bsetUpTests\(/.test(activeText);
}

function projectFacts(root, pkg, files = []) {
  const versions = {
    expo: packageVersion(pkg, 'expo'),
    reactNative: packageVersion(pkg, 'react-native'),
    reanimated: packageVersion(pkg, 'react-native-reanimated'),
    worklets: packageVersion(pkg, 'react-native-worklets'),
    nativewind: packageVersion(pkg, 'nativewind'),
    reactNativeCss: packageVersion(pkg, 'react-native-css'),
    skia: packageVersion(pkg, '@shopify/react-native-skia'),
    rive: packageVersion(pkg, '@rive-app/react-native'),
    lottie: packageVersion(pkg, 'lottie-react-native'),
    expoGl: packageVersion(pkg, 'expo-gl'),
    expoDevClient: packageVersion(pkg, 'expo-dev-client'),
    expoBuildProperties: packageVersion(pkg, 'expo-build-properties'),
  };
  const scriptText = Object.values(pkg.scripts ?? {}).join('\n');
  const matchingNativePackages = nativeMotionPackages.filter((name) => pkg.packages.has(name));
  const packageFiles = filesOwnedByPackage(files, root, pkg);
  const jestFileList = jestFiles(root, files, pkg);
  const hasJest =
    pkg.packages.has('jest') ||
    pkg.packages.has('@jest/globals') ||
    /\bjest\b/.test(scriptText) ||
    jestFileList.length > 0;
  const packageSourceText = packageFiles
    .filter((file) => /\.(js|jsx|ts|tsx|mjs|cjs)$/.test(file))
    .map((file) => {
      try {
        return fs.readFileSync(file, 'utf8');
      } catch {
        return '';
      }
    })
    .join('\n');
  const setupText = jestSetupFiles(root, files, pkg)
    .map((file) => readTextIfExists(path.join(root, file)))
    .join('\n');
  return {
    versions,
    packageManager: packageManager(root, pkg),
    nativeMotionPackages: matchingNativePackages,
    hasExpo: Boolean(versions.expo),
    hasReactNative: Boolean(versions.reactNative),
    hasNativeDirs: fs.existsSync(path.join(root, 'ios')) || fs.existsSync(path.join(root, 'android')),
    configFiles: configFiles(root, packageFiles),
    babelFiles: babelFiles(root, files, pkg),
    jestFiles: jestFileList,
    hasJest,
    hasReanimatedSetUpTests: containsReanimatedSetUpTests(setupText),
    hasReanimatedTests: /toHaveAnimatedStyle|toHaveAnimatedProps|getDefaultStyle|advanceTimersByTime|runAllTimers/.test(packageSourceText),
  };
}

function appConfigTexts(root, files, pkg) {
  const currentPackageFile = pkg?.packageFiles?.[0]?.file;
  const allPackageFiles = pkg?.allPackageFiles ?? pkg?.packageFiles ?? [];
  return configFiles(root, files)
    .filter((file) => appConfigNames.has(path.basename(file)))
    .filter((file) => {
      if (!currentPackageFile || allPackageFiles.length === 0) return true;
      return owningPackage(file, allPackageFiles)?.file === currentPackageFile;
    })
    .map((file) => ({ file, text: readTextIfExists(path.join(root, file)) }))
    .filter((entry) => entry.text);
}

function appConfigHasNewArchFalse(root, files, pkg) {
  for (const entry of appConfigTexts(root, files, pkg)) {
    if (/"newArchEnabled"\s*:\s*false/.test(entry.text) || /newArchEnabled\s*:\s*false/.test(entry.text)) {
      const line = lineForIndex(entry.text, entry.text.search(/newArchEnabled/));
      return { file: entry.file, line, excerpt: excerptForLine(entry.text.split('\n'), line) };
    }
  }
  return null;
}

function babelPluginNames(text) {
  const activeText = maskJsComments(text);
  const match = /plugins\s*:\s*\[/.exec(activeText);
  if (!match) return [];
  let index = match.index + match[0].length;
  let depth = 1;
  let quote = null;
  let escaped = false;
  for (; index < activeText.length; index += 1) {
    const char = activeText[index];
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === '\\') {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
    } else if (char === '[') {
      depth += 1;
    } else if (char === ']') {
      depth -= 1;
      if (depth === 0) break;
    }
  }
  const segment = activeText.slice(match.index + match[0].length, index);
  const names = [];
  let itemDepth = 0;
  for (let cursor = 0; cursor < segment.length; cursor += 1) {
    const char = segment[cursor];
    if (char === '[' || char === '{' || char === '(') {
      itemDepth += 1;
      continue;
    }
    if (char === ']' || char === '}' || char === ')') {
      itemDepth = Math.max(0, itemDepth - 1);
      continue;
    }
    if (char !== '"' && char !== "'" && char !== '`') continue;
    let end = cursor + 1;
    let isEscaped = false;
    for (; end < segment.length; end += 1) {
      if (isEscaped) isEscaped = false;
      else if (segment[end] === '\\') isEscaped = true;
      else if (segment[end] === char) break;
    }
    if (end >= segment.length) break;
    const before = segment.slice(0, cursor).trimEnd();
    const previous = before.at(-1);
    const isRequireResolvePlugin =
      /require\.resolve\s*\($/.test(before) &&
      (itemDepth === 1 || (itemDepth === 2 && previousContainer(segment, cursor) === '['));
    if (itemDepth === 0 || (itemDepth === 1 && previous === '[') || isRequireResolvePlugin) {
      names.push(segment.slice(cursor + 1, end));
    }
    cursor = end;
  }
  return names;
}

function previousContainer(text, endIndex) {
  const stack = [];
  let quote = null;
  let escaped = false;
  for (let index = 0; index < endIndex; index += 1) {
    const char = text[index];
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === '\\') {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
    } else if (char === '[' || char === '{' || char === '(') {
      stack.push(char);
    } else if (char === ']' || char === '}' || char === ')') {
      stack.pop();
    }
  }
  return stack.at(-2) ?? null;
}

function findSetUpTestsFile(root, files, pkg = null) {
  for (const file of jestSetupFiles(root, files, pkg)) {
    const text = readTextIfExists(path.join(root, file));
    if (containsReanimatedSetUpTests(text)) {
      return file;
    }
  }
  return null;
}

function packageFileForDependency(pkg, names) {
  return pkg.packageFiles?.find((entry) => names.some((name) => entry.packages.has(name)))?.file ?? 'package.json';
}

function scanPackageCompatibility(root, pkg, config, files) {
  const findings = [];
  const packageLines = [''];
  const facts = projectFacts(root, pkg, files);
  const packageFile = packageFileForDependency(pkg, [
    'react-native-reanimated',
    'react-native-worklets',
    'expo',
    'react-native',
    'nativewind',
    'react-native-css',
  ]);
  const reanimatedVersion = packageVersion(pkg, 'react-native-reanimated');
  const workletsVersion = packageVersion(pkg, 'react-native-worklets');
  const expoVersion = packageVersion(pkg, 'expo');
  const reactNativeVersion = packageVersion(pkg, 'react-native');
  const reanimatedMajor = dependencyMajor(reanimatedVersion);
  const expoMajor = dependencyMajor(expoVersion);
  const reactNativeMinor = reactNativeVersion?.match(/(?:npm:)?[~^<>=\s]*0\.(\d+)/)?.[1];

  if (
    reanimatedMajor === 4 &&
    !workletsVersion &&
    !isIgnored(config, 'native.reanimated4-missing-worklets', packageFile, packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.reanimated4-missing-worklets',
      'high',
      'high',
      'react-native-reanimated is pinned to major 4 but react-native-worklets is not declared.',
      'Install the Expo-compatible react-native-worklets version or verify the repo intentionally resolves it elsewhere before accepting Reanimated 4 proof.',
      `react-native-reanimated: ${reanimatedVersion}`,
      packageFile,
    ));
  }

  const compatibility = workletsCompatibility(reanimatedVersion, workletsVersion);
  if (
    compatibility === 'mismatch' &&
    !isIgnored(config, 'native.reanimated4-worklets-version-mismatch', packageFile, packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.reanimated4-worklets-version-mismatch',
      'high',
      'high',
      'react-native-reanimated and react-native-worklets versions are not in a known compatible minor pairing.',
      'Install the react-native-worklets minor that matches the Reanimated minor before accepting native validation proof.',
      `react-native-reanimated: ${reanimatedVersion}; react-native-worklets: ${workletsVersion}`,
      packageFile,
    ));
  } else if (
    compatibility === 'unknown' &&
    !isIgnored(config, 'native.reanimated4-worklets-version-unknown', packageFile, packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.reanimated4-worklets-version-unknown',
      'medium',
      'medium',
      'react-native-reanimated and react-native-worklets versions are outside the scanner compatibility table.',
      'Verify the pair against the installed package release notes before accepting native validation proof.',
      `react-native-reanimated: ${reanimatedVersion}; react-native-worklets: ${workletsVersion}`,
      packageFile,
    ));
  }

  if (
    workletsVersion &&
    reanimatedVersion &&
    reanimatedMajor !== null &&
    reanimatedMajor < 4 &&
    !isIgnored(config, 'native.worklets-with-reanimated3', packageFile, packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.worklets-with-reanimated3',
      'medium',
      'medium',
      'react-native-worklets is declared with a Reanimated major that may not own the worklets package boundary.',
      'Verify whether the repo is migrating to Reanimated 4; otherwise remove unnecessary worklets setup or document the package-specific reason.',
      `react-native-reanimated: ${reanimatedVersion}; react-native-worklets: ${workletsVersion}`,
      packageFile,
    ));
  }

  const staleNewArch = appConfigHasNewArchFalse(root, files, pkg);
  if (staleNewArch && expoMajor !== null && expoMajor >= 55) {
    if (!isIgnored(config, 'native.sdk55-newarch-disabled', staleNewArch.file, [''], staleNewArch.line)) {
      findings.push(makeFileFinding({
        ruleId: 'native.sdk55-newarch-disabled',
        severity: 'high',
        confidence: 'high',
        category: 'compatibility',
        file: staleNewArch.file,
        line: staleNewArch.line,
        excerpt: staleNewArch.excerpt,
        rationale: 'Expo SDK 55 and later always run on the New Architecture; disabling it is stale config.',
        recommendation: 'Remove newArchEnabled: false and validate native packages with Expo Doctor plus native build or EAS proof.',
      }));
    }
  } else if (staleNewArch && expoMajor !== null && expoMajor >= 53) {
    if (!isIgnored(config, 'native.sdk53-newarch-disabled-needs-proof', staleNewArch.file, [''], staleNewArch.line)) {
      findings.push(makeFileFinding({
        ruleId: 'native.sdk53-newarch-disabled-needs-proof',
        severity: 'medium',
        confidence: 'medium',
        category: 'compatibility',
        file: staleNewArch.file,
        line: staleNewArch.line,
        excerpt: staleNewArch.excerpt,
        rationale: 'Expo SDK 53 and SDK 54 enable the New Architecture by default; disabling it needs explicit compatibility rationale.',
        recommendation: 'Document the package-specific reason, run Expo Doctor, and plan migration proof before SDK 55.',
      }));
    }
  }

  if (staleNewArch && reactNativeMinor && Number(reactNativeMinor) >= 82) {
    if (!isIgnored(config, 'native.rn082-newarch-disabled', staleNewArch.file, [''], staleNewArch.line)) {
      findings.push(makeFileFinding({
        ruleId: 'native.rn082-newarch-disabled',
        severity: 'high',
        confidence: 'medium',
        category: 'compatibility',
        file: staleNewArch.file,
        line: staleNewArch.line,
        excerpt: staleNewArch.excerpt,
        rationale: 'React Native 0.82 and later run entirely on the New Architecture.',
        recommendation: 'Remove stale legacy-architecture config and validate native modules on the affected platforms.',
      }));
    }
  }

  const directoryCheck = pkg.raw?.expo?.doctor?.reactNativeDirectoryCheck;
  if (
    directoryCheck?.enabled === false &&
    facts.nativeMotionPackages.length > 0 &&
    !isIgnored(config, 'native.rn-directory-check-disabled', packageFile, packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.rn-directory-check-disabled',
      'medium',
      'high',
      'Expo Doctor React Native Directory checks are disabled while native motion packages are declared.',
      'Re-enable the check or document package-specific exclusions before accepting New Architecture compatibility proof.',
      'expo.doctor.reactNativeDirectoryCheck.enabled: false',
      packageFile,
    ));
  }

  const installExcludes = pkg.raw?.expo?.install?.exclude;
  const excludedNativePackages = Array.isArray(installExcludes)
    ? installExcludes.filter((name) => nativeMotionPackages.includes(name))
    : [];
  if (
    excludedNativePackages.length > 0 &&
    !isIgnored(config, 'native.expo-install-excludes-motion-package', packageFile, packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.expo-install-excludes-motion-package',
      'medium',
      'medium',
      'Expo install version validation excludes native motion packages.',
      'Verify the repo has a package-specific compatibility reason and add native build proof for excluded packages.',
      `excluded packages: ${excludedNativePackages.join(', ')}`,
      packageFile,
    ));
  }

  for (const babelFile of facts.babelFiles) {
    const full = path.join(root, babelFile);
    const text = readTextIfExists(full);
    const plugins = babelPluginNames(text);
    const lines = text.split('\n');
    const workletsIndex = plugins.lastIndexOf('react-native-worklets/plugin');
    const lastPlugin = plugins.at(-1);
    if (
      reanimatedMajor === 4 &&
      plugins.includes('react-native-reanimated/plugin') &&
      !isIgnored(config, 'native.reanimated4-old-babel-plugin', babelFile, lines, 1)
    ) {
      findings.push(makeFileFinding({
        ruleId: 'native.reanimated4-old-babel-plugin',
        severity: 'high',
        confidence: 'high',
        category: 'compatibility',
        file: babelFile,
        excerpt: 'react-native-reanimated/plugin is present in a Reanimated 4 config',
        rationale: 'Reanimated 4 projects using explicit Babel config should use the worklets plugin boundary.',
        recommendation: 'Replace the old plugin with react-native-worklets/plugin as the last Babel plugin unless Expo/babel-preset-expo owns the config.',
      }));
    }
    if (
      reanimatedMajor === 4 &&
      !facts.hasExpo &&
      workletsIndex < 0 &&
      !isIgnored(config, 'native.reanimated4-missing-worklets-babel-plugin', babelFile, lines, 1)
    ) {
      findings.push(makeFileFinding({
        ruleId: 'native.reanimated4-missing-worklets-babel-plugin',
        severity: 'high',
        confidence: 'high',
        category: 'compatibility',
        file: babelFile,
        excerpt: 'explicit Babel config without react-native-worklets/plugin',
        rationale: 'React Native CLI apps with Reanimated 4 need the worklets Babel plugin.',
        recommendation: 'Add react-native-worklets/plugin as the final Babel plugin.',
      }));
    }
    if (
      workletsIndex >= 0 &&
      lastPlugin !== 'react-native-worklets/plugin' &&
      !isIgnored(config, 'native.worklets-babel-plugin-not-last', babelFile, lines, 1)
    ) {
      findings.push(makeFileFinding({
        ruleId: 'native.worklets-babel-plugin-not-last',
        severity: 'high',
        confidence: 'medium',
        category: 'compatibility',
        file: babelFile,
        excerpt: `plugin order: ${plugins.join(' -> ')}`,
        rationale: 'The worklets Babel plugin must be listed last.',
        recommendation: 'Move react-native-worklets/plugin to the end of the Babel plugins array and rerun tests/native smoke.',
      }));
    }
  }

  if (
    reanimatedVersion &&
    facts.hasJest &&
    facts.hasReanimatedTests &&
    !findSetUpTestsFile(root, files, pkg) &&
    !isIgnored(config, 'native.reanimated-jest-setup-missing', 'package.json', packageLines, 1)
  ) {
    findings.push(makePackageFinding(
      'native.reanimated-jest-setup-missing',
      'medium',
      'medium',
      'Reanimated animation tests appear to be present, but no Reanimated setUpTests call was found.',
      'Add require(\"react-native-reanimated\").setUpTests() to the Jest setup file and use fake timers for time-based assertions.',
      'Reanimated tests detected without setUpTests()',
      packageFile,
    ));
  }

  return findings;
}

function scan(root, maxFiles) {
  if (!fs.existsSync(root)) throw new Error(`Root does not exist: ${root}`);
  const config = loadConfig(root);
  const files = listFiles(root, maxFiles);
  const pkg = readPackage(root);
  const facts = projectFacts(root, pkg, files);
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
      findings.push(...scanRule(rule, file, root, text, config, pkg.packageFiles));
    }
  }
  for (const packageFile of pkg.packageFiles) {
    findings.push(...scanPackageCompatibility(root, scopedPackage(pkg, packageFile), config, files));
  }
  const unique = uniqueFindings(findings);
  return {
    ok: !unique.some((finding) => finding.severity === 'high'),
    profile: profile.skillName,
    root,
    scannedFiles: files.length,
    rules: profile.rules.length + 8,
    facts,
    findings: unique,
    summary: severities.reduce((acc, severity) => {
      acc[severity] = unique.filter((finding) => finding.severity === severity).length;
      return acc;
    }, {}),
  };
}

function recommendedChecks(facts) {
  const checks = [];
  if (facts.hasExpo) {
    checks.push('repo wrapper for expo install --check');
    if (facts.nativeMotionPackages.length > 0 || facts.configFiles.some((file) => appConfigNames.has(path.basename(file)))) {
      checks.push('repo wrapper for Expo Doctor');
    }
  }
  if (facts.configFiles.length > 0) checks.push('expo config --type prebuild when config/native settings changed');
  if (facts.nativeMotionPackages.length > 0) checks.push('simulator/device or native build proof on affected platforms');
  if (facts.hasJest && facts.versions.reanimated) checks.push('focused Reanimated/Jest tests with setUpTests and fake timers');
  return checks;
}

function doctor(root, maxFiles) {
  const rootExists = fs.existsSync(root);
  const pkg = readPackage(root);
  const files = rootExists ? listFiles(root, maxFiles) : [];
  const facts = projectFacts(root, pkg, files);
  return {
    ok: rootExists,
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    packageManager: facts.packageManager,
    configuredRules: profile.rules.length + 8,
    sampleFileCount: files.length,
    configFile: rootExists && fs.existsSync(path.join(root, '.motion-audit.json')),
    facts,
    recommendedChecks: recommendedChecks(facts),
    notes: [
      'scan is read-only',
      'use --format json for machine-readable output',
      'findings can be suppressed with .motion-audit.json or inline motion-audit-ignore comments',
      'latest npm pins are provenance only; target repo lockfile and Expo SDK compatibility remain authoritative',
    ],
  };
}

function formatVersions(versions) {
  return Object.entries(versions)
    .filter(([, value]) => value)
    .map(([name, value]) => `${name}=${value}`)
    .join(', ') || '(none detected)';
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    return `# Motion Audit Doctor: ${result.profile}

- Root: ${result.root}
- Package JSON: ${result.packageJson ? 'yes' : 'no'}
- Package manager: ${result.packageManager}
- Config file: ${result.configFile ? 'yes' : 'no'}
- Configured rules: ${result.configuredRules}
- Sample file count: ${result.sampleFileCount}
- Status: ${result.ok ? 'ok' : 'failed'}
- Versions: ${formatVersions(result.facts.versions)}
- Native motion packages: ${result.facts.nativeMotionPackages.join(', ') || '(none detected)'}
- Config surfaces: ${result.facts.configFiles.join(', ') || '(none detected)'}
- Babel files: ${result.facts.babelFiles.join(', ') || '(none detected)'}
- Jest files: ${result.facts.jestFiles.join(', ') || '(none detected)'}
- Recommended checks: ${result.recommendedChecks.join('; ') || '(none)'}
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
- Versions: ${formatVersions(result.facts.versions)}
- Native motion packages: ${result.facts.nativeMotionPackages.join(', ') || '(none detected)'}

${findings || 'No findings.'}
`;
}

function emit(result, args) {
  const body = args.format === 'json'
    ? `${JSON.stringify(result, null, 2)}\n`
    : renderMarkdown(result);
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
  const result = args.command === 'doctor'
    ? doctor(args.root, args.maxFiles)
    : scan(args.root, args.maxFiles);
  emit(result, args);
  process.exit(result.ok ? 0 : 2);
} catch (error) {
  const payload = { ok: false, profile: profile.skillName, error: error.message };
  const wantsJson = process.argv.includes('--json') ||
    (process.argv.includes('--format') && process.argv.includes('json'));
  if (wantsJson) process.stdout.write(`${JSON.stringify(payload, null, 2)}\n`);
  else {
    console.error(error.message);
    console.error(usage());
  }
  process.exit(1);
}
