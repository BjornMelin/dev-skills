#!/usr/bin/env node
// R3F cinematic art-direction / look-dev static auditor.
// Complements (does NOT duplicate) web-three-r3f's lifecycle/perf/cleanup audit:
// this file audits the VISUAL-QUALITY layer only — tone mapping, color
// management, lighting, postprocessing, and materials. Setup, disposal, DPR,
// loaders, and Canvas/createRoot lifecycle stay in web-three-r3f.
// Regex/text heuristics, not an AST (this family is deliberately regex-based) —
// treat every finding as a LEAD to confirm against the repo's installed versions.
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "r3f-scene-polish",
  "rules": [
    {
      "id": "r3f.tone-mapping-unset",
      "severity": "low",
      "confidence": "low",
      "category": "tone-mapping",
      "kind": "fileContainsWithout",
      "include": "<Canvas\\b",
      "also": "<mesh\\b|meshStandardMaterial|meshPhysicalMaterial|<primitive\\b|useGLTF|<Gltf\\b|<Environment\\b|<Stage\\b|<Instances\\b|<points\\b",
      "without": "toneMapping|<ToneMapping|<Canvas[^>]*?\\bflat\\b|motion-audit-ignore",
      "message": "A <Canvas> renders 3D content with no explicit tone mapping choice (no gl={{ toneMapping }}, no flat prop, no <ToneMapping> effect).",
      "recommendation": "R3F's <Canvas> default is ACESFilmicToneMapping; choose deliberately — AgX/Neutral give more accurate highlight rolloff for a cinematic look, or set flat/NoToneMapping if you tone-map in postprocessing."
    },
    {
      "id": "r3f.double-tonemap",
      "severity": "medium",
      "confidence": "medium",
      "category": "tone-mapping",
      "kind": "fileContainsBoth",
      "include": "\\btoneMapping\\s*[:=](?!\\s*(?:THREE\\.)?NoToneMapping\\b)",
      "also": "<ToneMapping\\b",
      "message": "Renderer-level tone mapping (gl={{ toneMapping }}) is set to a non-None value AND a postprocessing <ToneMapping> effect is present — the image is tone-mapped twice.",
      "recommendation": "Tone-map in exactly one place: either set the renderer to NoToneMapping (<Canvas flat>) and keep <ToneMapping>, or drop <ToneMapping> and tone-map on the renderer."
    },
    {
      "id": "r3f.legacy-tonemapping-api",
      "severity": "medium",
      "confidence": "high",
      "category": "tone-mapping",
      "pattern": "<ToneMapping\\b[^>]*\\b(?:adaptive|middleGrey|maxLuminance)\\b",
      "message": "A postprocessing <ToneMapping> effect uses deprecated props (adaptive / middleGrey / maxLuminance).",
      "recommendation": "Select the algorithm with mode={ToneMappingMode.AGX | NEUTRAL | ACES_FILMIC | ...} instead of the removed adaptive-luminance props."
    },
    {
      "id": "r3f.deprecated-color-encoding",
      "severity": "medium",
      "confidence": "high",
      "category": "color-management",
      "pattern": "\\boutputEncoding\\b|\\bsRGBEncoding\\b|\\bLinearEncoding\\b|\\.encoding\\s*=(?!=)",
      "message": "Deprecated color-encoding API (outputEncoding / sRGBEncoding / LinearEncoding / .encoding =) — renamed in three r152.",
      "recommendation": "Use outputColorSpace, texture.colorSpace, and SRGBColorSpace / LinearSRGBColorSpace (THREE.ColorManagement is on by default since r152)."
    },
    {
      "id": "r3f.texture-colorspace-missing",
      "severity": "low",
      "confidence": "low",
      "category": "materials",
      "kind": "fileContainsWithout",
      "include": "\\bmap=|\\buseTexture\\b",
      "without": "SRGBColorSpace|colorSpace|motion-audit-ignore",
      "message": "A color/albedo texture (map= / useTexture) is used but no SRGBColorSpace/colorSpace assignment appears in the same file.",
      "recommendation": "Set colorSpace={THREE.SRGBColorSpace} on color/albedo textures; leave data maps (normal/roughness/metalness/ao) linear. Low-confidence heuristic — confirm the texture's role."
    },
    {
      "id": "r3f.no-environment-or-lights",
      "severity": "low",
      "confidence": "low",
      "category": "lighting",
      "kind": "fileContainsWithout",
      "include": "<mesh\\b|meshStandardMaterial|meshPhysicalMaterial",
      "without": "<Environment\\b|<Stage\\b|<[A-Za-z][A-Za-z0-9]*Light\\b|<Lightformer\\b|motion-audit-ignore",
      "message": "Meshes / PBR materials appear with no <Environment>, no <*Light>, and no <Stage> in the same file — likely an unlit or flat-looking scene.",
      "recommendation": "Light for form: add HDRI <Environment> (image-based lighting) and/or intentional key/fill/rim lights, or a one-liner <Stage>. Heuristic — lighting may live in a parent component."
    },
    {
      "id": "r3f.postprocessing-no-quality-ladder",
      "severity": "low",
      "confidence": "low",
      "category": "postprocessing",
      "kind": "fileContainsBothWithout",
      "include": "<EffectComposer\\b",
      "also": "<N8AO\\b|<DepthOfField\\b|<SSAO\\b",
      "without": "\\bdpr\\b|\\bquality\\b|PerformanceMonitor|useReducedMotion|prefers-reduced-motion|AdaptiveDpr|useDetectGPU|motion-audit-ignore",
      "message": "An <EffectComposer> runs expensive effects (N8AO / DepthOfField / SSAO) with no device/DPR/quality gating in the same file.",
      "recommendation": "Gate heavy effects behind a device/DPR quality tier (dpr clamp, PerformanceMonitor/AdaptiveDpr, useDetectGPU) and give reduced-motion / low-power a lighter branch."
    },
    {
      "id": "r3f.legacy-ssao",
      "severity": "low",
      "confidence": "medium",
      "category": "postprocessing",
      "pattern": "<SSAO\\b",
      "message": "Legacy <SSAO> ambient occlusion — the current best-in-class is <N8AO>.",
      "recommendation": "Prefer <N8AO> (it computes normals from depth, so it needs no enableNormalPass). If you keep <SSAO>, it requires EffectComposer's enableNormalPass."
    },
    {
      "id": "r3f.postprocessing-webgpu-mismatch",
      "severity": "high",
      "confidence": "medium",
      "category": "postprocessing",
      "kind": "fileContainsBoth",
      "include": "@react-three/postprocessing|<EffectComposer\\b",
      "also": "WebGPURenderer|three/webgpu",
      "message": "pmndrs @react-three/postprocessing (<EffectComposer>) is used together with a WebGPU renderer (WebGPURenderer / three/webgpu) — that postprocessing stack is WebGL-only.",
      "recommendation": "On the WebGPU backend use three's own TSL PostProcessing (three/webgpu + three/tsl passes), not <EffectComposer>. Or commit to the WebGL2 renderer for pmndrs postprocessing."
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
  scripts/audit.mjs scan --root . --format json --output art-direction-audit.json

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
  const names = [
    'three', '@react-three/fiber', '@react-three/drei',
    '@react-three/postprocessing', 'postprocessing', 'react', 'react-dom',
  ];
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

function scanRule(rule, file, root, text, config) {
  const relativePath = path.relative(root, file);
  const lines = text.split('\n');
  const findings = [];
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
        text.includes('"skillName": "r3f-scene-polish"')
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

function ruleCatalog() {
  return profile.rules.map((rule) => ({
    id: rule.id,
    category: rule.category,
    severity: rule.severity,
    summary: rule.message,
  }));
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
    rules: ruleCatalog(),
    sampleFileCount: fs.existsSync(root) ? listFiles(root, maxFiles).length : 0,
    configFile: fs.existsSync(path.join(root, '.motion-audit.json')),
    notes: [
      'scan is read-only',
      'art-direction / look-dev auditor — complements web-three-r3f (lifecycle/perf/cleanup), does not duplicate it',
      'findings are regex/text heuristics — treat them as leads and verify against installed three / R3F / drei / postprocessing versions',
      'use --format json for machine-readable output',
      'findings can be suppressed with .motion-audit.json or inline motion-audit-ignore comments',
    ],
  };
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    const rules = (result.rules ?? [])
      .map((rule) => `- \`${rule.id}\` [${rule.severity}] (${rule.category}) — ${rule.summary}`)
      .join('\n');
    return `# Motion Audit Doctor: ${result.profile}

- Root: ${result.root}
- Package JSON: ${result.packageJson ? 'yes' : 'no'}
- Package hints: ${Object.keys(result.packageHints ?? {}).length > 0 ? JSON.stringify(result.packageHints) : 'none'}
- Config file: ${result.configFile ? 'yes' : 'no'}
- Configured rules: ${result.configuredRules}
- Sample file count: ${result.sampleFileCount}
- Status: ${result.ok ? 'ok' : 'failed'}

## Rules (${result.rules?.length ?? 0})

${rules || '(none)'}
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
