#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const profile = {
  "skillName": "native-three-r3f",
  "rules": [
    {
      "id": "r3f.set-state-in-frame",
      "severity": "high",
      "confidence": "medium",
      "category": "performance",
      "kind": "useFrameContains",
      "pattern": "(?<!\\.)\\bset[A-Z][A-Za-z0-9_]*\\(",
      "message": "React state appears to be updated from a React Three Fiber frame loop.",
      "recommendation": "Use refs or external transient state for frame-loop mutation."
    },
    {
      "id": "r3f.canvas-size",
      "severity": "medium",
      "confidence": "low",
      "category": "rendering",
      "pattern": "<" + "Canvas(?![^>]*(style|className|height|width|frameloop|dpr))",
      "message": "Canvas appears without obvious sizing or render-loop constraints.",
      "recommendation": "Ensure stable dimensions and consider DPR/frameloop settings for performance."
    },
    {
      "id": "r3f.loader-without-boundary",
      "severity": "medium",
      "confidence": "medium",
      "category": "asset",
      "kind": "fileContainsBothWithout",
      "include": "use(GLTF|Texture|Loader)\\s*\\(",
      "also": "<" + "Canvas|from\\s+['\"]@react-three/fiber/native['\"]",
      "without": "Suspense|ErrorBoundary|fallback\\s*=|onError|try\\s*\\{",
      "message": "A native R3F asset loader appears without an obvious loading/error boundary.",
      "recommendation": "Wrap model/texture loading in Suspense plus an app-owned error/fallback path outside the GL scene."
    },
    {
      "id": "r3f.loop-no-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsBothWithout",
      "include": "useFrame\\s*\\(|frameloop\\s*=\\s*['\"]always['\"]|AnimationMixer|LoopRepeat|autoRotate",
      "also": "<" + "Canvas|from\\s+['\"]@react-three/fiber/native['\"]",
      "without": "useReducedMotion|AccessibilityInfo|reduceMotion|ReducedMotion|fallback|static|poster",
      "message": "Continuous or animated native 3D usage lacks an obvious reduced-motion or non-3D fallback.",
      "recommendation": "Gate decorative motion with reduced-motion handling and provide a static/native-view fallback when the scene is product-critical."
    },
    {
      "id": "r3f.native-entrypoint",
      "severity": "high",
      "confidence": "medium",
      "category": "native",
      "pattern": "from\\s+['\"]@react-three/fiber['\"]",
      "message": "A native React Native file appears to import the web R3F entrypoint.",
      "recommendation": "Use @react-three/fiber/native for React Native scenes."
    },
    {
      "id": "r3f.drei-native-entrypoint",
      "severity": "medium",
      "confidence": "medium",
      "category": "native",
      "kind": "fileContainsBoth",
      "include": "from\\s+['\"]@react-three/drei['\"]",
      "also": "from\\s+['\"]react-native['\"]|from\\s+['\"]expo-gl['\"]|from\\s+['\"]@react-three/fiber/native['\"]",
      "message": "A native React Native file appears to import the web Drei entrypoint.",
      "recommendation": "Use @react-three/drei/native and verify the helper exists in the native route."
    },
    {
      "id": "r3f.drei-web-only-helper",
      "severity": "high",
      "confidence": "medium",
      "category": "native",
      "kind": "fileContainsBoth",
      "include": "from\\s+['\"]@react-three/drei/native['\"]",
      "also": "import\\s*\\{[^}]*\\b(Html|Loader)\\b[^}]*\\}\\s*from\\s+['\"]@react-three/drei/native['\"]",
      "message": "A native Drei import appears to use a helper that is web-only in Drei's web route.",
      "recommendation": "Replace the helper with React Native UI around the Canvas or verify a native-safe helper in the installed Drei version."
    },
    {
      "id": "r3f.native-missing-expo-gl",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "packageHasAllWithout",
      "packages": ["@react-three/fiber", "react-native"],
      "withoutPackages": ["expo-gl"],
      "message": "React Native R3F dependencies are present without expo-gl in package.json.",
      "recommendation": "Install/verify expo-gl for @react-three/fiber/native or document why this repo does not use the native Canvas."
    },
    {
      "id": "r3f.native-missing-expo-asset",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "packageHasAllWithout",
      "packages": ["@react-three/fiber", "react-native"],
      "withoutPackages": ["expo-asset"],
      "message": "React Native R3F dependencies are present without expo-asset in package.json.",
      "recommendation": "Install/verify expo-asset for native loader and Metro asset interop, or document why no native asset loading is used."
    },
    {
      "id": "r3f.drei-missing-core-peers",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "packageHasAnyWithout",
      "packages": ["@react-three/drei"],
      "withoutPackages": ["@react-three/fiber", "three"],
      "message": "Drei is present without an obvious @react-three/fiber and three dependency declaration.",
      "recommendation": "Declare compatible @react-three/fiber and three versions in the app package that owns the native scene."
    },
    {
      "id": "r3f.expo-three-legacy",
      "severity": "low",
      "confidence": "medium",
      "category": "setup",
      "kind": "packageHasAny",
      "packages": ["expo-three"],
      "message": "expo-three is present and should be treated as a legacy/manual Expo GL bridge unless deliberately owned.",
      "recommendation": "Prefer @react-three/fiber/native for declarative scenes, or keep expo-three usage narrowly justified and validated."
    },
    {
      "id": "r3f.asset-exts-missing-metro",
      "severity": "medium",
      "confidence": "medium",
      "category": "asset",
      "kind": "assetExtensionNeedsMetro",
      "extensions": ["glb", "gltf", "bin", "ktx2", "hdr", "obj", "mtl"],
      "message": "Native 3D assets are referenced without an obvious Metro assetExts configuration for those formats.",
      "recommendation": "Add only the needed model/texture extensions to Metro resolver.assetExts or document the existing bundling path."
    },
    {
      "id": "r3f.remote-model-asset",
      "severity": "medium",
      "confidence": "medium",
      "category": "asset",
      "pattern": "https?://[^'\"`\\s]+\\.(glb|gltf|bin|ktx2|hdr|obj|mtl|png|jpe?g)",
      "message": "A model or texture appears to load from a remote URL.",
      "recommendation": "Define cache, timeout, offline, error, and content-size policy before relying on remote 3D assets in native release builds."
    },
    {
      "id": "r3f.dispose-null",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "pattern": "dispose\\s*=\\s*\\{\\s*null\\s*\\}",
      "message": "dispose={null} disables R3F automatic disposal for this subtree.",
      "recommendation": "Keep dispose={null} only when shared resources have an explicit lifetime owner and route unmount/remount has been validated."
    },
    {
      "id": "r3f.animation-mixer-no-frame-update",
      "severity": "high",
      "confidence": "medium",
      "category": "animation",
      "kind": "fileContainsWithout",
      "include": "AnimationMixer|new\\s+THREE\\.AnimationMixer",
      "without": "useFrame|mixer\\.update|useAnimations",
      "message": "AnimationMixer usage appears without an obvious frame-delta update path.",
      "recommendation": "Update mixers from useFrame with delta, or use a native-safe helper that owns mixer updates."
    },
    {
      "id": "r3f.manual-glview-missing-end-frame",
      "severity": "high",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "GLView|onContextCreate|ExpoWebGLRenderingContext",
      "also": "THREE\\.WebGLRenderer|WebGLRenderer|renderer\\.render\\s*\\(",
      "without": "endFrameEXP\\s*\\(",
      "message": "Manual Expo GLView Three renderer appears without gl.endFrameEXP() frame presentation.",
      "recommendation": "Call gl.endFrameEXP() after each presented frame, or migrate to @react-three/fiber/native Canvas."
    },
    {
      "id": "r3f.manual-glview-missing-cleanup",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "GLView|onContextCreate|ExpoWebGLRenderingContext",
      "also": "requestAnimationFrame|setAnimationLoop|renderer\\.render\\s*\\(",
      "without": "cancelAnimationFrame|setAnimationLoop\\s*\\(\\s*null\\s*\\)|dispose\\s*\\(|destroyContextAsync|destroyObjectAsync|return\\s+\\(\\s*\\)\\s*=>",
      "message": "Manual Expo GLView render loop appears without obvious unmount cleanup.",
      "recommendation": "Cancel loops and dispose renderer, controls, geometries, materials, textures, and native GL objects on unmount."
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

function shouldSkipFile(file) {
  return path.resolve(file) === selfFile;
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
      } else if (entry.isFile() && !shouldSkipFile(full) && fileExtensions.has(path.extname(entry.name))) {
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
      } else if (entry.isFile() && !shouldSkipFile(full)) {
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
  if (!fs.existsSync(root)) return { exists: false, packages: new Set(), packageFiles: [], scripts: {} };
  const files = listPackageFiles(root);
  const packages = new Set();
  const packageFiles = [];
  const scripts = {};
  for (const file of files) {
    try {
      const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
      Object.assign(scripts, pkg.scripts ?? {});
      const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
      const packageNames = new Set(Object.keys(deps ?? {}));
      for (const name of packageNames) packages.add(name);
      packageFiles.push({ file: path.relative(root, file), packages: packageNames });
    } catch {
      continue;
    }
  }
  return { exists: files.length > 0, packages, packageFiles, scripts };
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

function readUtf8(file) {
  try {
    return fs.readFileSync(file, 'utf8');
  } catch {
    return null;
  }
}

function assetExtensionFindings(rule, root, files, config) {
  const extensions = rule.extensions ?? [];
  const assetPattern = new RegExp(`(?:require\\(\\s*['"\`]([^'"\`\\n]+\\.(${extensions.join('|')}))['"\`]\\s*\\)|import\\s+(?:[^'"\`;]+?\\s+from\\s+)?['"\`]([^'"\`\\n]+\\.(${extensions.join('|')}))['"\`])`, 'gi');
  const referenced = new Map();
  for (const file of files) {
    const text = readUtf8(file);
    if (!text) continue;
    for (const match of text.matchAll(assetPattern)) {
      const assetPath = match[1] ?? match[3] ?? '';
      if (/^[a-z][a-z0-9+.-]*:\/\//i.test(assetPath)) continue;
      const extension = (match[2] ?? match[4]).toLowerCase();
      const line = lineForIndex(text, match.index ?? 0);
      const relativePath = path.relative(root, file);
      if (!referenced.has(extension) && !isIgnored(config, rule.id, relativePath, text.split('\n'), line)) {
        referenced.set(extension, { file: relativePath, line, excerpt: excerptForLine(text.split('\n'), line) });
      }
    }
  }
  if (referenced.size === 0) return [];

  const metroFiles = files.filter((file) => /(^|\/)metro\.config\.(js|cjs|mjs|ts)$/.test(path.relative(root, file).replaceAll(path.sep, '/')));
  const metroText = metroFiles.map((file) => readUtf8(file) ?? '').join('\n');
  const missing = [...referenced.keys()].filter((extension) => {
    return !assetExtsContains(metroText, extension);
  });
  if (missing.length === 0) return [];

  const first = referenced.get(missing[0]);
  return [
    {
      id: `${rule.id}:${first.file}:${first.line}`,
      ruleId: rule.id,
      severity: rule.severity,
      confidence: rule.confidence,
      category: rule.category,
      file: first.file,
      line: first.line,
      excerpt: `${first.excerpt}; missing assetExts: ${missing.join(', ')}`,
      rationale: rule.message,
      recommendation: rule.recommendation,
    },
  ];
}

function assetExtsContains(text, extension) {
  const escaped = extension.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  return new RegExp(`assetExts\\.(?:push|unshift)\\(\\s*['"\`]${escaped}['"\`]`, 'i').test(text)
    || new RegExp(`assetExts\\s*[:=]\\s*\\[[^\\]]*['"\`]${escaped}['"\`][^\\]]*\\]`, 'i').test(text);
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'useFrameContains') {
    const setterRegex = ruleRegex(rule.pattern);
    const frameRegex = /useFrame\s*\([\s\S]*?=>\s*\{([\s\S]*?)\}\s*(?:,|\))/g;
    const functionFrameRegex = /useFrame\s*\(\s*function(?:\s+\w+)?\s*\([^)]*\)\s*\{([\s\S]*?)\}\s*(?:,|\))/g;
    for (const match of text.matchAll(frameRegex)) {
      const body = match[1] ?? '';
      setterRegex.lastIndex = 0;
      const setterMatch = setterRegex.exec(body);
      if (!setterMatch) continue;
      const line = lineForIndex(text, (match.index ?? 0) + match[0].indexOf(body) + setterMatch.index);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    for (const match of text.matchAll(functionFrameRegex)) {
      const body = match[1] ?? '';
      setterRegex.lastIndex = 0;
      const setterMatch = setterRegex.exec(body);
      if (!setterMatch) continue;
      const line = lineForIndex(text, (match.index ?? 0) + match[0].indexOf(body) + setterMatch.index);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    const conciseFrameRegex = /useFrame\s*\([\s\S]*?=>\s*(?!\s*\{)([^\n;]*)/g;
    for (const match of text.matchAll(conciseFrameRegex)) {
      const body = match[1] ?? '';
      setterRegex.lastIndex = 0;
      const setterMatch = setterRegex.exec(body);
      if (!setterMatch) continue;
      const line = lineForIndex(text, (match.index ?? 0) + match[0].indexOf(body) + setterMatch.index);
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
  const allFiles = listAllFiles(root, maxFiles);
  const pkg = readPackage(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'assetExtensionNeedsMetro') {
      findings.push(...assetExtensionFindings(rule, root, files, config));
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
    if (rule.kind === 'packageHasAnyWithout') {
      for (const packageFile of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageFile.packages.has(name));
        const missing = (rule.withoutPackages ?? []).filter((name) => !packageFile.packages.has(name));
        if (matched.length === 0 || missing.length === 0) continue;
        if (isIgnored(config, rule.id, packageFile.file, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packageFile.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageFile.file,
          line: 1,
          excerpt: `matched packages: ${matched.join(', ')}; missing packages: ${missing.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageHasAllWithout') {
      for (const packageFile of pkg.packageFiles) {
        const matched = (rule.packages ?? []).filter((name) => packageFile.packages.has(name));
        const missing = (rule.withoutPackages ?? []).filter((name) => !packageFile.packages.has(name));
        if (matched.length !== (rule.packages ?? []).length || missing.length === 0) continue;
        if (isIgnored(config, rule.id, packageFile.file, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packageFile.file}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packageFile.file,
          line: 1,
          excerpt: `matched packages: ${matched.join(', ')}; missing packages: ${missing.join(', ')}`,
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
    sampledFiles: allFiles.length,
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
