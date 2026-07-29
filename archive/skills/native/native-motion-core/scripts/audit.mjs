#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "native-motion-core",
  "rules": [
    {
      "id": "native.runonjs-reanimated4",
      "severity": "medium",
      "confidence": "medium",
      "category": "compatibility",
      "whenReanimatedMajor": 4,
      "pattern": "\\brunOnJS\\s*\\(",
      "message": "runOnJS is present; Reanimated 4 guidance favors scheduleOnRN from react-native-worklets.",
      "recommendation": "Verify installed Reanimated/worklets version and migrate to scheduleOnRN when on Reanimated 4."
    },
    {
      "id": "native.reanimated4-missing-worklets",
      "severity": "high",
      "confidence": "high",
      "category": "compatibility",
      "kind": "packageDependency",
      "whenPackage": "react-native-reanimated",
      "whenMajor": 4,
      "missingPackage": "react-native-worklets",
      "message": "A package.json uses Reanimated 4 without react-native-worklets.",
      "recommendation": "Install a compatible react-native-worklets version and rebuild native dependencies."
    },
    {
      "id": "native.reanimated4-worklets-version-mismatch",
      "severity": "high",
      "confidence": "high",
      "category": "compatibility",
      "kind": "reanimatedWorkletsCompatibility",
      "message": "A package.json combines Reanimated 4 with an incompatible react-native-worklets minor version.",
      "recommendation": "Use the compatibility table for the installed Reanimated minor. Expo SDK 56's Reanimated 4.3.x baseline expects react-native-worklets 0.8.x."
    },
    {
      "id": "native.reanimated3-with-worklets",
      "severity": "high",
      "confidence": "high",
      "category": "compatibility",
      "kind": "packageDependency",
      "whenPackage": "react-native-reanimated",
      "whenMajor": 3,
      "presentPackage": "react-native-worklets",
      "message": "A package.json combines Reanimated 3 with react-native-worklets.",
      "recommendation": "Remove react-native-worklets for Reanimated 3 or migrate the app to Reanimated 4/New Architecture."
    },
    {
      "id": "native.old-reanimated-babel-plugin",
      "severity": "high",
      "confidence": "high",
      "category": "compatibility",
      "whenReanimatedMajor": 4,
      "pattern": "react-native-reanimated/plugin",
      "message": "The old Reanimated Babel plugin path is present.",
      "recommendation": "For Reanimated 4, use react-native-worklets/plugin as the last Babel plugin."
    },
    {
      "id": "native.removed-gesture-handler-api",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "whenReanimatedMajor": 4,
      "pattern": "\\buseAnimatedGestureHandler\\b",
      "message": "useAnimatedGestureHandler is present; it was removed in Reanimated 4.",
      "recommendation": "Use Gesture Handler 2's Gesture API when migrating to Reanimated 4."
    },
    {
      "id": "native.deprecated-worklet-threading-api",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "whenReanimatedMajor": 4,
      "pattern": "\\b(runOnUI|runOnRuntime|executeOnUIRuntimeSync)\\s*\\(",
      "message": "Deprecated worklet threading API is present.",
      "recommendation": "For Reanimated 4, migrate to scheduleOnUI, scheduleOnRuntime, or runOnUISync from react-native-worklets."
    },
    {
      "id": "native.deprecated-scroll-offset-api",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "whenReanimatedMajor": 4,
      "pattern": "\\buseScrollViewOffset\\b",
      "message": "useScrollViewOffset is present; Reanimated 4 renamed it to useScrollOffset.",
      "recommendation": "Use useScrollOffset when the installed Reanimated version supports it."
    },
    {
      "id": "native.reanimated3-spring-threshold",
      "severity": "medium",
      "confidence": "medium",
      "category": "migration",
      "whenReanimatedMajor": 4,
      "pattern": "\\b(restDisplacementThreshold|restSpeedThreshold)\\b",
      "message": "Reanimated 3 spring threshold config is present.",
      "recommendation": "For Reanimated 4, re-check withSpring config and migrate to energyThreshold where appropriate."
    },
    {
      "id": "native.scheduleonrn-inline-callback",
      "severity": "medium",
      "confidence": "medium",
      "category": "worklets",
      "whenReanimatedMajor": 4,
      "pattern": "\\bscheduleOnRN\\s*\\(\\s*(?:async\\s*)?(?:function\\b|\\([^)]*\\)\\s*=>|[A-Za-z_$][\\w$]*\\s*=>)",
      "message": "scheduleOnRN appears to receive a function created inline near a worklet call.",
      "recommendation": "Define functions passed to scheduleOnRN in RN-runtime scope, such as component or module scope."
    },
    {
      "id": "native.transition-property-all",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "transitionProperty\\s*:\\s*['\"]all['\"]",
      "message": "CSS transitionProperty is set to all near motion code.",
      "recommendation": "List the exact properties being transitioned to avoid unnecessary per-frame work."
    },
    {
      "id": "native.repeat-no-cancel",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "withRepeat\\(",
      "without": "cancelAnimation|useReducedMotion|ReducedMotion|AccessibilityInfo|cleanup|return\\s*\\(\\s*\\)\\s*=>",
      "message": "Repeating native animation lacks an obvious cancel or reduced-motion branch.",
      "recommendation": "Cancel repeaters on unmount/state change and reduce decorative loops for reduced motion."
    },
    {
      "id": "native.shared-value-js-read",
      "severity": "medium",
      "confidence": "low",
      "category": "performance",
      "requiresText": "react-native-reanimated|useSharedValue|SharedValue|useAnimatedStyle|useAnimatedReaction",
      "pattern": "[^A-Za-z0-9_]([A-Za-z0-9_]+)\\.value",
      "message": "Shared value reads on the JS thread can block or desynchronize if used outside worklets.",
      "recommendation": "Confirm this read is inside a worklet; otherwise derive and consume on the UI thread."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "requiresText": "react-native-reanimated|useAnimatedStyle|withTiming|withSpring|transitionProperty|Animated\\.",
      "pattern": "\\b(width|height|top|left|right|bottom|margin|padding)\\s*:",
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
const fileExtensions = new Set([
  '.js', '.jsx', '.ts', '.tsx', '.mjs', '.cjs', '.css', '.scss', '.sass',
  '.html', '.vue', '.svelte', '.json',
]);
const skipFiles = new Set(['scripts/audit.mjs']);
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

Package checks:
  scan inspects package.json files under --root for Reanimated/worklets
  compatibility and reports the package file that needs review.
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
        if (!skipFiles.has(rel)) files.push(full);
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

function parsePackageFile(file, root) {
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = dependencyMap(pkg);
    return {
      file,
      relativePath: path.relative(root, file),
      exists: true,
      packages: new Set(Object.keys(deps ?? {})),
      versions: deps ?? {},
      scripts: pkg.scripts ?? {},
    };
  } catch {
    return {
      file,
      relativePath: path.relative(root, file),
      exists: true,
      packages: new Set(),
      versions: {},
      scripts: {},
    };
  }
}

function readPackage(root) {
  const file = path.join(root, 'package.json');
  if (!fs.existsSync(file)) return { exists: false, packages: new Set() };
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = dependencyMap(pkg);
    return { exists: true, packages: new Set(Object.keys(deps ?? {})), scripts: pkg.scripts ?? {} };
  } catch {
    return { exists: true, packages: new Set(), scripts: {} };
  }
}

function packageMajor(version) {
  if (typeof version !== 'string') return null;
  const match = version.match(/(?:^|[^\d])(\d+)(?:\.|$)/);
  return match ? Number(match[1]) : null;
}

function packageMajorMinor(version) {
  if (typeof version !== 'string') return null;
  const match = version.match(/(?:^|[^\d])(\d+)\.(\d+)(?:\.|$)/);
  if (!match) return null;
  return `${Number(match[1])}.${Number(match[2])}`;
}

function hasPackage(packageInfo, packageName) {
  return packageInfo?.packages?.has(packageName) ?? false;
}

function reanimatedMajor(packageInfo) {
  return packageMajor(packageInfo?.versions?.['react-native-reanimated']);
}

function hasReanimated(packageInfo) {
  return hasPackage(packageInfo, 'react-native-reanimated');
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
  const reanimated = packageMajorMinor(reanimatedVersion);
  const worklets = packageMajorMinor(workletsVersion);
  if (!reanimatedVersion) return 'not-applicable';
  if (packageMajor(reanimatedVersion) !== 4) return 'not-applicable';
  if (!workletsVersion) return 'missing';
  const allowed = reanimatedWorkletsCompatibility[reanimated];
  if (!allowed || !worklets) return 'unknown';
  return allowed.has(worklets) ? 'compatible' : 'mismatch';
}

function packageInfosForContext(file, root, packageInfos) {
  const relativePath = path.relative(root, file);
  const containing = packageInfos
    .filter((packageInfo) => {
      const packageDir = path.dirname(packageInfo.file);
      return file === packageInfo.file || file.startsWith(packageDir + path.sep);
    })
    .sort((a, b) => path.dirname(b.file).length - path.dirname(a.file).length);
  const nearest = containing[0];
  if (nearest && hasReanimated(nearest)) return [nearest];
  const isRootMotionConfig = /(^|[\\/])(babel|metro)\.config\.[cm]?js$/.test(relativePath);
  if (isRootMotionConfig) {
    const reanimatedPackages = packageInfos.filter(hasReanimated);
    return reanimatedPackages.length > 0 ? reanimatedPackages : nearest ? [nearest] : [];
  }
  return nearest ? [nearest] : packageInfos.length === 1 ? packageInfos : [];
}

function ruleAppliesInContext(rule, contextPackageInfos, text) {
  if (rule.requiresText && !ruleRegex(rule.requiresText).test(text)) return false;
  if (Number.isFinite(rule.whenReanimatedMajor)) {
    return contextPackageInfos.some((packageInfo) => reanimatedMajor(packageInfo) === rule.whenReanimatedMajor);
  }
  return true;
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

function previousContainer(text, endIndex) {
  const stack = [];
  let quote = null;
  let escaped = false;
  for (let index = 0; index < endIndex; index += 1) {
    const char = text[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') quote = char;
    else if (char === '[' || char === '{' || char === '(') stack.push(char);
    else if (char === ']' || char === '}' || char === ')') stack.pop();
  }
  return stack.at(-2) ?? null;
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
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') quote = char;
    else if (char === '[') depth += 1;
    else if (char === ']') {
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

function scanRule(rule, file, root, text, config, packageInfos) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  const contextPackageInfos = packageInfosForContext(file, root, packageInfos);
  if (!ruleAppliesInContext(rule, contextPackageInfos, text)) return findings;
  if (rule.id === 'native.old-reanimated-babel-plugin') {
    if (!/^babel\.config\.(?:js|cjs|mjs|ts)$/.test(path.basename(relativePath))) return findings;
    if (!babelPluginNames(text).includes('react-native-reanimated/plugin')) return findings;
    if (!isIgnored(config, rule.id, relativePath, lines, 1)) {
      findings.push(makeFinding(rule, relativePath, 1, 'react-native-reanimated/plugin is present in active Babel plugins'));
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

function scanPackageRule(rule, packageInfo, config) {
  const version = packageInfo.versions[rule.whenPackage];
  if (!version) return [];
  if (Number.isFinite(rule.whenMajor) && packageMajor(version) !== rule.whenMajor) {
    return [];
  }
  const missingMatched =
    rule.missingPackage && !packageInfo.packages.has(rule.missingPackage);
  const presentMatched =
    rule.presentPackage && packageInfo.packages.has(rule.presentPackage);
  if (!missingMatched && !presentMatched) return [];
  if (isIgnored(config, rule.id, packageInfo.relativePath, [''], 1)) return [];
  const relation = missingMatched
    ? `missing package: ${rule.missingPackage}`
    : `present package: ${rule.presentPackage}`;
  return [
    makeFinding(
      rule,
      packageInfo.relativePath,
      1,
      `${rule.whenPackage}@${version}; ${relation}`,
    ),
  ];
}

function scanWorkletsCompatibilityRule(rule, packageInfo, config) {
  const reanimatedVersion = packageInfo.versions['react-native-reanimated'];
  if (packageMajor(reanimatedVersion) !== 4) return [];
  const workletsVersion = packageInfo.versions['react-native-worklets'];
  if (workletsCompatibility(reanimatedVersion, workletsVersion) !== 'mismatch') {
    return [];
  }
  if (isIgnored(config, rule.id, packageInfo.relativePath, [''], 1)) return [];
  return [
    makeFinding(
      rule,
      packageInfo.relativePath,
      1,
      `react-native-reanimated@${reanimatedVersion}; react-native-worklets@${workletsVersion}`,
    ),
  ];
}

function scan(root, maxFiles) {
  if (!fs.existsSync(root)) throw new Error(`Root does not exist: ${root}`);
  const config = loadConfig(root);
  const files = listFiles(root, maxFiles);
  const packageFiles = listPackageFiles(root, maxFiles).map((file) => parsePackageFile(file, root));
  const pkg = readPackage(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageDependency') {
      for (const packageInfo of packageFiles) {
        findings.push(...scanPackageRule(rule, packageInfo, config));
      }
      continue;
    }
    if (rule.kind === 'reanimatedWorkletsCompatibility') {
      for (const packageInfo of packageFiles) {
        findings.push(...scanWorkletsCompatibilityRule(rule, packageInfo, config));
      }
      continue;
    }
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
      findings.push(...scanRule(rule, file, root, text, config, packageFiles));
    }
  }
  return {
    ok: !findings.some((finding) => finding.severity === 'high'),
    profile: profile.skillName,
    root,
    scannedFiles: files.length,
    scannedPackageFiles: packageFiles.length,
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
  const packageFiles = fs.existsSync(root) ? listPackageFiles(root, maxFiles).map((file) => parsePackageFile(file, root)) : [];
  return {
    ok: fs.existsSync(root),
    profile: profile.skillName,
    root,
    packageJson: pkg.exists,
    packageFiles: packageFiles.length,
    reanimatedPackages: packageFiles
      .filter((packageInfo) => packageInfo.packages.has('react-native-reanimated'))
      .map((packageInfo) => ({
        file: packageInfo.relativePath,
        reanimated: packageInfo.versions['react-native-reanimated'],
        worklets: packageInfo.versions['react-native-worklets'] ?? null,
        workletsCompatibility: workletsCompatibility(
          packageInfo.versions['react-native-reanimated'],
          packageInfo.versions['react-native-worklets'],
        ),
      })),
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
- Package files: ${result.packageFiles}
- Reanimated packages: ${result.reanimatedPackages.length}
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
- Scanned package files: ${result.scannedPackageFiles}
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
