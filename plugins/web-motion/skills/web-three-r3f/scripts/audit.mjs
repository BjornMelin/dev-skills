#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "web-three-r3f",
  "rules": [
    {
      "id": "r3f.set-state-in-frame",
      "severity": "high",
      "confidence": "medium",
      "category": "performance",
      "kind": "useFrameContains",
      "include": "useFrame\\(",
      "also": "(^|[^.\\w$])set[A-Z][A-Za-z0-9_]*\\(",
      "message": "React state appears to be updated from a React Three Fiber frame loop.",
      "recommendation": "Use refs or external transient state for frame-loop mutation."
    },
    {
      "id": "r3f.canvas-size",
      "severity": "medium",
      "confidence": "low",
      "category": "rendering",
      "pattern": "<Canvas(?![^>]*(style|className|height|width|frameloop|dpr))",
      "message": "Canvas appears without obvious sizing or render-loop constraints.",
      "recommendation": "Ensure stable dimensions and consider DPR/frameloop settings for performance."
    },
    {
      "id": "r3f.canvas-without-fallback",
      "severity": "medium",
      "confidence": "low",
      "category": "rendering",
      "kind": "fileContainsWithout",
      "include": "<Canvas\\b",
      "without": "fallback=|ErrorBoundary|useErrorBoundary|onError|motion-audit-ignore",
      "message": "Canvas appears without an obvious unsupported-WebGL or context-error fallback.",
      "recommendation": "Add a Canvas fallback or route-level error boundary when the 3D surface is product-critical."
    },
    {
      "id": "r3f.raw-device-pixel-ratio",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "(?:dpr=\\{\\s*(?:window\\.)?devicePixelRatio\\s*\\}|setPixelRatio\\(\\s*(?:window\\.)?devicePixelRatio)",
      "message": "Canvas or renderer appears to use raw devicePixelRatio without a clamp.",
      "recommendation": "Clamp DPR with a range or adaptive policy, especially for mobile/heavy scenes."
    },
    {
      "id": "r3f.primitive-manual-ownership",
      "severity": "medium",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "fileContainsWithout",
      "include": "<primitive\\b",
      "without": "dispose\\(|useGLTF\\.clear|dispose=\\{null\\}|motion-audit-ignore",
      "message": "A React Three Fiber primitive was found without an obvious manual ownership note or cleanup path.",
      "recommendation": "Confirm who owns the primitive object and dispose external resources explicitly when obsolete."
    },
    {
      "id": "r3f.create-root-without-unmount",
      "severity": "high",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "fileContainsBothWithout",
      "include": "import\\s*\\{[^}]*\\bcreateRoot\\b[^}]*\\}\\s*from\\s*['\"]@react-three/fiber['\"]",
      "also": "\\bcreateRoot\\(",
      "without": "\\.unmount\\(|root\\.unmount|motion-audit-ignore",
      "message": "A custom React Three Fiber root was created without an obvious unmount path.",
      "recommendation": "Ensure custom createRoot(canvas) owners unmount the root and release scene resources on teardown."
    },
    {
      "id": "r3f.create-root-without-resize-owner",
      "severity": "medium",
      "confidence": "medium",
      "category": "rendering",
      "kind": "fileContainsBothWithout",
      "include": "import\\s*\\{[^}]*\\bcreateRoot\\b[^}]*\\}\\s*from\\s*['\"]@react-three/fiber['\"]",
      "also": "\\bcreateRoot\\(",
      "without": "resize|configure\\(\\s*\\{[\\s\\S]*size|useMeasure|react-use-measure|motion-audit-ignore",
      "message": "A custom React Three Fiber root was created without an obvious resize owner.",
      "recommendation": "Custom createRoot(canvas) code owns sizing; configure size on mount and resize or prefer Canvas."
    },
    {
      "id": "three.renderer-without-dispose",
      "severity": "high",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "rendererWithoutDispose",
      "include": "new\\s+(?:THREE\\.)?WebGLRenderer\\(",
      "without": "(?:renderer|gl)\\.dispose\\(|forceContextLoss\\(|motion-audit-ignore",
      "message": "A plain Three.js WebGLRenderer was created without an obvious cleanup path.",
      "recommendation": "Stop the render loop, remove listeners, and dispose renderer-owned resources on unmount."
    },
    {
      "id": "three.animation-loop-without-stop",
      "severity": "high",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "animationLoopWithoutStop",
      "include": "\\.setAnimationLoop\\(",
      "without": "\\.setAnimationLoop\\(\\s*null\\s*\\)|motion-audit-ignore",
      "message": "A Three.js animation loop was configured without an obvious stop call.",
      "recommendation": "Call renderer.setAnimationLoop(null) before disposing or replacing the renderer."
    },
    {
      "id": "three.renderer-listener-without-remove",
      "severity": "medium",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "fileContainsBothWithout",
      "include": "new\\s+(?:THREE\\.)?WebGLRenderer\\(",
      "also": "addEventListener\\(",
      "without": "removeEventListener\\(|motion-audit-ignore",
      "message": "A plain Three.js renderer owner adds listeners without an obvious removal path.",
      "recommendation": "Remove resize, pointer, visibility, and context listeners during scene teardown."
    },
    {
      "id": "three.render-target-without-dispose",
      "severity": "medium",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "fileContainsWithout",
      "include": "new\\s+(?:THREE\\.)?(?:WebGLRenderTarget|WebGLCubeRenderTarget|EffectComposer)\\(",
      "without": "\\.dispose\\(|motion-audit-ignore",
      "message": "A render target or postprocessing composer appears without an obvious disposal path.",
      "recommendation": "Dispose render targets, composers, and passes when obsolete or when the route unmounts."
    },
    {
      "id": "three.raf-without-cancel",
      "severity": "medium",
      "confidence": "medium",
      "category": "cleanup",
      "kind": "fileContainsWithout",
      "include": "requestAnimationFrame\\(",
      "without": "cancelAnimationFrame\\(|setAnimationLoop\\(|motion-audit-ignore",
      "message": "A requestAnimationFrame loop or scheduled frame appears without obvious cancellation.",
      "recommendation": "Cancel RAF on unmount, or use renderer.setAnimationLoop with a null cleanup."
    },
    {
      "id": "r3f.loader-instance-in-component",
      "severity": "medium",
      "confidence": "low",
      "category": "assets",
      "pattern": "new\\s+(?:GLTFLoader|TextureLoader|DRACOLoader|KTX2Loader|FBXLoader|OBJLoader)\\(",
      "message": "A Three.js loader instance was created directly in source.",
      "recommendation": "Prefer useLoader/Drei loader hooks, or memoize/configure the loader with a clear ownership path."
    },
    {
      "id": "r3f.loader-hook-without-suspense",
      "severity": "medium",
      "confidence": "low",
      "category": "assets",
      "kind": "fileContainsWithout",
      "include": "\\b(?:useLoader|useGLTF|useTexture)\\(",
      "without": "Suspense|fallback=|useProgress|motion-audit-ignore",
      "message": "R3F/Drei loader hooks appear without an obvious Suspense or loading boundary in the same file.",
      "recommendation": "Confirm the route or parent scene provides Suspense/loading and asset-error fallback behavior."
    },
    {
      "id": "r3f.decoder-cdn-boundary",
      "severity": "medium",
      "confidence": "medium",
      "category": "assets",
      "pattern": "\\.(?:setDecoderPath|setTranscoderPath)\\(\\s*['\"]https?://",
      "message": "A Draco/KTX2 decoder path points at a remote URL.",
      "recommendation": "Confirm the CDN path matches CSP, offline, asset-prefix, and deployment rules, or serve decoders with app assets."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\b(gsap\\.|motion\\.|<motion\\.|withRepeat\\(|withTiming\\(|withSpring\\(|useFrame\\(|lottie|Rive|Skia|animate\\(|@keyframes)\\b",
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

function readPackage(root) {
  const file = path.join(root, 'package.json');
  if (!fs.existsSync(file)) return { exists: false, packages: new Set(), versions: {}, scripts: {} };
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return {
      exists: true,
      packages: new Set(Object.keys(deps ?? {})),
      versions: deps ?? {},
      scripts: pkg.scripts ?? {},
    };
  } catch {
    return { exists: true, packages: new Set(), versions: {}, scripts: {} };
  }
}

function packageHints(pkg) {
  const names = ['three', '@react-three/fiber', '@react-three/drei', 'react', 'react-dom'];
  return names.reduce((acc, name) => {
    if (pkg.versions?.[name]) acc[name] = pkg.versions[name];
    return acc;
  }, {});
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

function matchingParen(text, openIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  for (let index = openIndex; index < text.length; index += 1) {
    const char = text[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
      continue;
    }
    if (char === '(') depth += 1;
    else if (char === ')') {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function matchingBrace(text, openIndex) {
  let depth = 0;
  let quote = null;
  let escaped = false;
  for (let index = openIndex; index < text.length; index += 1) {
    const char = text[index];
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
      continue;
    }
    if (char === '{') depth += 1;
    else if (char === '}') {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  return -1;
}

function openBraceStackAt(text, index) {
  const stack = [];
  let quote = null;
  let escaped = false;
  for (let cursor = 0; cursor < index; cursor += 1) {
    const char = text[cursor];
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
      continue;
    }
    if (char === '{') stack.push(cursor);
    else if (char === '}') stack.pop();
  }
  return stack;
}

function enclosingBraceBlock(text, index) {
  const open = openBraceStackAt(text, index).at(-1);
  if (open == null) return text;
  const close = matchingBrace(text, open);
  return close === -1 ? text.slice(open) : text.slice(open, close + 1);
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function cleanupRegexForTarget(target, methods) {
  return new RegExp(`\\b${escapeRegex(target)}\\s*\\??\\.\\s*(?:${methods})\\s*(?:\\?\\.\\s*)?\\(`, 'm');
}

function blockRangeForOpen(text, open) {
  const close = matchingBrace(text, open);
  return [open, close === -1 ? text.length : close + 1];
}

function isBareIdentifier(target) {
  return /^[A-Za-z_$][\w$]*$/.test(target);
}

function declarationIndexBefore(text, target, index, searchStart = 0) {
  if (!isBareIdentifier(target)) return null;
  const regex = new RegExp(`\\b(?:const|let|var)\\s+${escapeRegex(target)}\\b`, 'g');
  let found = null;
  for (const match of text.matchAll(regex)) {
    const matchIndex = match.index ?? 0;
    if (matchIndex > index) break;
    if (matchIndex >= searchStart) found = matchIndex;
  }
  return found;
}

function statementPrelude(text, open, ownerStart) {
  const start =
    Math.max(
      text.lastIndexOf(';', open - 1),
      text.lastIndexOf('{', open - 1),
      text.lastIndexOf('}', open - 1),
    ) + 1;
  return text.slice(Math.max(start, ownerStart), open).trim();
}

function isFunctionLikePrelude(prelude) {
  const compact = prelude.replace(/\/\*[\s\S]*?\*\//g, ' ').replace(/(^|[\s;{}])\/\/[^\n\r]*/g, '$1 ').trim();
  if (/^(?:if|for|while|switch|catch|with)\b/.test(compact)) return false;
  return /\bfunction\b/.test(compact) || /=>\s*$/.test(compact) || /^[A-Za-z_$][\w$]*\s*\([^)]*\)\s*$/.test(compact);
}

function isReturnCleanupPrelude(prelude) {
  return /\breturn\b/.test(prelude) && (/\bfunction\b/.test(prelude) || /=>\s*$/.test(prelude));
}

function returnedIdentifiers(text) {
  return new Set([...text.matchAll(/\breturn[^\S\n]+([A-Za-z_$][\w$]*)\s*;?/g)].map((match) => match[1]));
}

function functionNameForPrelude(prelude) {
  return (
    prelude.match(/\bfunction\s+([A-Za-z_$][\w$]*)\s*\(/)?.[1] ??
    prelude.match(/\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\b[\s\S]*=>\s*$/)?.[1] ??
    prelude.match(/\b([A-Za-z_$][\w$]*)\s*=\s*[\s\S]*=>\s*$/)?.[1] ??
    prelude.match(/\b([A-Za-z_$][\w$]*)\s*\([^)]*\)\s*$/)?.[1] ??
    null
  );
}

function ownerFunctionOpenAt(text, index) {
  const stack = openBraceStackAt(text, index);
  for (const open of stack.slice().reverse()) {
    if (isFunctionLikePrelude(statementPrelude(text, open, 0))) return open;
  }
  return null;
}

function ownerRangeForTarget(text, target, index) {
  const functionOpen = ownerFunctionOpenAt(text, index);
  const declarationIndex = declarationIndexBefore(text, target, index, functionOpen ?? 0);
  const ownerOpen =
    declarationIndex == null
      ? functionOpen ?? openBraceStackAt(text, index).at(-1)
      : openBraceStackAt(text, declarationIndex).at(-1);
  return ownerOpen == null ? [0, text.length] : blockRangeForOpen(text, ownerOpen);
}

function cleanupEvidenceText(text, target, index) {
  const [ownerStart, ownerEnd] = ownerRangeForTarget(text, target, index);
  const returned = returnedIdentifiers(text.slice(ownerStart, ownerEnd));
  const chars = text.slice(ownerStart, ownerEnd).split('');
  let quote = null;
  let escaped = false;
  for (let cursor = ownerStart + 1; cursor < ownerEnd; cursor += 1) {
    const char = text[cursor];
    if (quote) {
      if (escaped) escaped = false;
      else if (char === '\\') escaped = true;
      else if (char === quote) quote = null;
      continue;
    }
    if (char === '"' || char === "'" || char === '`') {
      quote = char;
      continue;
    }
    if (char !== '{') continue;
    const close = matchingBrace(text, cursor);
    if (close === -1 || close >= ownerEnd) continue;
    if (cursor <= index && index <= close) continue;
    const prelude = statementPrelude(text, cursor, ownerStart);
    const returnedHelper = returned.has(functionNameForPrelude(prelude));
    if (isFunctionLikePrelude(prelude) && !isReturnCleanupPrelude(prelude) && !returnedHelper) {
      for (let blank = cursor; blank <= close; blank += 1) chars[blank - ownerStart] = text[blank] === '\n' ? '\n' : ' ';
      cursor = close;
    }
  }
  return chars.join('');
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'useFrameContains') {
    const includeRegex = ruleRegex(rule.include);
    const alsoRegex = ruleRegex(rule.also);
    for (const match of text.matchAll(includeRegex)) {
      const openParen = text.indexOf('(', match.index ?? 0);
      const closeParen = openParen === -1 ? -1 : matchingParen(text, openParen);
      if (closeParen === -1) continue;
      const callText = text.slice(openParen, closeParen + 1);
      alsoRegex.lastIndex = 0;
      if (!alsoRegex.test(callText)) continue;
      const line = lineForIndex(text, match.index ?? 0);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'rendererWithoutDispose') {
    const assigned = /\b(?:(?:const|let|var)\s+([A-Za-z_$][\w$]*)(?:\s*:\s*[^=\n;]+)?|([A-Za-z_$][\w$]*(?:\??\.[A-Za-z_$][\w$]*|\[[^\]]+\])+)|([A-Za-z_$][\w$]*))\s*=\s*new\s+(?:THREE\.)?WebGLRenderer\s*\(/g;
    const matchedRanges = [];
    for (const match of text.matchAll(assigned)) {
      const name = match[1] ?? match[2] ?? match[3];
      const cleanup = cleanupRegexForTarget(name, 'dispose|forceContextLoss');
      if (cleanup.test(cleanupEvidenceText(text, name, match.index ?? 0))) {
        matchedRanges.push([match.index ?? 0, (match.index ?? 0) + match[0].length]);
        continue;
      }
      const line = lineForIndex(text, match.index ?? 0);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
      matchedRanges.push([match.index ?? 0, (match.index ?? 0) + match[0].length]);
    }
    const bare = /new\s+(?:THREE\.)?WebGLRenderer\s*\(/g;
    for (const match of text.matchAll(bare)) {
      const index = match.index ?? 0;
      if (matchedRanges.some(([start, end]) => index >= start && index < end)) continue;
      const line = lineForIndex(text, index);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'animationLoopWithoutStop') {
    const loopCall = /\b([A-Za-z_$][\w$]*(?:\??\.[A-Za-z_$][\w$]*|\[[^\]]+\])*)\s*\??\.\s*setAnimationLoop\s*(?:\?\.\s*)?\(\s*(?!null\b)/g;
    for (const match of text.matchAll(loopCall)) {
      const target = match[1];
      const cleanup = new RegExp(`\\b${escapeRegex(target)}\\s*\\??\\.\\s*setAnimationLoop\\s*(?:\\?\\.\\s*)?\\(\\s*null\\s*\\)`, 'm');
      if (cleanup.test(cleanupEvidenceText(text, target, match.index ?? 0))) continue;
      const line = lineForIndex(text, match.index ?? 0);
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
    const relativePath = path.relative(root, file);
    if (
      relativePath === path.join('scripts', 'audit.mjs') &&
      text.includes('"skillName": "web-three-r3f"')
    ) {
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
    packageHints: packageHints(pkg),
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
    packageHints: packageHints(pkg),
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
- Package hints: ${Object.keys(result.packageHints ?? {}).length > 0 ? JSON.stringify(result.packageHints) : 'none'}
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
- Package hints: ${Object.keys(result.packageHints ?? {}).length > 0 ? JSON.stringify(result.packageHints) : 'none'}
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
