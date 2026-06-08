#!/usr/bin/env node
// motion-audit-skip-file: this helper contains validation regex strings.
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';

const pluginRoot = path.resolve(import.meta.dirname, '..');
const skillsRoot = path.join(pluginRoot, 'skills');
const tick = String.fromCharCode(96);

const requiredSkills = [
  'gsap-core',
  'gsap-frameworks',
  'gsap-performance',
  'gsap-plugins',
  'gsap-react',
  'gsap-scrolltrigger',
  'gsap-timeline',
  'gsap-utils',
  'typegpu',
  'web-css-animations',
  'web-lottie',
  'web-motion-react',
  'web-rive',
  'web-tailwind-motion',
  'web-three-r3f',
  'web-waapi',
];

const bannedPortablePatterns = [
  new RegExp('\\.' + 'firecrawl'),
  new RegExp('\\/home\\/' + 'bjorn'),
  new RegExp('\\/tmp\\/' + 'motion'),
  new RegExp('~\\/' + 'repos\\/agents'),
  new RegExp('\\.\\.\\/\\.\\.\\/' + 'references'),
];
const unresolvedPlaceholderPattern = new RegExp(`\\b(${['TO' + 'DO', 'FIX' + 'ME'].join('|')})\\b`);
const requiredFiles = [
  'SKILL.md',
  'agents/openai.yaml',
  'references/source-ledger.md',
  'references/provenance.json',
  'references/index.md',
  'scripts/audit.mjs',
  'evals/evals.json',
  'evals/trigger-queries.json',
];

let failed = false;

function fail(message) {
  console.error(`web-motion validation failed: ${message}`);
  failed = true;
}

function read(file) {
  return readFileSync(file, 'utf8');
}

function listFiles(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) return listFiles(full);
    return entry.isFile() ? [full] : [];
  });
}

function parseFrontmatter(text) {
  const match = text.match(/^---\n([\s\S]*?)\n---/);
  if (!match) return null;
  const frontmatter = {};
  const lines = match[1].split('\n');
  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    const scalar = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (!scalar) continue;
    const key = scalar[1];
    let value = scalar[2].trim().replace(/^['"]|['"]$/g, '');
    if (value === '>-' || value === '|' || value === '>') {
      const block = [];
      for (let next = index + 1; next < lines.length; next += 1) {
        if (/^[A-Za-z0-9_-]+:\s*/.test(lines[next])) break;
        if (/^\s+/.test(lines[next])) block.push(lines[next].trim());
        index = next;
      }
      value = block.join(' ');
    }
    frontmatter[key] = value;
  }
  return frontmatter;
}

function json(file) {
  try {
    return JSON.parse(read(file));
  } catch (error) {
    fail(`${path.relative(pluginRoot, file)}: invalid JSON: ${error.message}`);
    return {};
  }
}

function relativeList(dir, filter) {
  return listFiles(dir).map((file) => path.relative(dir, file)).filter(filter).sort();
}

if (!existsSync(path.join(pluginRoot, '.codex-plugin', 'plugin.json'))) {
  fail('missing .codex-plugin/plugin.json');
}
if (!existsSync(path.join(pluginRoot, 'scripts', 'motion-skillkit.mjs'))) {
  fail('missing scripts/motion-skillkit.mjs');
}

for (const skill of requiredSkills) {
  const skillDir = path.join(skillsRoot, skill);
  if (!existsSync(skillDir)) {
    fail(`missing required skill ${skill}`);
    continue;
  }

  for (const required of requiredFiles) {
    const file = path.join(skillDir, required);
    if (!existsSync(file)) fail(`${skill}: missing ${required}`);
    else if (statSync(file).size === 0) fail(`${skill}: empty ${required}`);
  }

  if (existsSync(path.join(skillDir, 'templates'))) {
    fail(`${skill}: old top-level templates/ directory must move to assets/templates/`);
  }

  const nestedReferenceFiles = relativeList(path.join(skillDir, 'references'), (file) => file.split(path.sep).length > 1);
  if (nestedReferenceFiles.length > 0) {
    fail(`${skill}: references must be one level deep from SKILL.md (${nestedReferenceFiles.slice(0, 3).join(', ')})`);
  }

  const templateFiles = relativeList(path.join(skillDir, 'assets', 'templates'), (file) => file.endsWith('.md'));
  if (templateFiles.length < 2) {
    fail(`${skill}: assets/templates needs at least audit-report and review-checklist templates`);
  }
  const exampleFiles = relativeList(path.join(skillDir, 'assets', 'examples'), () => true);
  if (exampleFiles.length < 1) {
    fail(`${skill}: assets/examples needs at least one skill-specific fixture/example`);
  }

  if (!existsSync(path.join(skillDir, 'SKILL.md'))) continue;
  const skillText = read(path.join(skillDir, 'SKILL.md'));
  const fm = parseFrontmatter(skillText);
  if (!fm) fail(`${skill}: missing YAML frontmatter`);
  else {
    if (fm.name !== skill) fail(`${skill}: frontmatter name ${fm.name} does not match directory`);
    if (!fm.description || fm.description.length > 1024) fail(`${skill}: description missing or over 1024 characters`);
  }

  if (/HyperFrames/.test(skillText)) fail(`${skill}: SKILL.md still mentions HyperFrames runtime/source by name`);
  if (skillText.includes('<!-- motion-audit-resources:start -->')) {
    fail(`${skill}: SKILL.md still uses old motion-audit resource marker`);
  }
  if (!skillText.includes('<!-- skill-resources:start -->')) {
    fail(`${skill}: missing skill resource index marker`);
  }
  for (const listed of [
    'references/index.md',
    'references/source-ledger.md',
    'references/provenance.json',
    'evals/evals.json',
    'evals/trigger-queries.json',
    'scripts/audit.mjs',
  ]) {
    if (!skillText.includes(tick + listed + tick)) fail(`${skill}: SKILL.md resource index does not list ${listed}`);
  }
  for (const listed of ['assets/templates/', 'assets/examples/']) {
    if (!skillText.includes(listed)) fail(`${skill}: SKILL.md resource index does not list ${listed}`);
  }

  const referenceFiles = relativeList(path.join(skillDir, 'references'), (file) => file.endsWith('.md'));
  const tailoredReferences = referenceFiles.filter((file) => !['source-ledger.md', 'index.md'].includes(file));
  if (tailoredReferences.length < 3) {
    fail(`${skill}: needs at least 3 tailored/source reference markdown files`);
  }
  for (const reference of referenceFiles) {
    const routed = `references/${reference}`;
    if (!skillText.includes(tick + routed + tick)) {
      fail(`${skill}: SKILL.md resource index does not route ${routed}`);
    }
  }

  const openai = existsSync(path.join(skillDir, 'agents', 'openai.yaml'))
    ? read(path.join(skillDir, 'agents', 'openai.yaml'))
    : '';
  if (!openai.includes('default_prompt:') || !openai.includes(`$${skill}`)) {
    fail(`${skill}: agents/openai.yaml missing default_prompt with $skill reference`);
  }
  if (skill === 'gsap-frameworks' && !/allow_implicit_invocation:\s*false/.test(openai)) {
    fail('gsap-frameworks must be explicit-only');
  }

  const ledger = existsSync(path.join(skillDir, 'references', 'source-ledger.md'))
    ? read(path.join(skillDir, 'references', 'source-ledger.md'))
    : '';
  if (!/Checked at:/.test(ledger)) fail(`${skill}: source ledger missing Checked at`);
  if (!ledger.includes('references/provenance.json')) fail(`${skill}: source ledger missing provenance link`);
  for (const pattern of bannedPortablePatterns) {
    if (pattern.test(ledger)) fail(`${skill}: source ledger contains banned non-portable pattern ${pattern}`);
  }

  const prov = json(path.join(skillDir, 'references', 'provenance.json'));
  if (!prov.checked_at || !Array.isArray(prov.upstream_sources) || prov.upstream_sources.length === 0) {
    fail(`${skill}: provenance missing checked_at or upstream_sources`);
  }

  const evals = json(path.join(skillDir, 'evals', 'evals.json'));
  if (!Array.isArray(evals.evals) || evals.evals.length < 3) fail(`${skill}: evals/evals.json needs at least 3 evals`);
  for (const item of evals.evals || []) {
    if (!Array.isArray(item.assertions) || item.assertions.length < 4) {
      fail(`${skill}: eval ${item.id || '<missing>'} needs at least 4 assertions`);
    }
  }

  const triggers = json(path.join(skillDir, 'evals', 'trigger-queries.json'));
  const queries = triggers.queries || [];
  if (!Array.isArray(queries) || queries.length < 20) fail(`${skill}: trigger-queries needs at least 20 queries`);
  const uniqueQueries = new Set(queries.map((query) => String(query.query || '').trim().toLowerCase()));
  if (uniqueQueries.size !== queries.length || uniqueQueries.has('')) {
    fail(`${skill}: trigger-queries must be non-empty and unique`);
  }
  const pos = queries.filter((query) => query.should_trigger === true).length;
  const neg = queries.filter((query) => query.should_trigger === false).length;
  if (pos < 8 || neg < 8) fail(`${skill}: trigger-queries must include balanced positives and negatives`);
}

for (const file of listFiles(pluginRoot)) {
  const relativePath = path.relative(pluginRoot, file);
  const text = read(file);
  for (const pattern of bannedPortablePatterns) {
    if (pattern.test(text)) fail(`${relativePath}: contains banned non-portable pattern ${pattern}`);
  }
  if (unresolvedPlaceholderPattern.test(text)) fail(`${relativePath}: contains unresolved placeholder marker`);
  if (/\/templates\//.test(relativePath) && !/\/assets\/templates\//.test(relativePath)) {
    fail(`${relativePath}: templates must live under assets/templates`);
  }
  if (/\/(README|CHANGELOG|INSTALLATION_GUIDE|QUICK_REFERENCE)\.md$/i.test(relativePath)) {
    fail(`${relativePath}: extraneous skill documentation file`);
  }
}

const tool = spawnSync(
  process.execPath,
  [path.join(pluginRoot, 'scripts', 'motion-skillkit.mjs'), 'validate-atomic', '--format', 'json'],
  { encoding: 'utf8' },
);
if (tool.status !== 0) fail(`motion-skillkit validate-atomic failed: ${tool.stdout}${tool.stderr}`);

if (failed) process.exit(1);
console.log('web-motion atomic skill validation passed');
