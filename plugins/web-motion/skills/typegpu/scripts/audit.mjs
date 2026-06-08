#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const profile = {
  "skillName": "typegpu",
  "rules": [
    {
      "id": "typegpu.gpu-guard",
      "severity": "medium",
      "confidence": "medium",
      "category": "compatibility",
      "kind": "fileContainsWithout",
      "include": "\\btgpu\\.init\\(|\\.configureContext\\(",
      "without": "navigator\\.gpu|requestAdapter|gpu\\s+not\\s+supported|unsupported|fallback|WebGPU",
      "message": "TypeGPU/WebGPU code lacks an obvious GPU capability guard.",
      "recommendation": "Check navigator.gpu/requestAdapter and provide a fallback or clear error state."
    },
    {
      "id": "typegpu.missing-webgpu-types",
      "severity": "medium",
      "confidence": "medium",
      "category": "setup",
      "kind": "packageRequiresWhenAny",
      "whenPackages": ["typegpu"],
      "requiredPackages": ["@webgpu/types"],
      "message": "The project depends on TypeGPU but package.json does not include @webgpu/types.",
      "recommendation": "Install @webgpu/types or confirm another checked-in typing strategy for WebGPU globals."
    },
    {
      "id": "typegpu.use-gpu-without-unplugin",
      "severity": "high",
      "confidence": "medium",
      "category": "setup",
      "kind": "packageRequiresWhenFilesContain",
      "whenPackages": ["typegpu"],
      "requiredPackages": ["unplugin-typegpu"],
      "filePattern": "['\\\"]use gpu['\\\"]",
      "message": "TypeScript/JavaScript 'use gpu' shader functions were found without unplugin-typegpu in package.json.",
      "recommendation": "Install and configure unplugin-typegpu for the target bundler before relying on TypeGPU TS shader functions."
    },
    {
      "id": "typegpu.per-frame-resource",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "kind": "fileContainsBoth",
      "include": "create(?:Buffer|Texture|BindGroup|Sampler|View)\\(",
      "also": "requestAnimationFrame|useFrame\\(|setInterval",
      "message": "GPU resources may be allocated inside a per-frame path.",
      "recommendation": "Create buffers, textures, views, bind groups, and samplers during setup and reuse them."
    },
    {
      "id": "typegpu.shader-console-log",
      "severity": "medium",
      "confidence": "medium",
      "category": "debugging",
      "kind": "fileContainsBoth",
      "include": "['\\\"]use gpu['\\\"]",
      "also": "console\\.log\\(",
      "message": "A TypeGPU shader appears to contain console.log debugging.",
      "recommendation": "Remove shader console.log from production paths; it injects atomics and is intended for debugging."
    },
    {
      "id": "typegpu.unsized-array-create-buffer",
      "severity": "high",
      "confidence": "medium",
      "category": "typing",
      "pattern": "createBuffer\\(\\s*d\\.arrayOf\\(\\s*[^,\\)]*\\s*\\)",
      "message": "createBuffer appears to receive an unsized d.arrayOf schema.",
      "recommendation": "Call d.arrayOf(Element, count) for CPU buffer creation; use the unsized factory only in bind group layouts."
    },
    {
      "id": "typegpu.visible-motion-without-reduced-motion",
      "severity": "medium",
      "confidence": "low",
      "category": "accessibility",
      "kind": "fileContainsBothWithout",
      "include": "createRenderPipeline\\(|requestAnimationFrame|useFrame\\(",
      "also": "typegpu|tgpu\\.|['\\\"]use gpu['\\\"]",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|reduceMotion|static fallback",
      "message": "Visible or continuous TypeGPU rendering was found without an obvious reduced-motion/static-quality branch.",
      "recommendation": "For nonessential motion, add reduced-motion behavior, a static frame, or document an existing app-level policy."
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

function readPackageFile(file) {
  if (!fs.existsSync(file)) return { exists: false, packages: new Set(), scripts: {} };
  try {
    const pkg = JSON.parse(fs.readFileSync(file, 'utf8'));
    const deps = Object.assign({}, pkg.dependencies, pkg.devDependencies, pkg.peerDependencies, pkg.optionalDependencies);
    return { exists: true, file, dir: path.dirname(file), packages: new Set(Object.keys(deps ?? {})), scripts: pkg.scripts ?? {} };
  } catch {
    return { exists: true, file, dir: path.dirname(file), packages: new Set(), scripts: {} };
  }
}

function readPackage(root) {
  return readPackageFile(path.join(root, 'package.json'));
}

function packageRecords(root) {
  return listPackageFiles(root).map(readPackageFile);
}

function nearestPackageRecord(file, records) {
  return records
    .filter((record) => {
      const relative = path.relative(record.dir, file);
      return relative === '' || (!relative.startsWith('..') && !path.isAbsolute(relative));
    })
    .sort((a, b) => b.dir.length - a.dir.length)[0] ?? null;
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

function hasAnyPackage(pkg, names) {
  return (names ?? []).some((name) => pkg.packages.has(name));
}

function missingPackages(pkg, names) {
  return (names ?? []).filter((name) => !pkg.packages.has(name));
}

function filesContain(files, pattern) {
  const regex = ruleRegex(pattern);
  for (const file of files) {
    try {
      if (regex.test(fs.readFileSync(file, 'utf8'))) return true;
      regex.lastIndex = 0;
    } catch {
      // Ignore unreadable files during a best-effort static audit.
    }
  }
  return false;
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
  const packages = packageRecords(root);
  const findings = [];
  for (const rule of profile.rules) {
    if (config.ignoreRules.includes(rule.id)) continue;
    if (rule.kind === 'packageRequiresWhenAny') {
      for (const pkg of packages) {
        if (!pkg.exists || !hasAnyPackage(pkg, rule.whenPackages)) continue;
        const missing = missingPackages(pkg, rule.requiredPackages);
        if (missing.length === 0) continue;
        const packagePath = path.relative(root, pkg.file);
        if (isIgnored(config, rule.id, packagePath, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packagePath}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packagePath,
          line: 1,
          excerpt: `missing packages: ${missing.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageRequiresWhenFilesContain') {
      for (const pkg of packages) {
        if (!pkg.exists || !hasAnyPackage(pkg, rule.whenPackages)) continue;
        const ownedFiles = files.filter((file) => nearestPackageRecord(file, packages) === pkg);
        if (!filesContain(ownedFiles, rule.filePattern)) continue;
        const missing = missingPackages(pkg, rule.requiredPackages);
        if (missing.length === 0) continue;
        const packagePath = path.relative(root, pkg.file);
        if (isIgnored(config, rule.id, packagePath, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packagePath}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packagePath,
          line: 1,
          excerpt: `missing packages: ${missing.join(', ')}`,
          rationale: rule.message,
          recommendation: rule.recommendation,
        });
      }
      continue;
    }
    if (rule.kind === 'packageHasAny') {
      for (const pkg of packages) {
        const matched = (rule.packages ?? []).filter((name) => pkg.packages.has(name));
        if (matched.length === 0) continue;
        const packagePath = path.relative(root, pkg.file);
        if (isIgnored(config, rule.id, packagePath, [''], 1)) continue;
        findings.push({
          id: `${rule.id}:${packagePath}:1`,
          ruleId: rule.id,
          severity: rule.severity,
          confidence: rule.confidence,
          category: rule.category,
          file: packagePath,
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
      'package-aware rules inspect package.json dependencies and devDependencies',
      'use --format json for machine-readable output',
      'findings can be suppressed with .motion-audit.json or inline motion-audit-ignore comments',
    ],
  };
}

function renderMarkdown(result) {
  if (result.sampleFileCount !== undefined) {
    return `# TypeGPU Audit Doctor: ${result.profile}

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
  return `# TypeGPU Audit Report: ${result.profile}

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
