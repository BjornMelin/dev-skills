#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "web-waapi",
  "rules": [
    {
      "id": "waapi.fire-and-forget-animation",
      "severity": "high",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "animationCallWithoutOwner",
      "pattern": "\\.animate\\s*\\(",
      "without": "(?:const|let|var)\\s+[A-Za-z_$][\\w$]*\\s*=|[A-Za-z_$][\\w$]*\\s*=\\s*[^\\n]*\\.animate\\s*\\(|[A-Za-z_$][\\w$]*(?:\\??\\.[A-Za-z_$][\\w$]*|\\[[^\\]]+\\])+\\s*=|return\\s+[^\\n]*\\.animate\\s*\\(|\\.getAnimations\\s*\\(|motion-audit-ignore",
      "message": "Element.animate() appears to be fired without keeping the returned Animation handle.",
      "recommendation": "Store or return the Animation handle unless the effect is intentionally fire-and-forget and cannot be interrupted."
    },
    {
      "id": "waapi.handle-without-cleanup",
      "severity": "high",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "animationHandleWithoutCleanup",
      "include": "\\b(?:const|let|var)\\s+[A-Za-z_$][\\w$]*\\s*=\\s*[^;\\n]*\\.animate\\s*\\(|new\\s+Animation\\s*\\(",
      "without": "\\.cancel\\(|\\.finish\\(|\\.persist\\(|addEventListener\\s*\\(\\s*['\"]finish|\\.onfinish\\b",
      "message": "A WAAPI Animation handle appears without an obvious cleanup, finish, persist, or final-style handoff path.",
      "recommendation": "Cancel on unmount/retrigger, intentionally finish, or commit final styles and remove the animation effect."
    },
    {
      "id": "waapi.fill-without-persistence-decision",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fillWithoutPersistenceDecision",
      "include": "\\.animate\\(",
      "also": "\\bfill\\s*:\\s*['\"](?:forwards|both)['\"]",
      "without": "\\.cancel\\(|\\.persist\\(|replaceState|CSS-owned|final style|final-state|data-state",
      "message": "A filling WAAPI animation appears without an explicit persistence or removal decision.",
      "recommendation": "Prefer CSS-owned final styles or commitStyles() followed by cancel(); use persist() only when automatic removal is wrong."
    },
    {
      "id": "waapi.finished-without-abort-handling",
      "severity": "medium",
      "confidence": "medium",
      "category": "lifecycle",
      "kind": "fileContainsBothWithout",
      "include": "\\.finished\\b",
      "also": "\\.cancel\\(",
      "without": "AbortError|\\.catch\\s*\\(|try\\s*\\{|Promise\\.allSettled",
      "message": "animation.finished is used in a file that cancels animations without obvious AbortError handling.",
      "recommendation": "Catch expected AbortError rejection from interrupted animations and rethrow unexpected errors."
    },
    {
      "id": "waapi.commitstyles-without-error-boundary",
      "severity": "medium",
      "confidence": "low",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "commitStyles\\s*\\(",
      "without": "try\\s*\\{|catch\\s*\\(|NoModificationAllowedError|InvalidStateError|isConnected|getClientRects|offsetParent|motion-audit-ignore",
      "message": "commitStyles() writes computed values inline and can fail for unsupported, disconnected, or non-rendered targets.",
      "recommendation": "Use commitStyles() only for deliberate persistence and guard or catch cases where the target may not accept committed inline styles."
    },
    {
      "id": "waapi.newer-feature-without-support-policy",
      "severity": "medium",
      "confidence": "medium",
      "category": "support",
      "kind": "fileContainsWithout",
      "include": "\\b(?:ScrollTimeline|ViewTimeline|rangeStart|rangeEnd|timeline\\s*:|iterationComposite|composite\\s*:\\s*['\"](?:add|accumulate)['\"])",
      "without": "CSS\\.supports|@supports|browser support|support policy|feature guard|typeof\\s+(?:ScrollTimeline|ViewTimeline)|(?:ScrollTimeline|ViewTimeline)\\s+in\\s+window|motion-audit-ignore",
      "message": "Newer WAAPI timeline/range/composition behavior appears without obvious browser-support proof or guard.",
      "recommendation": "Check the target browser policy and add a feature guard, fallback, or explicit support note before shipping."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "requires": "\\.animate\\s*\\(|new\\s+KeyframeEffect\\s*\\(",
      "pattern": "\\b(width|height|top|left|right|bottom|margin|padding)\\s*:",
      "message": "Layout-affecting properties are being animated or configured near motion code.",
      "recommendation": "Prefer transform and opacity in hot paths; measure before keeping layout animation."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\.animate\\s*\\(|new\\s+Animation\\s*\\(|new\\s+KeyframeEffect\\s*\\(",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|reduceMotion|matchMedia\\s*\\(",
      "message": "WAAPI motion code was found without an obvious reduced-motion branch in the same file.",
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
  if (!fs.existsSync(file)) return { exists: false, packages: new Set() };
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return { exists: true, packages: new Set(Object.keys(deps ?? {})), scripts: pkg.scripts ?? {} };
  } catch {
    return { exists: true, packages: new Set(), scripts: {} };
  }
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

function testRegex(pattern) {
  return new RegExp(pattern, 'ms');
}

function escapeRegex(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
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

function cleanupRegexForTarget(target) {
  const escaped = escapeRegex(target);
  return new RegExp(`\\b${escaped}\\s*\\??\\.\\s*(?:cancel|finish|persist)\\s*(?:\\?\\.\\s*)?\\(|\\b${escaped}\\s*\\??\\.\\s*onfinish\\b|\\b${escaped}\\s*\\??\\.\\s*addEventListener\\s*(?:\\?\\.\\s*)?\\(\\s*['"]finish['"]`, 'm');
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

function ownerRangeForTarget(text, target, index) {
  const functionOpen = ownerFunctionOpenAt(text, index);
  const declarationIndex = declarationIndexBefore(text, target, index, functionOpen ?? 0);
  const ownerOpen =
    declarationIndex == null
      ? functionOpen ?? openBraceStackAt(text, index).at(-1)
      : openBraceStackAt(text, declarationIndex).at(-1);
  return ownerOpen == null ? [0, text.length] : blockRangeForOpen(text, ownerOpen);
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

function assignmentTargetBefore(text, index) {
  const statementStart = Math.max(
    text.lastIndexOf(';', index),
    text.lastIndexOf('{', index),
    text.lastIndexOf('}', index),
  ) + 1;
  const prefix = text.slice(statementStart, index);
  const match = prefix.match(/(?:const|let|var)\s+([A-Za-z_$][\w$]*)(?:\s*:\s*[^=\n;]+)?\s*=([\s\S]*)$|([A-Za-z_$][\w$]*(?:\??\.[A-Za-z_$][\w$]*|\[[^\]]+\])+)\s*=([\s\S]*)$|([A-Za-z_$][\w$]*)\s*=([\s\S]*)$/);
  const tail = match?.[2] ?? match?.[4] ?? match?.[6] ?? '';
  if (tail.includes('\n')) {
    const beforeFirstNewline = tail.slice(0, tail.indexOf('\n')).trim();
    const afterLastNewline = tail.slice(tail.lastIndexOf('\n') + 1).trim();
    if (beforeFirstNewline !== '' && afterLastNewline !== '') return null;
  }
  return match?.[1] ?? match?.[3] ?? match?.[5] ?? null;
}

function ownsAnimationCall(prefix) {
  const assignment = prefix.match(/(?:const|let|var)\s+[A-Za-z_$][\w$]*(?:\s*:\s*[^=\n;]+)?\s*=([\s\S]*)$|[A-Za-z_$][\w$]*(?:\??\.[A-Za-z_$][\w$]*|\[[^\]]+\])+\s*=([\s\S]*)$|\b[A-Za-z_$][\w$]*\s*=([\s\S]*)$/);
  const returned = prefix.match(/\breturn[^\S\n]+([\s\S]*)$/);
  const tail = assignment?.[1] ?? assignment?.[2] ?? assignment?.[3] ?? returned?.[1] ?? null;
  if (tail == null) return false;
  if (!tail.includes('\n')) return true;
  const lines = tail.split('\n').map((line) => line.trim()).filter(Boolean);
  if (lines.length <= 1) return true;
  return lines.slice(0, -1).every((line) => /(?:[([,{?:+\-*/%&|^!.]|=>)$/.test(line));
}

function animationCallIndexForOption(text, index) {
  const regex = /\.animate\s*\(|new\s+Animation\s*\(/g;
  let result = null;
  for (const match of text.matchAll(regex)) {
    const matchIndex = match.index ?? 0;
    if (matchIndex > index) break;
    const openParen = text.indexOf('(', matchIndex);
    if (openParen === -1) continue;
    const closeParen = matchingParen(text, openParen);
    if (closeParen !== -1 && index > openParen && index < closeParen) result = matchIndex;
  }
  return result;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.requires && !testRegex(rule.requires).test(text)) return findings;
  if (rule.kind === 'animationHandleWithoutCleanup') {
    const regex = /\b(?:(?:const|let|var)\s+([A-Za-z_$][\w$]*)(?:\s*:\s*[^=\n;]+)?|([A-Za-z_$][\w$]*(?:\??\.[A-Za-z_$][\w$]*|\[[^\]]+\])+)|([A-Za-z_$][\w$]*))\s*=\s*(?:[\s\S]{0,240}?\.animate\s*\(|new\s+Animation\s*\()/g;
    for (const match of text.matchAll(regex)) {
      const callOffset = match[0].search(/\.animate\s*\(|new\s+Animation\s*\(/);
      const callIndex = (match.index ?? 0) + Math.max(0, callOffset);
      const statementStart = Math.max(
        text.lastIndexOf(';', callIndex),
        text.lastIndexOf('{', callIndex),
        text.lastIndexOf('}', callIndex),
      ) + 1;
      if (!ownsAnimationCall(text.slice(statementStart, callIndex))) continue;
      const name = match[1] ?? match[2] ?? match[3];
      const cleanup = cleanupRegexForTarget(name);
      if (cleanup.test(cleanupEvidenceText(text, name, match.index ?? 0))) continue;
      const line = lineForIndex(text, match.index ?? 0);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'animationCallWithoutOwner') {
    const callRegex = /\.animate\s*\(|new\s+Animation\s*\(/g;
    for (const match of text.matchAll(callRegex)) {
      const index = match.index ?? 0;
      const statementStart = Math.max(
        text.lastIndexOf(';', index),
        text.lastIndexOf('{', index),
        text.lastIndexOf('}', index),
      ) + 1;
      const prefix = text.slice(statementStart, index);
      const owned = ownsAnimationCall(prefix);
      if (owned || prefix.includes('motion-audit-ignore')) continue;
      const line = lineForIndex(text, index);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'fillWithoutPersistenceDecision') {
    const regex = ruleRegex(rule.also);
    for (const match of text.matchAll(regex)) {
      const index = match.index ?? 0;
      const animateIndex = animationCallIndexForOption(text, index);
      if (animateIndex == null) continue;
      const target = assignmentTargetBefore(text, animateIndex);
      const evidence = target
        ? cleanupEvidenceText(text, target, animateIndex)
        : enclosingBraceBlock(text, animateIndex);
      const targetCleanup = target
        ? new RegExp(`\\b${escapeRegex(target)}\\s*\\??\\.\\s*(?:cancel|persist)\\s*(?:\\?\\.\\s*)?\\(`, 'm').test(evidence)
        : false;
      if (targetCleanup || /replaceState|CSS-owned|final style|final-state|data-state/.test(evidence)) continue;
      const line = lineForIndex(text, index);
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'linePatternWithoutNearby') {
    const regex = ruleRegex(rule.pattern);
    const without = rule.without ? testRegex(rule.without) : null;
    for (const match of text.matchAll(regex)) {
      const line = lineForIndex(text, match.index ?? 0);
      const nearby = rule.scope === 'line'
        ? (lines[line - 1] ?? '')
        : lines.slice(Math.max(0, line - 3), Math.min(lines.length, line + 3)).join('\n');
      if (!without || !without.test(nearby)) {
        if (!isIgnored(config, rule.id, relativePath, lines, line)) {
          findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
        }
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
      'regex findings are review leads, not proof',
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
