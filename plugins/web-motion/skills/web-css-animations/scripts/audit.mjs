#!/usr/bin/env node
// motion-audit-skip-file: this helper contains rule strings that look like findings.
import fs from 'node:fs';
import path from 'node:path';

const fileSkipMarker = 'motion-audit-skip-file';

const profile = {
  "skillName": "web-css-animations",
  "rules": [
    {
      "id": "motion.transition-all",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "\\btransitionProperty\\s*[:=]\\s*\\{?\\s*(?:['\"`][^'\"`{};]*\\ball\\b|[^,;{}\\n]*\\ball\\b)|\\btransition\\s*[:=]\\s*\\{?\\s*['\"`][^'\"`{};]*\\ball\\b|\\btransition(?:-property)?\\s*:(?!\\s*\\{?\\s*['\"`])\\s*[^;{}]*\\ball\\b|\\btransition-all\\b",
      "message": "Broad transition-all can animate future expensive properties unintentionally.",
      "recommendation": "Transition only the properties that should move, usually transform and opacity."
    },
    {
      "id": "motion.long-transition-without-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\btransition(?:-[a-z-]+)?\\s*:[^;{}]*?(?:[\\s:,])(?:(?:[6-9]\\d{2}|[1-9]\\d{3,})ms|(?:0?\\.[6-9]\\d*|[1-9](?:\\.\\d+)?)s)\\b",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|AccessibilityInfo|reduceMotion",
      "message": "A long CSS transition was found without an obvious reduced-motion branch in the same file.",
      "recommendation": "Replace nonessential spatial movement under prefers-reduced-motion, or document why the motion is essential and already handled elsewhere."
    },
    {
      "id": "motion.layout-property",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "\\btransitionProperty\\s*[:=]\\s*\\{?\\s*(?:['\"`][^'\"`{};]*\\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\\b|[^,;{}\\n]*\\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\\b)|\\btransition\\s*[:=]\\s*\\{?\\s*['\"`][^'\"`{};]*\\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\\b|\\btransition(?:-property)?\\s*:(?!\\s*\\{?\\s*['\"`])\\s*[^;{}]*\\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\\b",
      "message": "A transition appears to animate layout-affecting properties.",
      "recommendation": "Prefer transform and opacity in hot paths; measure before keeping layout animation."
    },
    {
      "id": "motion.keyframes-layout-property",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "kind": "keyframesLayoutProperty",
      "message": "A keyframe animation appears to animate layout or text-flow properties.",
      "recommendation": "Prefer transform and opacity; keep layout/text-flow keyframes only with target limits and performance proof."
    },
    {
      "id": "motion.will-change-broad",
      "severity": "medium",
      "confidence": "high",
      "category": "performance",
      "pattern": "\\bwill-change\\s*:\\s*(all|contents|scroll-position)\\b|\\bwillChange\\s*=\\s*['\"](all|contents|scroll-position)['\"]",
      "message": "Broad will-change can force wasteful layer or rendering preparation.",
      "recommendation": "Use will-change only for specific active properties, usually transform or opacity."
    },
    {
      "id": "motion.infinite-animation-without-reduced-motion",
      "severity": "medium",
      "confidence": "high",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\banimation(?:-[a-z-]+)?\\s*:[^;{}]*\\binfinite\\b|\\banimate-(spin|pulse|ping|bounce)\\b",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|AccessibilityInfo|reduceMotion",
      "message": "Infinite CSS motion was found without an obvious reduced-motion branch in the same file.",
      "recommendation": "Disable or replace nonessential infinite animation under prefers-reduced-motion."
    },
    {
      "id": "motion.discrete-transition-without-supports",
      "severity": "medium",
      "confidence": "medium",
      "category": "compatibility",
      "kind": "fileContainsWithout",
      "include": "\\btransition-behavior\\s*:\\s*allow-discrete\\b|\\btransition\\s*:[^;{}]*\\ballow-discrete\\b|@starting-style\\b",
      "without": "@supports\\s*\\([^)]*(transition-behavior|@starting-style)|CSS\\.supports\\s*\\([^)]*(transition-behavior|@starting-style)|browser support|Baseline 2024|support policy",
      "message": "Newer discrete transition CSS was found without an obvious feature guard or support-policy note.",
      "recommendation": "Guard transition-behavior/@starting-style with @supports or confirm the target browser support policy."
    },
    {
      "id": "motion.scroll-timeline-without-supports",
      "severity": "medium",
      "confidence": "medium",
      "category": "compatibility",
      "kind": "scrollTimelineWithoutSupport",
      "message": "Scroll-driven CSS animation was found without an obvious feature guard in the same file.",
      "recommendation": "Guard newer timeline CSS with @supports or document the target browser support policy."
    },
    {
      "id": "motion.custom-property-animation-unregistered",
      "severity": "low",
      "confidence": "medium",
      "category": "correctness",
      "kind": "customPropertyAnimationUnregistered",
      "message": "A custom property appears to be animated without an obvious registration in the same file.",
      "recommendation": "Register typed custom properties with @property or CSS.registerProperty() when interpolation matters."
    },
    {
      "id": "motion.animation-name-without-duration",
      "severity": "low",
      "confidence": "medium",
      "category": "correctness",
      "kind": "fileContainsWithout",
      "include": "\\banimation-name\\s*:\\s*(?!none\\b)[^;{}]+",
      "without": "\\banimation-duration\\s*:|\\banimation\\s*:",
      "message": "animation-name was found without an obvious animation-duration in the same file.",
      "recommendation": "Make animation timing explicit; CSS animations default to zero duration when no duration is set."
    },
    {
      "id": "motion.animation-timeline-before-shorthand",
      "severity": "medium",
      "confidence": "medium",
      "category": "correctness",
      "kind": "animationTimelineBeforeShorthand",
      "message": "animation-timeline appears before an animation shorthand that may reset it to auto.",
      "recommendation": "Declare animation-timeline after any animation shorthand in the same rule or cascade block."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "(?:\\b(?:gsap\\.|motion\\.|withRepeat\\(|withTiming\\(|withSpring\\(|useFrame\\(|lottie|Rive|Skia|animate\\()|<motion\\.|@keyframes\\b)",
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
  // motion-audit-skip-file in the first 512 characters skips a generated/helper file
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

function registeredCustomProperties(text) {
  const names = new Set();
  for (const match of text.matchAll(/@property\s+(--[a-zA-Z0-9_-]+)/g)) {
    if (isIncidentalStringMatch(text, match.index ?? 0)) continue;
    names.add(match[1]);
  }
  for (const match of text.matchAll(/registerProperty\s*\(\s*\{[\s\S]{0,400}?(?:\bname|['"`]name['"`])\s*:\s*['"`](--[a-zA-Z0-9_-]+)['"`]/g)) {
    if (isIncidentalStringMatch(text, match.index ?? 0)) continue;
    names.add(match[1]);
  }
  return names;
}

function keyframesBlocks(text) {
  const blocks = [];
  const regex = /@keyframes\s+[-_a-zA-Z0-9]+\s*\{/g;
  for (const match of text.matchAll(regex)) {
    const openBrace = text.indexOf('{', match.index ?? 0);
    if (openBrace === -1) continue;
    let depth = 0;
    for (let index = openBrace; index < text.length; index += 1) {
      const char = text[index];
      if (char === '{') depth += 1;
      else if (char === '}') depth -= 1;
      if (depth === 0) {
        blocks.push({ text: text.slice(match.index ?? 0, index + 1), index: match.index ?? 0 });
        break;
      }
    }
  }
  return blocks;
}

function selectorBeforeBlock(text, openBrace) {
  const previousClose = text.lastIndexOf('}', openBrace);
  const previousOpen = text.lastIndexOf('{', openBrace - 1);
  const previousBoundary = Math.max(previousClose, previousOpen);
  const prelude = text.slice(previousBoundary + 1, openBrace).trim();
  const declarationBoundary = prelude.lastIndexOf(';');
  return (declarationBoundary === -1 ? prelude : prelude.slice(declarationBoundary + 1)).trim();
}

function cssBlocks(text) {
  const stack = [];
  const blocks = [];
  for (let index = 0; index < text.length; index += 1) {
    const char = text[index];
    if (char === '{') {
      const selector = selectorBeforeBlock(text, index);
      const parent = stack[stack.length - 1];
      const parentContext = parent?.context ?? '';
      const parentSelectorKey = parent?.selectorKey ?? '';
      const context = selector.startsWith('@') ? `${parentContext}|${selector}` : parentContext;
      const selectorKey = selector.startsWith('@')
        ? parentSelectorKey
        : [parentSelectorKey, selector].filter(Boolean).join(' ');
      stack.push({ openBrace: index, selector, selectorKey, context });
    } else if (char === '}') {
      const block = stack.pop();
      if (!block) continue;
      blocks.push({
        text: text.slice(block.openBrace + 1, index),
        index: block.openBrace + 1,
        end: index,
        selector: block.selector,
        selectorKey: block.selectorKey,
        context: block.context,
      });
    }
  }
  return blocks.sort((a, b) => a.index - b.index);
}

function topLevelCssText(blockText) {
  let depth = 0;
  let result = '';
  for (const char of blockText) {
    if (char === '{') {
      depth += 1;
      result += ' ';
    } else if (char === '}') {
      depth = Math.max(0, depth - 1);
      result += ' ';
    } else {
      result += depth === 0 ? char : ' ';
    }
  }
  return result;
}

function keyframesLayoutFindings(rule, relativePath, text, lines, config) {
  const findings = [];
  const seenLines = new Set();
  const searchableText = stripCommentsPreserveLength(text);
  const layoutProperty = /\b(width|height|top|left|right|bottom|margin|padding|font-size|line-height)\s*:/g;
  for (const block of keyframesBlocks(searchableText)) {
    if (isIncidentalStringMatch(searchableText, block.index)) continue;
    layoutProperty.lastIndex = 0;
    for (const match of block.text.matchAll(layoutProperty)) {
      const line = lineForIndex(text, block.index + (match.index ?? 0));
      if (seenLines.has(line)) continue;
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        seenLines.add(line);
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
  }
  return findings;
}

function animationTimelineBeforeShorthandFindings(rule, relativePath, text, lines, config) {
  const searchableText = stripCommentsPreserveLength(text);
  const activeTimelineBySelector = new Map();
  const activeShorthandBySelector = new Map();
  const pendingResetBySelector = new Map();
  const findActiveTimeline = (selector, context) => {
    const candidates = [...activeTimelineBySelector.values()]
      .filter((entry) => selectorsOverlap(entry.selector, selector))
      .filter((entry) => (
        entry.context === context
        || entry.context === ''
        || context.startsWith(entry.context)
      ))
      .sort((a, b) => b.context.length - a.context.length);
    return candidates[0] ?? null;
  };
  const findMoreSpecificShorthand = (selector, context) => {
    const candidates = [...activeShorthandBySelector.values()]
      .filter((entry) => selectorsMoreSpecificOverlap(entry.selector, selector))
      .filter((entry) => (
        entry.context === context
        || entry.context === ''
        || context.startsWith(entry.context)
      ))
      .sort((a, b) => b.context.length - a.context.length);
    return candidates[0] ?? null;
  };
  for (const block of cssBlocks(searchableText)) {
    if (isIncidentalStringMatch(searchableText, block.index)) continue;
    const declarations = topLevelCssText(block.text);
    const timelineMatches = [...declarations.matchAll(/\b(?:animation-timeline|animationTimeline)\s*[:=]\s*[^;{},]+;?/g)];
    const shorthandMatches = [...declarations.matchAll(/\banimation\s*:/g)];
    const lastTimeline = timelineMatches[timelineMatches.length - 1] ?? null;
    const lastShorthand = shorthandMatches[shorthandMatches.length - 1] ?? null;
    const lastTimelineIndex = lastTimeline ? block.index + (lastTimeline.index ?? 0) : -1;
    const lastShorthandIndex = lastShorthand ? block.index + (lastShorthand.index ?? 0) : -1;
    const selectorKey = block.selectorKey || block.selector;
    const key = `${block.context}|${selectorKey}`;

    if (lastShorthand && lastShorthandIndex > lastTimelineIndex) {
      const active = findActiveTimeline(selectorKey, block.context);
      if (active && selectorKey) {
        pendingResetBySelector.set(key, { index: active.index });
      } else if (lastTimeline) {
        pendingResetBySelector.set(key, { index: lastTimelineIndex });
      }
    }

    if (!selectorKey) continue;
    if (lastTimeline && lastTimelineIndex > lastShorthandIndex) {
      pendingResetBySelector.delete(key);
      const moreSpecificShorthand = findMoreSpecificShorthand(selectorKey, block.context);
      if (moreSpecificShorthand) {
        pendingResetBySelector.set(key, { index: lastTimelineIndex });
      }
      activeTimelineBySelector.set(key, { selector: selectorKey, context: block.context, index: lastTimelineIndex });
    } else if (lastShorthand && lastShorthandIndex > lastTimelineIndex) {
      activeTimelineBySelector.delete(key);
      activeShorthandBySelector.set(key, { selector: selectorKey, context: block.context, index: lastShorthandIndex });
    }
  }
  return [...pendingResetBySelector.values()]
    .filter((entry) => {
      const line = lineForIndex(text, entry.index);
      return !isIgnored(config, rule.id, relativePath, lines, line);
    })
    .map((entry) => {
      const line = lineForIndex(text, entry.index);
      return makeFinding(rule, relativePath, line, excerptForLine(lines, line));
    });
}

function splitSelectorList(selector) {
  return selector
    .split(',')
    .map((part) => part.trim())
    .filter(Boolean);
}

function selectorsOverlap(left, right) {
  const leftSelectors = splitSelectorList(left);
  const rightSelectors = splitSelectorList(right);
  return leftSelectors.some((leftSelector) => rightSelectors.some((rightSelector) => (
    leftSelector === rightSelector
    || selectorRefinesSameElement(leftSelector, rightSelector)
    || selectorRefinesSameElement(rightSelector, leftSelector)
  )));
}

function selectorsMoreSpecificOverlap(candidate, base) {
  const candidateSelectors = splitSelectorList(candidate);
  const baseSelectors = splitSelectorList(base);
  return candidateSelectors.some((candidateSelector) => baseSelectors.some((baseSelector) => (
    candidateSelector !== baseSelector && selectorRefinesSameElement(baseSelector, candidateSelector)
  )));
}

function selectorRefinesSameElement(base, candidate) {
  if (!candidate.startsWith(base)) return false;
  const refinement = candidate.slice(base.length);
  return /^[:[.#]/.test(refinement) && !/[\s>+~]/.test(refinement);
}

const CSS_SCROLL_TIMELINE_PROPERTY_PATTERN = [
  'animation-(?:timeline|range(?:-(?:start|end))?)',
  'scroll-timeline(?:-(?:name|axis))?',
  'view-timeline(?:-(?:name|axis|inset))?',
  'timeline-scope',
].join('|');
const SCROLL_TIMELINE_PROPERTY_PATTERN = [
  CSS_SCROLL_TIMELINE_PROPERTY_PATTERN,
  'animation(?:Timeline|Range(?:Start|End)?)',
  'scrollTimeline(?:Name|Axis)?',
  'viewTimeline(?:Name|Axis|Inset)?',
  'timelineScope',
].join('|');
const CSS_STRING_QUOTE_PATTERN = "['\"`]";
const SCROLL_TIMELINE_PROPERTY_RE = new RegExp(`(?:${SCROLL_TIMELINE_PROPERTY_PATTERN})`);
const CSS_SCROLL_TIMELINE_PROPERTY_RE = new RegExp(`(?:${CSS_SCROLL_TIMELINE_PROPERTY_PATTERN})`);
const CSS_SCROLL_TIMELINE_SUPPORT_PROPERTY_RE = new RegExp(`(^|[^A-Za-z0-9_-])(?:${CSS_SCROLL_TIMELINE_PROPERTY_PATTERN})(?![A-Za-z0-9_-])`);
const SCROLL_TIMELINE_GUARD_GLOBAL_RE = /(?:CSS\.supports|@supports)\s*\(/g;
const SCROLL_TIMELINE_USAGE_RE = new RegExp(`(^|[^A-Za-z0-9_\\-'"\\\`])(${SCROLL_TIMELINE_PROPERTY_PATTERN})\\s*[:=]`, 'g');
const SCROLL_TIMELINE_QUOTED_KEY_RE = new RegExp(`${CSS_STRING_QUOTE_PATTERN}(${SCROLL_TIMELINE_PROPERTY_PATTERN})${CSS_STRING_QUOTE_PATTERN}\\s*:`, 'g');
const SCROLL_TIMELINE_SET_PROPERTY_RE = new RegExp(`\\bsetProperty\\s*\\(\\s*${CSS_STRING_QUOTE_PATTERN}(${CSS_SCROLL_TIMELINE_PROPERTY_PATTERN})${CSS_STRING_QUOTE_PATTERN}`, 'g');
const SCROLL_TIMELINE_BRACKET_PROPERTY_RE = new RegExp(`\\[\\s*${CSS_STRING_QUOTE_PATTERN}(${SCROLL_TIMELINE_PROPERTY_PATTERN})${CSS_STRING_QUOTE_PATTERN}\\s*\\]\\s*=`, 'g');

function stripCommentsPreserveLength(text) {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, (match) => match.replace(/[^\n\r]/g, ' '))
    .replace(/(^|[\s;{}])\/\/[^\n\r]*/g, (match, prefix) => prefix + ' '.repeat(match.length - prefix.length));
}

function stripComments(text) {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, ' ')
    .replace(/(^|[\s;{}])\/\/[^\n\r]*/g, (match, prefix) => `${prefix} `);
}

function isSupportedStringGuard(text, span) {
  const stringStart = span.start;
  const prefix = text.slice(Math.max(0, stringStart - 80), stringStart);
  if (/\b(matchMedia|CSS\.supports|useReducedMotion|AccessibilityInfo|ReducedMotion)\s*\(?\s*$/.test(prefix)) return true;
  const content = text.slice(span.start + 1, span.end);
  return /prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|reduceMotion/.test(content)
    && isSupportedStringInclude(text, span);
}

function isSupportedStringInclude(text, span) {
  const prefix = text.slice(Math.max(0, span.start - 80), span.start);
  const content = text.slice(span.start + 1, span.end);
  if (/\b(className|class|style|styles|css|styled(?:\.\w+)?)\s*[:=]?\s*$/.test(prefix)) return true;
  return text[span.start] === '`' && (
    /@keyframes|animation|transition|animate-|motion\.|gsap\.|<motion/.test(content)
    || SCROLL_TIMELINE_PROPERTY_RE.test(content)
  );
}

function stringSpans(text) {
  const spans = [];
  for (let index = 0; index < text.length; index += 1) {
    const quote = text[index];
    if (quote !== '"' && quote !== "'" && quote !== '`') continue;
    let escaped = false;
    for (let end = index + 1; end < text.length; end += 1) {
      if (escaped) {
        escaped = false;
      } else if (text[end] === '\\') {
        escaped = true;
      } else if (text[end] === quote) {
        spans.push({ start: index, end });
        index = end;
        break;
      } else if (quote !== '`' && (text[end] === '\n' || text[end] === '\r')) {
        break;
      }
      if (end === text.length - 1) {
        spans.push({ start: index, end: text.length });
        index = end;
      }
    }
  }
  return spans;
}

function hasWithoutEvidence(text, regex) {
  const spans = stringSpans(text);
  for (const match of text.matchAll(regex)) {
    const index = match.index ?? 0;
    const span = spans.find((candidate) => index > candidate.start && index < candidate.end);
    if (!span || isSupportedStringGuard(text, span)) return true;
  }
  return false;
}

function discreteEvidenceRegexForMatch(matchText) {
  if (/@starting-style/.test(matchText)) return /@starting-style/;
  if (/\btransition\s*:/.test(matchText) && /\ballow-discrete\b/.test(matchText)) {
    return /(?:transition-behavior|\btransition\b[\s:'",]+[^)\n;{]*\ballow-discrete\b)/;
  }
  return /transition-behavior/;
}

function hasPositiveDiscreteSupportEvidence(text, evidenceRegex) {
  for (const match of text.matchAll(/@supports\b/g)) {
    const index = match.index ?? 0;
    if (isIncidentalStringMatch(text, index)) continue;
    const openBraceIndex = text.indexOf('{', index);
    if (openBraceIndex === -1) continue;
    const condition = text.slice(index + match[0].length, openBraceIndex);
    if (isPositiveDiscreteSupportsCondition(condition, evidenceRegex)) return true;
  }
  for (const match of text.matchAll(/CSS\.supports\s*\(/g)) {
    const index = match.index ?? 0;
    if (isInsideString(text, index)) continue;
    const openIndex = text.indexOf('(', index);
    const closeIndex = openIndex === -1 ? -1 : findMatchingParen(text, openIndex);
    if (closeIndex === -1) continue;
    const callText = text.slice(index, closeIndex + 1);
    if (!evidenceRegex.test(callText)) continue;
    const prefix = text.slice(Math.max(0, index - 16), index);
    const suffix = text.slice(closeIndex + 1, closeIndex + 24);
    const negated = /!\s*\(*\s*$/.test(prefix)
      || /\bfalse\s*={2,3}\s*$/.test(prefix)
      || /\btrue\s*!={1,2}\s*$/.test(prefix)
      || /^\s*={2,3}\s*false\b/.test(suffix)
      || /^\s*!={1,2}\s*true\b/.test(suffix);
    const statement = statementAroundCssSupports(text, index, closeIndex);
    const disjunctive = hasUnsafeCssSupportsDisjunction(
      statement.text,
      index - statement.start,
      evidenceRegex,
    );
    if (!negated && !disjunctive) return true;
  }
  return false;
}

function isPositiveDiscreteSupportsCondition(condition, evidenceRegex) {
  return evidenceRegex.test(condition)
    && !/^(\s*not\b|\s*\(\s*not\b)/i.test(condition)
    && !/\bnot\s*\(/i.test(condition)
    && !hasUnsafeFeatureConditionOr(condition, evidenceRegex);
}

function hasDiscreteSupportPolicyEvidence(text) {
  return hasWithoutEvidence(text, /browser support|Baseline 2024|support policy/gi);
}

function isDiscreteUseGuarded(text, index, evidenceRegex) {
  return enclosingBlockPreludes(text, index)
    .some((prelude) => isDiscreteGuardBlockPrelude(prelude, evidenceRegex));
}

function isInsideSupportsPrelude(text, index) {
  const supportsIndex = text.lastIndexOf('@supports', index);
  if (supportsIndex === -1) return false;
  const openBraceIndex = text.indexOf('{', supportsIndex);
  return openBraceIndex !== -1 && supportsIndex < index && index < openBraceIndex;
}

function isDiscreteGuardBlockPrelude(prelude, evidenceRegex) {
  const clean = stripComments(prelude).trim();
  if (clean.startsWith('@supports')) {
    const condition = clean.replace(/^@supports\s*/i, '').trim();
    return isPositiveDiscreteSupportsCondition(condition, evidenceRegex);
  }
  return /\bif\s*\(/.test(clean) && hasPositiveDiscreteSupportEvidence(clean, evidenceRegex);
}

function statementAroundCssSupports(text, supportsIndex, closeIndex) {
  const before = [
    text.lastIndexOf('\n', supportsIndex),
    text.lastIndexOf(';', supportsIndex),
    text.lastIndexOf('{', supportsIndex),
    text.lastIndexOf('}', supportsIndex),
  ];
  const after = [
    text.indexOf('\n', closeIndex + 1),
    text.indexOf(';', closeIndex + 1),
    text.indexOf('{', closeIndex + 1),
  ].filter((index) => index !== -1);
  const start = Math.max(...before) + 1;
  const end = after.length > 0 ? Math.min(...after) : text.length;
  return { text: text.slice(start, end), start };
}

function isIncidentalStringMatch(text, index) {
  const span = stringSpans(text).find((candidate) => index > candidate.start && index < candidate.end);
  return Boolean(span && !isSupportedStringInclude(text, span));
}

function isInsideString(text, index) {
  return stringSpans(text).some((candidate) => index > candidate.start && index < candidate.end);
}

function isStandalonePropertyString(text, span) {
  const content = text.slice(span.start + 1, span.end);
  if (!SCROLL_TIMELINE_PROPERTY_RE.test(content)) return false;
  const after = text.slice(span.end + 1, span.end + 16);
  const before = text.slice(Math.max(0, span.start - 32), span.start);
  return /^\s*[:\],)]/.test(after) || /\bsetProperty\s*\(\s*$/.test(before);
}

function isIncidentalPropertyStringMatch(text, index) {
  const containing = stringSpans(text).filter((candidate) => index > candidate.start && index < candidate.end);
  if (containing.length === 0) return false;
  return containing.some((span) => !isSupportedStringInclude(text, span) && !isStandalonePropertyString(text, span));
}

function isScrollTimelineGuardPrelude(prelude) {
  const clean = stripComments(prelude).trim();
  if (clean.startsWith('@supports')) {
    const condition = clean.replace(/^@supports\s*/i, '').trim();
    const negatesTimelineFeature = /^not\b/i.test(condition) || /^\(\s*not\b/i.test(condition) || /\bnot\s*\(/i.test(condition);
    const disjunctiveTimelineFeature = hasUnsafeCssSupportsConditionOr(condition);
    return !negatesTimelineFeature && !disjunctiveTimelineFeature && CSS_SCROLL_TIMELINE_SUPPORT_PROPERTY_RE.test(clean);
  }
  const supportsIndex = clean.search(/\bCSS\.supports\s*\(/);
  const negatesCssSupports = supportsIndex !== -1 && (
    /!\s*\(*\s*$/.test(clean.slice(0, supportsIndex))
    || /\)\s*={2,3}\s*false\b/.test(clean.slice(supportsIndex))
    || /\)\s*!={1,2}\s*true\b/.test(clean.slice(supportsIndex))
    || /\bfalse\s*={2,3}\s*CSS\.supports\s*\(/.test(clean)
    || /\btrue\s*!={1,2}\s*CSS\.supports\s*\(/.test(clean)
  );
  const supportsExpression = supportsIndex === -1 ? '' : clean.slice(supportsIndex);
  const disjunctiveCssSupports = hasUnsafeCssSupportsDisjunction(clean, supportsIndex, CSS_SCROLL_TIMELINE_PROPERTY_RE);
  return supportsIndex !== -1 && !negatesCssSupports && !disjunctiveCssSupports && CSS_SCROLL_TIMELINE_SUPPORT_PROPERTY_RE.test(supportsExpression);
}

function parenDepthAt(text, offset) {
  let depth = 0;
  for (let index = 0; index < offset; index += 1) {
    if (text[index] === '(') depth += 1;
    else if (text[index] === ')') depth = Math.max(0, depth - 1);
  }
  return depth;
}

function hasTopLevelAndBetween(text, start, end, maxDepth) {
  let depth = parenDepthAt(text, start);
  for (let index = start; index < end - 1; index += 1) {
    if (text[index] === '(') depth += 1;
    else if (text[index] === ')') depth = Math.max(0, depth - 1);
    else if (text[index] === '&' && text[index + 1] === '&' && depth <= maxDepth) return true;
  }
  return false;
}

function hasUnsafeCssSupportsDisjunction(text, supportsIndex, featureRegex = null) {
  if (supportsIndex === -1) return false;
  const openIndex = text.indexOf('(', supportsIndex);
  const closeIndex = openIndex === -1 ? -1 : findMatchingParen(text, openIndex);
  const supportsArgs = openIndex === -1 || closeIndex === -1 ? '' : text.slice(openIndex + 1, closeIndex);
  if (featureRegex ? hasUnsafeFeatureConditionOr(supportsArgs, featureRegex) : /\bor\b/i.test(supportsArgs)) return true;
  const supportsDepth = parenDepthAt(text, supportsIndex);
  for (let index = text.indexOf('||'); index !== -1; index = text.indexOf('||', index + 2)) {
    if (parenDepthAt(text, index) <= supportsDepth) return true;
  }
  return false;
}

function hasUnsafeCssSupportsConditionOr(condition) {
  return hasUnsafeFeatureConditionOr(condition, CSS_SCROLL_TIMELINE_PROPERTY_RE);
}

function hasUnsafeFeatureConditionOr(condition, featureRegex) {
  const featureMatch = condition.match(featureRegex);
  if (!featureMatch || featureMatch.index === undefined) return false;
  const featureIndex = featureMatch.index;
  const featureDepth = parenDepthAt(condition, featureIndex);
  for (const match of condition.matchAll(/\bor\b/gi)) {
    const orIndex = match.index ?? 0;
    const orDepth = parenDepthAt(condition, orIndex);
    if (orBranchesContainFeature(condition, orIndex, orDepth, featureRegex)) continue;
    if (orDepth < featureDepth) return true;
    if (orDepth === featureDepth) {
      const start = Math.min(orIndex + match[0].length, featureIndex);
      const end = Math.max(orIndex, featureIndex);
      if (!hasLowerDepthCssAndBetween(condition, start, end, featureDepth)) return true;
    }
  }
  return false;
}

function orBranchesContainFeature(condition, orIndex, orDepth, featureRegex) {
  const leftStart = branchStartForOr(condition, orIndex, orDepth);
  const rightEnd = branchEndForOr(condition, orIndex, orDepth);
  const left = condition.slice(leftStart, orIndex);
  const right = condition.slice(orIndex + 2, rightEnd);
  return featureRegex.test(left) && featureRegex.test(right);
}

function branchStartForOr(condition, orIndex, orDepth) {
  let start = 0;
  for (const match of condition.slice(0, orIndex).matchAll(/\bor\b/gi)) {
    const index = match.index ?? 0;
    if (parenDepthAt(condition, index) === orDepth) start = index + match[0].length;
  }
  for (let index = orIndex - 1; index >= 0; index -= 1) {
    if (parenDepthAt(condition, index) < orDepth) return Math.max(start, index + 1);
  }
  return start;
}

function branchEndForOr(condition, orIndex, orDepth) {
  let end = condition.length;
  const afterOr = orIndex + 2;
  for (const match of condition.slice(afterOr).matchAll(/\bor\b/gi)) {
    const index = afterOr + (match.index ?? 0);
    if (parenDepthAt(condition, index) === orDepth) {
      end = index;
      break;
    }
  }
  for (let index = afterOr; index < end; index += 1) {
    if (parenDepthAt(condition, index) < orDepth) return index;
  }
  return end;
}

function hasLowerDepthCssAndBetween(text, start, end, depth) {
  for (const match of text.slice(start, end).matchAll(/\band\b/gi)) {
    const index = start + (match.index ?? 0);
    if (parenDepthAt(text, index) < depth) return true;
  }
  return false;
}

function isScrollTimelineGuardBlockPrelude(prelude) {
  const clean = stripComments(prelude).trim();
  if (clean.startsWith('@supports')) return isScrollTimelineGuardPrelude(clean);
  return /\bif\s*\(/.test(clean) && isScrollTimelineGuardPrelude(clean);
}

function enclosingBlockPreludes(text, index) {
  const searchableText = stripCommentsPreserveLength(text);
  const stack = [];
  for (let cursor = 0; cursor <= index && cursor < searchableText.length; cursor += 1) {
    if (searchableText[cursor] === '{') {
      const previousClose = searchableText.lastIndexOf('}', cursor);
      const previousOpen = searchableText.lastIndexOf('{', cursor - 1);
      stack.push(searchableText.slice(Math.max(previousClose, previousOpen) + 1, cursor).trim());
    } else if (searchableText[cursor] === '}') {
      stack.pop();
    }
  }
  return stack;
}

function hasGuardOnSameLineBeforeIndex(text, index) {
  const lineStart = text.lastIndexOf('\n', index) + 1;
  const lineEnd = text.indexOf('\n', index);
  const line = stripCommentsPreserveLength(text.slice(lineStart, lineEnd === -1 ? text.length : lineEnd));
  const offset = index - lineStart;
  for (const match of line.matchAll(SCROLL_TIMELINE_GUARD_GLOBAL_RE)) {
    const guardIndex = match.index ?? 0;
    if (guardIndex > offset) continue;
    const guardContextStart = Math.max(line.lastIndexOf(';', guardIndex), line.lastIndexOf('{', guardIndex), line.lastIndexOf('}', guardIndex)) + 1;
    if (!isScrollTimelineGuardPrelude(line.slice(guardContextStart, offset))) continue;
    const betweenGuardAndUse = line.slice(guardIndex + match[0].length, offset);
    if (betweenGuardAndUse.includes('||')) continue;
    if (/[{}]/.test(betweenGuardAndUse)) continue;
    const questionIndex = betweenGuardAndUse.indexOf('?');
    if (questionIndex !== -1 && betweenGuardAndUse.indexOf(':', questionIndex) !== -1) continue;
    if (!betweenGuardAndUse.includes(';')) return true;
  }
  return false;
}

function findMatchingParen(text, openIndex) {
  let depth = 0;
  for (let cursor = openIndex; cursor < text.length; cursor += 1) {
    if (text[cursor] === '(') {
      depth += 1;
    } else if (text[cursor] === ')') {
      depth -= 1;
      if (depth === 0) return cursor;
    }
  }
  return -1;
}

function supportConditionBounds(text, index) {
  const masked = stripCommentsPreserveLength(text);
  const supportsIndex = Math.max(masked.lastIndexOf('@supports', index), masked.lastIndexOf('CSS.supports', index));
  if (supportsIndex === -1) return null;
  if (masked.startsWith('@supports', supportsIndex)) {
    const closeIndex = masked.indexOf('{', supportsIndex);
    if (closeIndex === -1) return null;
    return { masked, supportsIndex, openIndex: supportsIndex, closeIndex };
  }
  const openIndex = masked.indexOf('(', supportsIndex);
  if (openIndex === -1) return null;
  const closeIndex = findMatchingParen(masked, openIndex);
  return { masked, supportsIndex, openIndex, closeIndex };
}

function hasScrollTimelineGuard(text, index, blocks) {
  if (hasGuardOnSameLineBeforeIndex(text, index)) {
    return true;
  }
  if (hasUnbracedScrollTimelineGuard(text, index)) {
    return true;
  }
  if (enclosingBlockPreludes(text, index).some((prelude) => isScrollTimelineGuardBlockPrelude(prelude))) {
    return true;
  }
  return blocks.some((block) => (
    index >= block.index
    && index <= block.end
    && block.context.split('|').some((prelude) => isScrollTimelineGuardPrelude(prelude))
  ));
}

function isInsideSupportCondition(text, index) {
  const bounds = supportConditionBounds(text, index);
  if (!bounds) return false;
  return bounds.closeIndex !== -1 && index > bounds.openIndex && index < bounds.closeIndex;
}

function hasUnbracedScrollTimelineGuard(text, index) {
  const bounds = supportConditionBounds(text, index);
  if (!bounds || bounds.closeIndex === -1 || bounds.closeIndex >= index) return false;
  if (bounds.masked.slice(bounds.supportsIndex, bounds.closeIndex + 1).startsWith('@supports')) return false;
  const preludeStart = Math.max(
    bounds.masked.lastIndexOf('\n', bounds.supportsIndex),
    bounds.masked.lastIndexOf(';', bounds.supportsIndex),
    bounds.masked.lastIndexOf('{', bounds.supportsIndex),
    bounds.masked.lastIndexOf('}', bounds.supportsIndex),
  ) + 1;
  const prelude = bounds.masked.slice(preludeStart, bounds.closeIndex + 1);
  if (!/\bif\s*\(/.test(prelude)) return false;
  if (!isScrollTimelineGuardPrelude(prelude)) return false;
  const betweenConditionAndUse = bounds.masked.slice(bounds.closeIndex + 1, index);
  if (/[{;]/.test(betweenConditionAndUse)) return false;
  const earlierLines = betweenConditionAndUse.split('\n').slice(0, -1);
  return earlierLines.every((line) => line.replace(/[)\s]/g, '') === '');
}

function scrollTimelineFindings(rule, relativePath, text, lines, config) {
  const findings = [];
  const seenLines = new Set();
  const blocks = cssBlocks(text);
  const searchableText = stripCommentsPreserveLength(text);
  const matches = [
    ...[...searchableText.matchAll(SCROLL_TIMELINE_USAGE_RE)]
      .map((match) => (match.index ?? 0) + match[0].lastIndexOf(match[2]))
      .filter((index) => !isIncidentalStringMatch(searchableText, index)),
    ...[...searchableText.matchAll(SCROLL_TIMELINE_QUOTED_KEY_RE)]
      .map((match) => (match.index ?? 0) + match[0].lastIndexOf(match[1]))
      .filter((index) => !isIncidentalPropertyStringMatch(searchableText, index)),
    ...[...searchableText.matchAll(SCROLL_TIMELINE_SET_PROPERTY_RE)]
      .map((match) => (match.index ?? 0) + match[0].lastIndexOf(match[1]))
      .filter((index) => !isIncidentalPropertyStringMatch(searchableText, index)),
    ...[...searchableText.matchAll(SCROLL_TIMELINE_BRACKET_PROPERTY_RE)]
      .map((match) => (match.index ?? 0) + match[0].lastIndexOf(match[1]))
      .filter((index) => !isIncidentalPropertyStringMatch(searchableText, index)),
  ].sort((a, b) => a - b);
  for (const index of matches) {
    const line = lineForIndex(text, index);
    if (isInsideSupportCondition(text, index)) continue;
    if (hasScrollTimelineGuard(text, index, blocks)) continue;
    if (seenLines.has(line)) continue;
    if (!isIgnored(config, rule.id, relativePath, lines, line)) {
      seenLines.add(line);
      findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
    }
  }
  return findings;
}

function customPropertyAnimationFindings(rule, relativePath, text, lines, config) {
  const searchableText = stripCommentsPreserveLength(text);
  const registered = registeredCustomProperties(searchableText);
  const findings = [];
  const seen = new Set();
  const seenLines = new Set();
  const addFinding = (name, index, options = {}) => {
    if (registered.has(name) || seen.has(name)) return;
    if (!options.allowStringValue && isIncidentalStringMatch(searchableText, index)) return;
    const line = lineForIndex(text, index);
    if (seenLines.has(line)) return;
    if (isIgnored(config, rule.id, relativePath, lines, line)) return;
    seen.add(name);
    seenLines.add(line);
    findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
  };

  const addPropertyValueFindings = (value, baseIndex, options = {}) => {
    for (const match of value.matchAll(/(?:^|[\s,])(--[a-zA-Z0-9_-]+)\b/g)) {
      addFinding(match[1], baseIndex + (match.index ?? 0) + match[0].lastIndexOf(match[1]), options);
    }
  };
  const topLevelTokens = (value) => {
    const tokens = [];
    let start = null;
    let depth = 0;
    for (let index = 0; index <= value.length; index += 1) {
      const char = value[index] ?? ' ';
      if (char === '(') depth += 1;
      else if (char === ')' && depth > 0) depth -= 1;
      const boundary = index === value.length || (depth === 0 && /\s/.test(char));
      if (boundary) {
        if (start != null) tokens.push({ text: value.slice(start, index), index: start });
        start = null;
      } else if (start == null) start = index;
    }
    return tokens;
  };
  const isTransitionPropertyToken = (token) =>
    !/^(?:\d*\.?\d+m?s|ease|ease-in|ease-out|ease-in-out|linear|step-start|step-end|allow-discrete|normal|none|all|inherit|initial|unset|revert|revert-layer)$/i.test(token) &&
    !/^(?:cubic-bezier|steps|linear|var|min|max|clamp)\(/i.test(token);
  const addShorthandValueFindings = (value, baseIndex, options = {}) => {
    let partStart = 0;
    for (const part of value.split(',')) {
      const property = topLevelTokens(part).find((token) => isTransitionPropertyToken(token.text) && /^--[a-zA-Z0-9_-]+\b/.test(token.text));
      if (property) addFinding(property.text.match(/^--[a-zA-Z0-9_-]+\b/)?.[0], baseIndex + partStart + property.index, options);
      partStart += part.length + 1;
    }
  };

  const transitionPropertyDeclarations = [
    /\btransition-property\s*:([^;{}]*)/gms,
    /\btransitionProperty\s*[:=]\s*\{?\s*(?:['"`]([^'"`{};]*)['"`]|([^,;{}\n]*))/gms,
  ];
  for (const declarationRegex of transitionPropertyDeclarations) {
    for (const declaration of searchableText.matchAll(declarationRegex)) {
      if (isIncidentalStringMatch(searchableText, declaration.index ?? 0)) continue;
      const value = declaration[1] ?? declaration[2] ?? declaration[3] ?? '';
      if (!value) continue;
      const baseIndex = (declaration.index ?? 0) + declaration[0].indexOf(value);
      addPropertyValueFindings(value, baseIndex, { allowStringValue: true });
    }
  }
  const transitionShorthandDeclarations = [
    /\btransition\s*:([^;{}]*)/gms,
    /\btransition\s*[:=]\s*\{?\s*['"`]([^'"`{};]*)['"`]/gms,
  ];
  for (const declarationRegex of transitionShorthandDeclarations) {
    for (const declaration of searchableText.matchAll(declarationRegex)) {
      if (isIncidentalStringMatch(searchableText, declaration.index ?? 0)) continue;
      const value = declaration[1] ?? '';
      if (!value) continue;
      const baseIndex = (declaration.index ?? 0) + declaration[0].indexOf(value);
      addShorthandValueFindings(value, baseIndex, { allowStringValue: true });
    }
  }

  for (const block of keyframesBlocks(searchableText)) {
    const blockText = block.text;
    const baseIndex = block.index;
    for (const match of blockText.matchAll(/(?:^|[;{}\s])(--[a-zA-Z0-9_-]+)\s*:/gms)) {
      addFinding(match[1], baseIndex + (match.index ?? 0));
    }
  }

  return findings;
}

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
  if (rule.kind === 'keyframesLayoutProperty') {
    return keyframesLayoutFindings(rule, relativePath, text, lines, config);
  }
  if (rule.kind === 'animationTimelineBeforeShorthand') {
    return animationTimelineBeforeShorthandFindings(rule, relativePath, text, lines, config);
  }
  if (rule.kind === 'scrollTimelineWithoutSupport') {
    return scrollTimelineFindings(rule, relativePath, text, lines, config);
  }
  if (rule.kind === 'customPropertyAnimationUnregistered') {
    return customPropertyAnimationFindings(rule, relativePath, text, lines, config);
  }
  if (rule.kind === 'fileContainsWithout' || rule.kind === 'fileContainsBoth' || rule.kind === 'fileContainsBothWithout') {
    const seenLines = new Set();
    const guardText = rule.without ? stripCommentsPreserveLength(text) : text;
    const searchableText = stripCommentsPreserveLength(text);
    const includeMatches = [...searchableText.matchAll(ruleRegex(rule.include))];
    const alsoMatch = rule.also ? ruleRegex(rule.also).exec(searchableText) : null;
    const withoutMatch = rule.id === 'motion.discrete-transition-without-supports'
      ? null
      : rule.without ? hasWithoutEvidence(guardText, ruleRegex(rule.without)) : null;
    const discreteSupportPolicy = rule.id === 'motion.discrete-transition-without-supports'
      ? hasDiscreteSupportPolicyEvidence(text)
      : false;
    for (const includeMatch of includeMatches) {
      const includeIndex = includeMatch.index ?? 0;
      if (rule.id === 'motion.discrete-transition-without-supports' && isInsideSupportsPrelude(guardText, includeIndex)) continue;
      const includeWithoutMatch = rule.id === 'motion.discrete-transition-without-supports'
        ? discreteSupportPolicy || isDiscreteUseGuarded(
          guardText,
          includeIndex,
          discreteEvidenceRegexForMatch(includeMatch[0] ?? ''),
        )
        : withoutMatch;
      const matches =
        rule.kind === 'fileContainsBoth'
          ? alsoMatch
          : rule.kind === 'fileContainsBothWithout'
            ? alsoMatch && !includeWithoutMatch
            : (!rule.also || alsoMatch) && !includeWithoutMatch;
      if (!matches) continue;
      if (isIncidentalStringMatch(searchableText, includeIndex)) continue;
      const line = lineForIndex(text, includeIndex);
      if (seenLines.has(line)) continue;
      if (!isIgnored(config, rule.id, relativePath, lines, line)) {
        seenLines.add(line);
        findings.push(makeFinding(rule, relativePath, line, excerptForLine(lines, line)));
      }
    }
    return findings;
  }
  if (rule.kind === 'packageHasAny') return findings;
  const regex = ruleRegex(rule.pattern);
  const searchableText = stripCommentsPreserveLength(text);
  for (const match of searchableText.matchAll(regex)) {
    if (isIncidentalStringMatch(searchableText, match.index ?? 0)) continue;
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
      if (text.slice(0, 512).includes(fileSkipMarker)) continue;
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
