#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const currentScript = fs.realpathSync(fileURLToPath(import.meta.url));

const profile = {
  "skillName": "web-rive",
  "rules": [
    {
      "id": "rive.deprecated-webgl-runtime",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageHasAny",
      "packages": ["@rive-app/webgl", "@rive-app/react-webgl"],
      "message": "Deprecated Rive WebGL runtime package is installed.",
      "recommendation": "Use @rive-app/webgl2 or @rive-app/react-webgl2 unless an existing migration plan explicitly owns the legacy runtime."
    },
    {
      "id": "rive.legacy-rive-react-package",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "kind": "packageHasAny",
      "packages": ["rive-react"],
      "message": "Legacy rive-react package is installed.",
      "recommendation": "Migrate to @rive-app/react-webgl2, @rive-app/react-canvas, or @rive-app/react-canvas-lite based on the asset renderer requirements."
    },
    {
      "id": "rive.mixed-react-renderers",
      "severity": "medium",
      "confidence": "medium",
      "category": "dependency",
      "kind": "packageHasMultiple",
      "packages": ["@rive-app/react-webgl2", "@rive-app/react-canvas", "@rive-app/react-canvas-lite"],
      "message": "Multiple Rive React renderer packages are installed.",
      "recommendation": "Keep one renderer per surface unless there is an explicit package/runtime boundary."
    },
    {
      "id": "rive.mixed-web-renderers",
      "severity": "medium",
      "confidence": "medium",
      "category": "dependency",
      "kind": "packageHasMultiple",
      "packages": ["@rive-app/webgl2", "@rive-app/canvas", "@rive-app/canvas-lite"],
      "message": "Multiple plain Rive Web renderer packages are installed.",
      "recommendation": "Keep WebGL2, Canvas, and Canvas Lite behind explicit surface boundaries with measured package/renderer tradeoffs."
    },
    {
      "id": "rive.deprecated-runtime-import",
      "severity": "high",
      "confidence": "high",
      "category": "dependency",
      "pattern": "from\\s+['\"](?:@rive-app/(?:react-)?webgl|rive-react)['\"]|['\"](?:@rive-app/(?:react-)?webgl|rive-react)['\"]",
      "message": "Deprecated Rive runtime import is used in source.",
      "recommendation": "Replace deprecated imports with @rive-app/webgl2, @rive-app/react-webgl2, @rive-app/canvas, or @rive-app/react-canvas as appropriate."
    },
    {
      "id": "rive.raw-input-name",
      "severity": "medium",
      "confidence": "low",
      "category": "contract",
      "pattern": "\\b(useStateMachineInput|stateMachineInputs|viewModelByName|useViewModel|useViewModelInstance(?:Number|String|Boolean|Color|Enum|Trigger|Image|List|Artboard)?)\\s*\\([^\\n;]*['\"][A-Za-z0-9 _./-]+['\"]",
      "message": "Rive asset contract names appear inline.",
      "recommendation": "Prefer a typed local mapping for artboard, state machine, view model, property, input, and event names."
    },
    {
      "id": "rive.inline-contract-props",
      "severity": "low",
      "confidence": "low",
      "category": "contract",
      "pattern": "\\b(artboard|stateMachines|animations)\\s*[:=]\\s*['\"][A-Za-z0-9 _./-]+['\"]",
      "message": "Rive artboard, state machine, or animation name appears inline.",
      "recommendation": "Move asset contract strings into a single typed map or wrapper so designer-export changes fail in one place."
    },
    {
      "id": "rive.legacy-state-machine-inputs",
      "severity": "low",
      "confidence": "medium",
      "category": "contract",
      "pattern": "\\b(useStateMachineInput|stateMachineInputs)\\s*\\(",
      "message": "Legacy state-machine Inputs API is used.",
      "recommendation": "Confirm the .riv asset intentionally exposes legacy Inputs. Prefer Data Binding View Models for new durable product contracts."
    },
    {
      "id": "rive.data-binding-without-instance-policy",
      "severity": "low",
      "confidence": "low",
      "category": "contract",
      "kind": "fileContainsBothWithout",
      "include": "\\b(viewModelByName|defaultViewModel|useViewModel|useViewModelInstance(?:Number|String|Boolean|Color|Enum|Trigger|Image|List|Artboard)?)\\b",
      "also": "\\b(@rive-app/|useRive\\(|new\\s+Rive\\s*\\(|\\.riv\\b)",
      "without": "\\b(autoBind|viewModelInstance|instanceByName|instanceByIndex|defaultInstance|instance\\(|bindViewModelInstance|default instance|Default instance)\\b",
      "message": "Rive Data Binding usage appears without an obvious ViewModelInstance selection policy.",
      "recommendation": "Document whether the wrapper uses auto-binding, a named instance, the default instance, or a newly created instance."
    },
    {
      "id": "rive.remote-riv-asset",
      "severity": "medium",
      "confidence": "medium",
      "category": "asset",
      "pattern": "\\bsrc\\s*[:=]\\s*['\"]https?://[^'\"]+\\.riv(?:[?#][^'\"]*)?['\"]",
      "message": "A remote .riv asset URL is referenced directly.",
      "recommendation": "Prefer local/versioned assets or require an allowlisted origin, fallback UI, cache strategy, and failure test."
    },
    {
      "id": "rive.demo-cdn-asset",
      "severity": "low",
      "confidence": "medium",
      "category": "asset",
      "pattern": "https?://cdn\\.rive\\.app/animations/[^'\"]+\\.riv",
      "message": "A Rive demo CDN asset appears in application code.",
      "recommendation": "Use demo CDN assets only in examples. Production surfaces should own the .riv file or route it through the app's controlled asset pipeline."
    },
    {
      "id": "rive.unpinned-unpkg-runtime",
      "severity": "medium",
      "confidence": "medium",
      "category": "dependency",
      "pattern": "https?://unpkg\\.com/@rive-app/(?:webgl2|canvas|canvas-lite)(?:(?=$|[/\"'`?])|@(?!\\d+\\.\\d+\\.\\d+(?:[-+][0-9A-Za-z.-]+)?(?:$|[/\"'`?]))[^/\"'`?]*)",
      "message": "Unpinned Rive script-tag runtime is loaded from unpkg.",
      "recommendation": "Pin the runtime version or use the package manager so runtime upgrades are reviewed with source and asset validation."
    },
    {
      "id": "rive.missing-stable-dimensions",
      "severity": "low",
      "confidence": "low",
      "category": "layout",
      "kind": "fileContainsBothWithout",
      "include": "\\b(@rive-app/|useRive\\(|<Rive\\b|new\\s+Rive\\s*\\()",
      "also": "\\b(src|stateMachines|artboard)\\b",
      "without": "\\b(width|height|aspect-ratio|aspectRatio|className|style|resizeToCanvas|resizeDrawingSurfaceToCanvas)\\b",
      "message": "Rive usage appears without an obvious stable size or resize path in the same file.",
      "recommendation": "Reserve dimensions/aspect ratio before mount and verify resize/high-DPI behavior."
    },
    {
      "id": "rive.missing-state-contract",
      "severity": "low",
      "confidence": "low",
      "category": "contract",
      "kind": "fileContainsBothWithout",
      "include": "\\b(@rive-app/|useRive\\(|<Rive\\b|new\\s+Rive\\s*\\()",
      "also": "\\b(src|buffer)\\b",
      "without": "\\b(stateMachines|animations|RIVE_CONTRACT|riveContract)\\b",
      "message": "Rive usage appears without an explicit state machine, animation, or named contract map in the same file.",
      "recommendation": "Pass the intended state machine/animation or document the exported asset default in a local contract map."
    },
    {
      "id": "rive.webgl2-offscreen-disabled",
      "severity": "medium",
      "confidence": "medium",
      "category": "performance",
      "pattern": "\\buseOffscreenRenderer\\s*:\\s*false\\b",
      "message": "Rive WebGL2 offscreen renderer sharing is disabled.",
      "recommendation": "Keep the default unless measured evidence shows a need; verify many-instance and navigation cleanup behavior when disabling it."
    },
    {
      "id": "rive.imperative-instance-without-cleanup",
      "severity": "medium",
      "confidence": "low",
      "category": "lifecycle",
      "kind": "fileContainsWithout",
      "include": "\\bnew\\s+Rive\\s*\\(",
      "without": "\\bcleanup\\s*\\(|deleteRiveRenderer\\s*\\(|stopRendering\\s*\\(",
      "message": "Imperative Rive instance appears without cleanup in the same file.",
      "recommendation": "Call cleanup() on unmount/dispose and only call deleteRiveRenderer() after cleanup when fully tearing down WebGL renderer resources."
    },
    {
      "id": "motion.missing-reduced-motion",
      "severity": "medium",
      "confidence": "medium",
      "category": "accessibility",
      "kind": "fileContainsWithout",
      "include": "\\b(@rive-app/|useRive\\(|<Rive\\b|new\\s+Rive\\s*\\(|\\.riv\\b)",
      "without": "prefers-reduced-motion|motion-reduce|motion-safe|useReducedMotion|ReducedMotion|AccessibilityInfo|reduceMotion",
      "message": "Rive animation code was found without an obvious reduced-motion branch in the same file.",
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
const skillArtifactDirs = new Set(['agents', 'assets', 'examples', 'references', 'scripts', 'templates']);

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

function shouldSkipDir(relativePath, rootIsSkillDir = false) {
  const normalized = relativePath.split(path.sep).join('/');
  if (rootIsSkillDir && skillArtifactDirs.has(normalized.split('/')[0])) return true;
  if (/^skills\/[^/]+\/(?:agents|assets|examples|references|scripts|templates)(?:\/|$)/.test(normalized)) return true;
  return relativePath.split(path.sep).some((part) => skipDirs.has(part));
}

function readDirEntries(dir) {
  try {
    return fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return [];
  }
}

function shouldSkipFile(relativePath, fileName, fullPath = '') {
  const normalized = relativePath.split(path.sep).join('/');
  if (fullPath) {
    try {
      if (fs.readFileSync(fullPath, 'utf8').slice(0, 512).includes('motion-audit-skip-file')) return true;
    } catch {}
  }
  if (normalized === 'scripts/audit.mjs' || normalized.endsWith('/scripts/audit.mjs')) return true;
  if (/^skills\/[^/]+\/SKILL\.md$/.test(normalized)) return true;
  return path.extname(fileName) === '.json' && fileName !== 'package.json';
}

function listFiles(root, maxFiles) {
  const files = [];
  const rootIsSkillDir = fs.existsSync(path.join(root, 'SKILL.md')) && fs.existsSync(path.join(root, 'references'));
  function walk(dir) {
    if (files.length >= maxFiles) return;
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel, rootIsSkillDir)) walk(full);
      } else if (
        entry.isFile() &&
        !shouldSkipFile(rel, entry.name, full) &&
        fileExtensions.has(path.extname(entry.name)) &&
        fs.realpathSync(full) !== currentScript
      ) {
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
  const rootIsSkillDir = fs.existsSync(path.join(root, 'SKILL.md')) && fs.existsSync(path.join(root, 'references'));
  function walk(dir) {
    for (const entry of readDirEntries(dir)) {
      const full = path.join(dir, entry.name);
      const rel = path.relative(root, full);
      if (entry.isDirectory()) {
        if (!shouldSkipDir(rel, rootIsSkillDir)) walk(full);
      } else if (entry.isFile() && entry.name === 'package.json') {
        files.push(full);
      }
    }
  }
  walk(root);
  return files;
}

function readPackageFile(file) {
  if (!fs.existsSync(file)) return { exists: false, packages: new Set() };
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
    if (rule.kind === 'packageHasMultiple') {
      for (const pkg of packages) {
        const matched = (rule.packages ?? []).filter((name) => pkg.packages.has(name));
        if (matched.length <= 1) continue;
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
      'compare installed @rive-app/* versions with references/source-ledger.md before changing imports',
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
