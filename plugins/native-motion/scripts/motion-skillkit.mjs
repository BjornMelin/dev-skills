#!/usr/bin/env node
// motion-audit-skip-file: this helper contains scanner rule strings.
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';

const pluginRoot = path.resolve(import.meta.dirname, '..');
const skillsRoot = path.join(pluginRoot, 'skills');
const pluginName = readPluginName();
const args = process.argv.slice(2);
const localPathPattern = new RegExp([
  '\\.' + 'firecrawl',
  '\\/home\\/[^\\/\\s]+',
  '\\/Users\\/[^\\/\\s]+',
  '[A-Za-z]:\\/[^\\/\\s]+',
  '[A-Za-z]:\\/Users\\/[^\\/\\s]+',
  '~\\/' + 'repos\\/agents',
  '\\/tmp\\/' + 'motion',
].join('|'));

function usage() {
  return 'Usage: node scripts/motion-skillkit.mjs <command> [options]\n\n' +
    'Commands:\n' +
    '  doctor                 Report plugin, skill, metadata, and resource state.\n' +
    '  sources verify         Verify source ledgers and provenance files.\n' +
    '  eval --all             Validate eval and trigger-query schemas.\n' +
    '  scan --root <path>     Static scan for common motion review issues.\n' +
    '  validate-atomic        Run doctor, resources, sources, assets, and eval schema checks.\n\n' +
    'Options:\n' +
    '  --format json|markdown Output format. Default: markdown.\n' +
    '  --json                 Alias for --format json.\n' +
    '  --output <file>        Write output to file instead of stdout.\n' +
    '  --root <path>          Scan root for scan command. Default: current directory.\n' +
    '  --max-files <n>        Max files for scan. Default: 2000.\n' +
    '  --help                 Show help.\n';
}
function readPluginName() {
  try {
    const manifest = JSON.parse(readFileSync(path.join(pluginRoot, '.codex-plugin', 'plugin.json'), 'utf8'));
    return manifest.name || path.basename(pluginRoot);
  } catch {
    return path.basename(pluginRoot);
  }
}

function portableText(value) {
  return String(value).replace(/\\+/g, '/');
}

function option(name, fallback) {
  const index = args.indexOf(name);
  if (index >= 0 && args[index + 1]) return args[index + 1];
  return fallback;
}
function has(flag) { return args.includes(flag); }
function format() { return has('--json') ? 'json' : option('--format', 'markdown'); }
function out(result) {
  const body = format() === 'json' ? JSON.stringify(result, null, 2) + '\n' : toMarkdown(result);
  const output = option('--output', null);
  if (output) {
    mkdirSync(path.dirname(path.resolve(output)), { recursive: true });
    writeFileSync(output, body);
  } else {
    process.stdout.write(body);
  }
}
function fail(message, details = {}) {
  out({ ok: false, plugin: pluginName, error: { message, ...details } });
  process.exit(1);
}
function read(file) { return readFileSync(file, 'utf8'); }
function lineForIndex(text, index) {
  return text.slice(0, index).split('\n').length;
}
function excerptForLine(lines, lineNumber) {
  return (lines[lineNumber - 1] ?? '').trim().slice(0, 180);
}
function registeredCustomProperties(text) {
  const names = new Set();
  for (const match of text.matchAll(/@property\s+(--[a-zA-Z0-9_-]+)/g)) {
    if (!isIncidentalStringMatch(text, match.index ?? 0)) names.add(match[1]);
  }
  for (const match of text.matchAll(/registerProperty\s*\(\s*\{[\s\S]{0,400}?(?:\bname|['"`]name['"`])\s*:\s*['"`](--[a-zA-Z0-9_-]+)['"`]/g)) {
    if (!isIncidentalStringMatch(text, match.index ?? 0)) names.add(match[1]);
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
function addCustomPropertyFindings(text, lines, scanRoot, file, findings) {
  const searchableText = stripCommentsPreserveLength(text);
  const registered = registeredCustomProperties(searchableText);
  const seen = new Set();
  const seenLines = new Set();
  const addFinding = (name, index, options = {}) => {
    if (registered.has(name) || seen.has(name)) return;
    if (!options.allowStringValue && isIncidentalStringMatch(searchableText, index)) return;
    const line = lineForIndex(text, index);
    if (seenLines.has(line)) return;
    if (lines[line - 1]?.includes('motion-audit-ignore')) return;
    seen.add(name);
    seenLines.add(line);
    findings.push({
      id: 'custom-property-animation',
      file: path.relative(scanRoot, file),
      line,
      excerpt: excerptForLine(lines, line),
      hint: 'Register animated CSS custom properties with @property or CSS.registerProperty().',
    });
  };
  const addPropertyValueFindings = (value, baseIndex, options = {}) => {
    for (const match of value.matchAll(/(?:^|[\s,])(--[a-zA-Z0-9_-]+)\b/g)) {
      addFinding(match[1], baseIndex + (match.index ?? 0) + match[0].lastIndexOf(match[1]), options);
    }
  };
  const addShorthandValueFindings = (value, baseIndex, options = {}) => {
    let partStart = 0;
    for (const part of value.split(',')) {
      const property = part.match(/(?:^|\s)(--[a-zA-Z0-9_-]+)\b/);
      if (property) addFinding(property[1], baseIndex + partStart + (property.index ?? 0) + property[0].lastIndexOf(property[1]), options);
      partStart += part.length + 1;
    }
  };
  const propertyDeclarations = [
    /\btransition-property\s*:([^;{}]*)/gms,
    /\btransitionProperty\s*[:=]\s*\{?\s*(?:['"`]([^'"`{};]*)['"`]|([^,;{}\n]*))/gms,
  ];
  for (const declarationRegex of propertyDeclarations) {
    for (const declaration of searchableText.matchAll(declarationRegex)) {
      if (isIncidentalStringMatch(searchableText, declaration.index ?? 0)) continue;
      const value = declaration[1] ?? declaration[2] ?? declaration[3] ?? '';
      if (!value) continue;
      const baseIndex = (declaration.index ?? 0) + declaration[0].indexOf(value);
      addPropertyValueFindings(value, baseIndex, { allowStringValue: true });
    }
  }
  const shorthandDeclarations = [
    /\btransition\s*:([^;{}]*)/gms,
    /\btransition\s*[:=]\s*\{?\s*['"`]([^'"`{};]*)['"`]/gms,
  ];
  for (const declarationRegex of shorthandDeclarations) {
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
function stringSpans(text) {
  const spans = [];
  for (let index = 0; index < text.length; index += 1) {
    const quote = text[index];
    if (quote !== '"' && quote !== "'" && quote !== '`') continue;
    let escaped = false;
    for (let end = index + 1; end < text.length; end += 1) {
      if (escaped) escaped = false;
      else if (text[end] === '\\') escaped = true;
      else if (text[end] === quote) {
        spans.push({ start: index, end });
        index = end;
        break;
      } else if (quote !== '`' && (text[end] === '\n' || text[end] === '\r')) break;
      if (end === text.length - 1) {
        spans.push({ start: index, end: text.length });
        index = end;
      }
    }
  }
  return spans;
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
function isIncidentalStringMatch(text, index) {
  const span = stringSpans(text).find((candidate) => index > candidate.start && index < candidate.end);
  return Boolean(span && !isSupportedStringInclude(text, span));
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
  if (featureRegex ? hasUnsafeCssSupportsConditionOr(supportsArgs) : /\bor\b/i.test(supportsArgs)) return true;
  const supportsDepth = parenDepthAt(text, supportsIndex);
  for (let index = text.indexOf('||'); index !== -1; index = text.indexOf('||', index + 2)) {
    if (parenDepthAt(text, index) <= supportsDepth) return true;
  }
  return false;
}
function hasUnsafeCssSupportsConditionOr(condition) {
  const timelineMatch = condition.match(CSS_SCROLL_TIMELINE_PROPERTY_RE);
  if (!timelineMatch || timelineMatch.index === undefined) return false;
  const timelineIndex = timelineMatch.index;
  const timelineDepth = parenDepthAt(condition, timelineIndex);
  for (const match of condition.matchAll(/\bor\b/gi)) {
    const orIndex = match.index ?? 0;
    const orDepth = parenDepthAt(condition, orIndex);
    if (orBranchesContainFeature(condition, orIndex, orDepth, CSS_SCROLL_TIMELINE_PROPERTY_RE)) continue;
    if (orDepth < timelineDepth) return true;
    if (orDepth === timelineDepth) {
      const start = Math.min(orIndex + match[0].length, timelineIndex);
      const end = Math.max(orIndex, timelineIndex);
      if (!hasLowerDepthCssAndBetween(condition, start, end, timelineDepth)) return true;
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

function hasNearbyScrollTimelineGuard(text, index) {
  if (hasGuardOnSameLineBeforeIndex(text, index)) return true;
  if (hasUnbracedScrollTimelineGuard(text, index)) return true;
  const searchableText = stripCommentsPreserveLength(text);
  const stack = [];
  for (let cursor = 0; cursor <= index && cursor < searchableText.length; cursor += 1) {
    if (searchableText[cursor] === '{') {
      const previousClose = searchableText.lastIndexOf('}', cursor);
      const previousOpen = searchableText.lastIndexOf('{', cursor - 1);
      const prelude = searchableText.slice(Math.max(previousClose, previousOpen) + 1, cursor).trim();
      stack.push(prelude);
    } else if (searchableText[cursor] === '}') {
      stack.pop();
    }
  }
  return stack.some((prelude) => isScrollTimelineGuardBlockPrelude(prelude));
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
function addScrollTimelineFindings(text, lines, scanRoot, file, findings) {
  const seenLines = new Set();
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
    if (isInsideSupportCondition(text, index)) continue;
    if (hasNearbyScrollTimelineGuard(text, index)) continue;
    const line = lineForIndex(text, index);
    if (seenLines.has(line)) continue;
    if (lines[line - 1]?.includes('motion-audit-ignore')) continue;
    seenLines.add(line);
    findings.push({
      id: 'scroll-timeline-support',
      file: path.relative(scanRoot, file),
      line,
      excerpt: excerptForLine(lines, line),
      hint: 'Guard scroll-driven timeline CSS with @supports or CSS.supports.',
    });
  }
}
function isScannableMotionFile(file) {
  return /\.(js|jsx|ts|tsx|css|scss|md|mjs|cjs)$/.test(file);
}
function globalRegex(regex) {
  return regex.global ? regex : new RegExp(regex.source, `${regex.flags}g`);
}
function listFiles(dir, acc = [], limit = Infinity) {
  if (!existsSync(dir) || acc.length >= limit) return acc;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (acc.length >= limit) break;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (['.git','node_modules','.next','dist','build','coverage','.turbo'].includes(entry.name)) continue;
      listFiles(full, acc, limit);
    } else if (entry.isFile()) acc.push(full);
  }
  return acc;
}
function listScannableFiles(dir, acc = [], limit = Infinity) {
  if (!existsSync(dir) || acc.length >= limit) return acc;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (acc.length >= limit) break;
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (['.git','node_modules','.next','dist','build','coverage','.turbo'].includes(entry.name)) continue;
      listScannableFiles(full, acc, limit);
    } else if (entry.isFile() && isScannableMotionFile(full)) acc.push(full);
  }
  return acc;
}
function skillDirs() {
  return readdirSync(skillsRoot, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name).sort();
}
function parseFrontmatter(text) {
  const match = text.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return {};
  const fm = {};
  let current = null;
  for (const line of match[1].split('\n')) {
    const m = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (m) {
      current = m[1];
      fm[current] = m[2].replace(/^['"]|['"]$/g, '');
    } else if (current && /^\s+/.test(line)) {
      fm[current] += ' ' + line.trim();
    }
  }
  return fm;
}
function loadJson(file) {
  try { return JSON.parse(read(file)); } catch (error) { return { __error: String(error) }; }
}
function inspectSkill(name) {
  const dir = path.join(skillsRoot, name);
  const skill = read(path.join(dir, 'SKILL.md'));
  const fm = parseFrontmatter(skill);
  const openai = existsSync(path.join(dir, 'agents', 'openai.yaml')) ? read(path.join(dir, 'agents', 'openai.yaml')) : '';
  const evals = loadJson(path.join(dir, 'evals', 'evals.json'));
  const triggers = loadJson(path.join(dir, 'evals', 'trigger-queries.json'));
  const prov = loadJson(path.join(dir, 'references', 'provenance.json'));
  const referenceFiles = listFiles(path.join(dir, 'references')).filter((file) => file.endsWith('.md'));
  const assetTemplateFiles = listFiles(path.join(dir, 'assets', 'templates')).filter((file) => file.endsWith('.md'));
  const assetExampleFiles = listFiles(path.join(dir, 'assets', 'examples'));
  return {
    name,
    lines: skill.split('\n').length,
    description_length: fm.description?.length ?? 0,
    implicit: /allow_implicit_invocation:\s*true/.test(openai),
    has_openai: !!openai,
    eval_count: Array.isArray(evals.evals) ? evals.evals.length : 0,
    trigger_count: Array.isArray(triggers.queries) ? triggers.queries.length : 0,
    source_count: Array.isArray(prov.upstream_sources) ? prov.upstream_sources.length : 0,
    reference_count: referenceFiles.length,
    asset_template_count: assetTemplateFiles.length,
    asset_example_count: assetExampleFiles.length,
  };
}
function doctor() {
  const skills = skillDirs().map(inspectSkill);
  return { ok: true, plugin: pluginName, plugin_root: pluginRoot, skill_count: skills.length, skills };
}
function verifySources() {
  const findings = [];
  for (const name of skillDirs()) {
    const dir = path.join(skillsRoot, name);
    const skillText = existsSync(path.join(dir, 'SKILL.md')) ? read(path.join(dir, 'SKILL.md')) : '';
    const ledger = path.join(dir, 'references', 'source-ledger.md');
    const prov = path.join(dir, 'references', 'provenance.json');
    const index = path.join(dir, 'references', 'index.md');
    if (!existsSync(index)) findings.push({ skill: name, severity: 'error', message: 'missing references/index.md' });
    for (const file of listFiles(path.join(dir, 'references'))) {
      const rel = path.relative(path.join(dir, 'references'), file);
      if (rel.split(path.sep).length > 1) findings.push({ skill: name, severity: 'error', message: 'nested reference file is not one-level: ' + rel });
      const routed = 'references/' + rel.split(path.sep).join('/');
      if (!skillText.includes('`' + routed + '`')) {
        findings.push({ skill: name, severity: 'error', message: 'SKILL.md resource index does not route ' + routed });
      }
    }
    if (!existsSync(ledger)) findings.push({ skill: name, severity: 'error', message: 'missing references/source-ledger.md' });
    else {
      const text = portableText(read(ledger));
      if (!/Checked at:/.test(text)) findings.push({ skill: name, severity: 'error', message: 'source ledger missing Checked at' });
      if (localPathPattern.test(text)) findings.push({ skill: name, severity: 'error', message: 'source ledger contains machine-local path' });
    }
    const data = existsSync(prov) ? loadJson(prov) : null;
    if (!data) findings.push({ skill: name, severity: 'error', message: 'missing references/provenance.json' });
    else if (data.__error) findings.push({ skill: name, severity: 'error', message: data.__error });
    else {
      if (!data.checked_at) findings.push({ skill: name, severity: 'error', message: 'provenance missing checked_at' });
      if (!Array.isArray(data.upstream_sources) || data.upstream_sources.length === 0) findings.push({ skill: name, severity: 'error', message: 'provenance missing upstream_sources' });
    }
  }
  return { ok: findings.every((f) => f.severity !== 'error'), plugin: pluginName, findings };
}
function verifyAssets() {
  const findings = [];
  for (const name of skillDirs()) {
    const dir = path.join(skillsRoot, name);
    const templates = listFiles(path.join(dir, 'assets', 'templates')).filter((file) => file.endsWith('.md'));
    const examples = listFiles(path.join(dir, 'assets', 'examples'));
    if (existsSync(path.join(dir, 'templates'))) {
      findings.push({ skill: name, severity: 'error', message: 'old top-level templates/ directory must move to assets/templates/' });
    }
    if (templates.length < 2) {
      findings.push({ skill: name, severity: 'error', message: 'assets/templates needs audit and review templates' });
    }
    if (examples.length < 1) {
      findings.push({ skill: name, severity: 'error', message: 'assets/examples needs at least one fixture or starter example' });
    }
  }
  return { ok: findings.every((f) => f.severity !== 'error'), plugin: pluginName, findings };
}
function verifyEvals() {
  const findings = [];
  for (const name of skillDirs()) {
    const dir = path.join(skillsRoot, name);
    const evals = loadJson(path.join(dir, 'evals', 'evals.json'));
    const triggers = loadJson(path.join(dir, 'evals', 'trigger-queries.json'));
    if (evals.__error) findings.push({ skill: name, severity: 'error', message: evals.__error });
    if (!Array.isArray(evals.evals) || evals.evals.length < 3) findings.push({ skill: name, severity: 'error', message: 'evals/evals.json must include at least 3 evals' });
    for (const ev of evals.evals || []) {
      if (!ev.prompt || !ev.expected_output) findings.push({ skill: name, severity: 'error', message: 'eval ' + (ev.id || '<missing>') + ' missing prompt or expected_output' });
      if (!Array.isArray(ev.assertions) || ev.assertions.length < 4) findings.push({ skill: name, severity: 'error', message: 'eval ' + (ev.id || '<missing>') + ' has fewer than 4 assertions' });
    }
    if (triggers.__error) findings.push({ skill: name, severity: 'error', message: triggers.__error });
    const queries = triggers.queries || [];
    if (!Array.isArray(queries) || queries.length < 20) findings.push({ skill: name, severity: 'error', message: 'trigger evals need at least 20 queries' });
    const pos = queries.filter((q) => q.should_trigger === true).length;
    const neg = queries.filter((q) => q.should_trigger === false).length;
    if (pos < 8 || neg < 8) findings.push({ skill: name, severity: 'error', message: 'trigger evals need balanced positives and negatives' });
  }
  return { ok: findings.every((f) => f.severity !== 'error'), plugin: pluginName, findings };
}
function scan() {
  const scanRoot = path.resolve(option('--root', process.cwd()));
  const maxFiles = Number(option('--max-files', '2000'));
  const patterns = [
    { id: 'transition-all', scope: 'file', re: /\btransitionProperty\s*[:=]\s*\{?\s*(?:['"`][^'"`{};]*\ball\b|[^,;{}\n]*\ball\b)|\btransition\s*[:=]\s*\{?\s*['"`][^'"`{};]*\ball\b|\btransition(?:-property)?\s*:(?!\s*\{?\s*['"`])\s*[^;{}]*\ball\b|\btransition-all\b/g, hint: 'Prefer explicit transitioned properties.' },
    { id: 'infinite-animation', re: /repeat:\s*-1|animation-iteration-count:\s*infinite|loop=\{?true/, hint: 'Verify reduced-motion and interruption behavior for loops.' },
    { id: 'layout-animation', scope: 'file', re: /\btransitionProperty\s*[:=]\s*\{?\s*(?:['"`][^'"`{};]*\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\b|[^,;{}\n]*\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\b)|\btransition\s*[:=]\s*\{?\s*['"`][^'"`{};]*\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\b|\btransition(?:-property)?\s*:(?!\s*\{?\s*['"`])\s*[^;{}]*\b(width|height|top|left|right|bottom|margin(?:-[a-z-]+)?|padding(?:-[a-z-]+)?)\b/g, hint: 'Layout or paint animation needs performance proof.' },
    { id: 'missing-reduced-motion', re: /gsap\.|animate\(|withTiming|withSpring|lottie|Rive|Canvas/, hint: 'Check for reduced-motion or static fallback.' },
  ];
  const files = listScannableFiles(scanRoot, [], maxFiles);
  const findings = [];
  for (const file of files) {
    if (!isScannableMotionFile(file)) continue;
    const text = read(file);
    if (text.slice(0, 512).includes('motion-audit-skip-file')) continue;
    const lines = text.split('\n');
    const searchableText = stripCommentsPreserveLength(text);
    const searchableLines = searchableText.split('\n');
    const lineOffsets = [];
    let lineOffset = 0;
    for (const searchableLine of searchableLines) {
      lineOffsets.push(lineOffset);
      lineOffset += searchableLine.length + 1;
    }
    addCustomPropertyFindings(text, lines, scanRoot, file, findings);
    addScrollTimelineFindings(text, lines, scanRoot, file, findings);
    for (const pattern of patterns.filter((item) => item.scope === 'file')) {
      pattern.re.lastIndex = 0;
      for (const match of searchableText.matchAll(pattern.re)) {
        if (isIncidentalStringMatch(searchableText, match.index ?? 0)) continue;
        const line = lineForIndex(text, match.index ?? 0);
        if (!lines[line - 1]?.includes('motion-audit-ignore')) {
          findings.push({ id: pattern.id, file: path.relative(scanRoot, file), line, excerpt: excerptForLine(lines, line), hint: pattern.hint });
        }
      }
    }
    lines.forEach((line, index) => {
      for (const pattern of patterns.filter((item) => item.scope !== 'file')) {
        const searchableLine = searchableLines[index] ?? '';
        const lineRegex = globalRegex(pattern.re);
        for (const match of searchableLine.matchAll(lineRegex)) {
          const absoluteIndex = (lineOffsets[index] ?? 0) + (match.index ?? 0);
          if (!isIncidentalStringMatch(searchableText, absoluteIndex) && !line.includes('motion-audit-ignore')) {
            findings.push({ id: pattern.id, file: path.relative(scanRoot, file), line: index + 1, excerpt: line.trim().slice(0, 180), hint: pattern.hint });
            break;
          }
        }
      }
    });
  }
  return { ok: true, plugin: pluginName, root: scanRoot, files_scanned: files.length, findings };
}
function validateAtomic() {
  const results = [doctor(), verifySources(), verifyAssets(), verifyEvals()];
  return { ok: results.every((r) => r.ok), plugin: pluginName, results };
}
function toMarkdown(result) {
  if (result.error) return '# ' + result.plugin + '\n\nERROR: ' + result.error.message + '\n';
  const lines = ['# ' + (result.plugin || pluginName), '', 'ok: ' + result.ok];
  if (result.skill_count != null) lines.push('skill_count: ' + result.skill_count);
  if (Array.isArray(result.skills)) for (const s of result.skills) lines.push('- ' + s.name + ': lines=' + s.lines + ', openai=' + s.has_openai + ', evals=' + s.eval_count + ', triggers=' + s.trigger_count + ', sources=' + s.source_count + ', references=' + s.reference_count + ', templates=' + s.asset_template_count + ', examples=' + s.asset_example_count);
  if (Array.isArray(result.findings)) for (const f of result.findings) lines.push('- [' + (f.severity || 'info') + '] ' + (f.skill || f.file || f.id || 'finding') + ': ' + (f.message || f.hint || ''));
  if (Array.isArray(result.results)) for (const r of result.results) lines.push('', toMarkdown(r));
  return lines.join('\n') + '\n';
}

if (has('--help') || args.length === 0) {
  process.stdout.write(usage());
  process.exit(0);
}
const command = args[0];
if (command === 'doctor') out(doctor());
else if (command === 'sources' && args[1] === 'verify') { const r = verifySources(); out(r); if (!r.ok) process.exit(1); }
else if (command === 'eval' && has('--all')) { const r = verifyEvals(); out(r); if (!r.ok) process.exit(1); }
else if (command === 'scan') out(scan());
else if (command === 'validate-atomic') { const r = validateAtomic(); out(r); if (!r.ok) process.exit(1); }
else fail('unknown command: ' + args.join(' '));
