import {
  copyFileSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { Database } from 'bun:sqlite';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export type Severity = 'error' | 'warn' | 'info';
export type OutputFormat = 'text' | 'json' | 'md';
export type Confidence = 'high' | 'medium' | 'low';
export type FixKind = 'safe' | 'unsafe' | 'manual';
export type CommandName =
  | 'audit'
  | 'list-rules'
  | 'explain'
  | 'plan-fixes'
  | 'apply-safe-fixes'
  | 'validate'
  | 'benchmark'
  | 'release-sync';

export type Finding = Readonly<{
  ruleId: string;
  category: string;
  severity: Severity;
  confidence: Confidence;
  file: string;
  line: number;
  column: number;
  message: string;
  why?: string | undefined;
  suggestedFix?: string | undefined;
  snippet?: string | undefined;
  suppressionKey: string;
}>;

export type PlannedFix = Readonly<{
  ruleId: string;
  ruleIds: readonly string[];
  kind: FixKind;
  file: string;
  description: string;
  before?: string | undefined;
  after?: string | undefined;
  apply: () => void;
}>;

export type AuditConfig = Readonly<{
  disabledRules: readonly string[];
  severityOverrides: Readonly<Record<string, Severity>>;
  excludeDirs: readonly string[];
  baselineKeys: readonly string[];
  adapters: readonly string[];
  includePaths: readonly string[];
  maxFiles: number;
  maxBytes: number;
  validationCommands: readonly string[];
  manageGitignore: boolean;
}>;

export type CliArgs = Readonly<{
  command: CommandName;
  root: string;
  format: OutputFormat;
  failOn?: Severity | undefined;
  explain?: string | undefined;
  configPath?: string | undefined;
  baselinePath?: string | undefined;
  includePaths: readonly string[];
  excludeDirs: readonly string[];
  adapters: readonly string[];
  maxFiles?: number | undefined;
  maxBytes?: number | undefined;
}>;

type MutableAuditConfig = {
  disabledRules: string[];
  severityOverrides: Record<string, Severity>;
  excludeDirs: string[];
  baselineKeys: string[];
  adapters: string[];
  includePaths: string[];
  maxFiles: number;
  maxBytes: number;
  validationCommands: string[];
  manageGitignore: boolean;
};

type PackageJson = Readonly<{
  packageManager?: string;
  scripts?: Record<string, string>;
  dependencies?: Record<string, string>;
  devDependencies?: Record<string, string>;
  workspaces?: unknown;
}>;

type TsConfig = Readonly<{
  compilerOptions?: Record<string, unknown>;
}>;

type VercelConfig = Readonly<Record<string, unknown>>;

export type SkillContext = Readonly<{
  skillRoot: string;
  rulesDir: string;
  referencesDir: string;
  scriptsDir: string;
}>;

type RepoSignals = Readonly<{
  bunFirst: boolean;
  vercelBunEnabled: boolean;
  hasWorkspaces: boolean;
}>;

type RepoSnapshot = ReturnType<typeof createRepoSnapshot>;

type AdapterContext = Readonly<{
  root: string;
  snapshot: RepoSnapshot;
  signals: RepoSignals;
}>;

type AdapterDefinition = Readonly<{
  id: string;
  detect: (context: AdapterContext) => boolean;
  run: (context: AdapterContext) => readonly Finding[];
}>;

const DEFAULT_EXCLUDE_DIRS = [
  'node_modules',
  '.git',
  '.bun-platform',
  '.next',
  '.turbo',
  'dist',
  'build',
  'coverage',
  'out',
  'opensrc',
] as const;

const JS_EXTENSIONS = new Set(['ts', 'tsx', 'js', 'jsx', 'mjs', 'cjs']);

function parseSeverity(input: string | undefined): Severity | undefined {
  if (input === 'error' || input === 'warn' || input === 'info') return input;
  return undefined;
}

export function createSkillContext(moduleUrl: string): SkillContext {
  const filename = fileURLToPath(moduleUrl);
  const scriptsDir = path.dirname(filename);
  const skillRoot = path.resolve(scriptsDir, '..');
  return {
    skillRoot,
    rulesDir: path.join(skillRoot, 'rules'),
    referencesDir: path.join(skillRoot, 'references'),
    scriptsDir,
  };
}

export function parseCliArgs(argv: readonly string[]): CliArgs {
  const args = [...argv];
  const out: {
    command: CommandName;
    root: string;
    format: OutputFormat;
    failOn?: Severity | undefined;
    explain?: string | undefined;
    configPath?: string | undefined;
    baselinePath?: string | undefined;
    includePaths: string[];
    excludeDirs: string[];
    adapters: string[];
    maxFiles?: number | undefined;
    maxBytes?: number | undefined;
  } = {
    command: 'audit',
    root: process.cwd(),
    format: 'text',
    includePaths: [],
    excludeDirs: [],
    adapters: [],
  };

  const commands = new Set<CommandName>([
    'audit',
    'list-rules',
    'explain',
    'plan-fixes',
    'apply-safe-fixes',
    'validate',
    'benchmark',
    'release-sync',
  ]);

  if (args.length > 0 && commands.has(args[0] as CommandName)) {
    out.command = args.shift() as CommandName;
  } else if (args.includes('--list-rules')) {
    out.command = 'list-rules';
  } else if (args.includes('--explain')) {
    out.command = 'explain';
  }

  const take = (flag: string): string | undefined => {
    const index = args.indexOf(flag);
    if (index === -1) return undefined;
    const value = args[index + 1];
    if (!value || value.startsWith('--')) return undefined;
    return value;
  };

  const root = take('--root');
  if (root) out.root = path.resolve(root);

  const format = take('--format');
  if (format === 'text' || format === 'json' || format === 'md') out.format = format;

  const failOn = parseSeverity(take('--fail-on'));
  if (failOn) out.failOn = failOn;

  const explain = take('--explain');
  if (explain) out.explain = explain.trim();
  else if (out.command === 'explain' && args[0] && !args[0].startsWith('--')) out.explain = args[0];

  const configPath = take('--config');
  if (configPath) out.configPath = path.resolve(configPath);

  const baselinePath = take('--baseline');
  if (baselinePath) out.baselinePath = path.resolve(baselinePath);

  const maxFiles = take('--max-files');
  if (maxFiles && !Number.isNaN(Number(maxFiles))) out.maxFiles = Number(maxFiles);

  const maxBytes = take('--max-bytes');
  if (maxBytes && !Number.isNaN(Number(maxBytes))) out.maxBytes = Number(maxBytes);

  for (let index = 0; index < args.length; index++) {
    const value = args[index];
    if (value === '--include' && args[index + 1]) out.includePaths.push(args[index + 1]!);
    if (value === '--exclude' && args[index + 1]) out.excludeDirs.push(args[index + 1]!);
    if (value === '--adapter' && args[index + 1]) out.adapters.push(args[index + 1]!);
  }

  return out;
}

export function listRuleIds(context: SkillContext): readonly string[] {
  const files = readdirSync(context.rulesDir).filter((name) => name.endsWith('.md') && name !== '_index.md');
  return files.map((name) => name.replace(/\.md$/, '')).sort((a, b) => a.localeCompare(b));
}

export function explainRule(context: SkillContext, ruleId: string): string {
  return readFileSync(path.join(context.rulesDir, `${ruleId}.md`), 'utf8');
}

function safeReadText(filePath: string): string | undefined {
  try {
    return readFileSync(filePath, 'utf8');
  } catch {
    return undefined;
  }
}

function safeReadJson<T>(filePath: string): T | undefined {
  try {
    return JSON.parse(readFileSync(filePath, 'utf8')) as T;
  } catch {
    return undefined;
  }
}

function fileExists(filePath: string): boolean {
  try {
    return statSync(filePath).isFile();
  } catch {
    return false;
  }
}

function readBaselineKeys(root: string, baselinePath?: string, configPath?: string): readonly string[] {
  if (baselinePath) {
    const baseline = safeReadJson<{ suppressionKeys?: string[] } | string[]>(baselinePath);
    if (Array.isArray(baseline)) return baseline;
    return baseline?.suppressionKeys ?? [];
  }
  const config = safeReadJson<{ baseline?: string | string[] }>(configPath ?? path.join(root, 'bun-platform.config.json'));
  if (!config?.baseline) return [];
  if (Array.isArray(config.baseline)) return config.baseline;
  const configBaselinePath = path.resolve(root, config.baseline);
  const baseline = safeReadJson<{ suppressionKeys?: string[] } | string[]>(configBaselinePath);
  if (Array.isArray(baseline)) return baseline;
  return baseline?.suppressionKeys ?? [];
}

export function loadAuditConfig(root: string, configPath?: string, cliArgs?: Partial<CliArgs>): AuditConfig {
  const defaults: MutableAuditConfig = {
    disabledRules: [],
    severityOverrides: {},
    excludeDirs: [...DEFAULT_EXCLUDE_DIRS],
    baselineKeys: [],
    adapters: ['auto'],
    includePaths: [],
    maxFiles: Number.POSITIVE_INFINITY,
    maxBytes: Number.POSITIVE_INFINITY,
    validationCommands: [],
    manageGitignore: true,
  };

  const resolved = configPath ?? path.join(root, 'bun-platform.config.json');
  const loaded = safeReadJson<{
    disabledRules?: string[];
    severityOverrides?: Record<string, Severity>;
    excludeDirs?: string[];
    baseline?: string | string[];
    adapters?: string[];
    includePaths?: string[];
    maxFiles?: number;
    maxBytes?: number;
    validationCommands?: string[];
    manageGitignore?: boolean;
  }>(resolved);

  if (loaded?.disabledRules) defaults.disabledRules = [...loaded.disabledRules];
  if (loaded?.severityOverrides) defaults.severityOverrides = { ...loaded.severityOverrides };
  if (loaded?.excludeDirs) defaults.excludeDirs = [...new Set([...defaults.excludeDirs, ...loaded.excludeDirs])];
  if (loaded?.adapters && loaded.adapters.length > 0) defaults.adapters = [...loaded.adapters];
  if (loaded?.includePaths) defaults.includePaths = [...loaded.includePaths.map((value) => path.resolve(root, value))];
  if (typeof loaded?.maxFiles === 'number') defaults.maxFiles = loaded.maxFiles;
  if (typeof loaded?.maxBytes === 'number') defaults.maxBytes = loaded.maxBytes;
  if (loaded?.validationCommands) defaults.validationCommands = [...loaded.validationCommands];
  if (typeof loaded?.manageGitignore === 'boolean') defaults.manageGitignore = loaded.manageGitignore;
  defaults.baselineKeys = [...readBaselineKeys(root, cliArgs?.baselinePath, resolved)];

  if (cliArgs?.excludeDirs && cliArgs.excludeDirs.length > 0) {
    defaults.excludeDirs = [...new Set([...defaults.excludeDirs, ...cliArgs.excludeDirs])];
  }
  if (cliArgs?.includePaths && cliArgs.includePaths.length > 0) {
    defaults.includePaths = [...new Set([...defaults.includePaths, ...cliArgs.includePaths.map((value) => path.resolve(root, value))])];
  }
  if (cliArgs?.adapters && cliArgs.adapters.length > 0) {
    defaults.adapters = [...cliArgs.adapters];
  }
  if (typeof cliArgs?.maxFiles === 'number') defaults.maxFiles = cliArgs.maxFiles;
  if (typeof cliArgs?.maxBytes === 'number') defaults.maxBytes = cliArgs.maxBytes;

  return defaults;
}

function getRuleCategory(ruleId: string): string {
  const prefix = ruleId.split('-')[0] ?? 'other';
  switch (prefix) {
    case 'pm':
      return 'package-manager';
    case 'runtime':
      return 'runtime';
    case 'vercel':
      return 'vercel';
    case 'scripts':
      return 'scripts';
    case 'tsconfig':
      return 'typescript';
    case 'test':
      return 'testing';
    case 'build':
      return 'build';
    case 'perf':
      return 'performance';
    case 'migrate':
      return 'migration';
    case 'troubleshooting':
      return 'troubleshooting';
    default:
      return 'other';
  }
}

function severityRank(severity: Severity): number {
  switch (severity) {
    case 'error':
      return 3;
    case 'warn':
      return 2;
    case 'info':
      return 1;
  }
}

function lineColFromIndex(content: string, index: number): Readonly<{ line: number; column: number }> {
  let line = 1;
  let lastLineStart = 0;
  for (let i = 0; i < index; i++) {
    if (content.charCodeAt(i) === 10) {
      line++;
      lastLineStart = i + 1;
    }
  }
  return { line, column: index - lastLineStart + 1 };
}

function findFirst(content: string, regex: RegExp): Readonly<{ line: number; column: number; snippet: string }> | undefined {
  const expression = regex.global ? regex : new RegExp(regex.source, `${regex.flags}g`);
  expression.lastIndex = 0;
  const match = expression.exec(content);
  if (!match) return undefined;
  const index = match.index;
  const { line, column } = lineColFromIndex(content, index);
  const lineStart = content.lastIndexOf('\n', index);
  const nextBreak = content.indexOf('\n', index);
  const snippet = content.slice(lineStart === -1 ? 0 : lineStart + 1, nextBreak === -1 ? content.length : nextBreak);
  return { line, column, snippet: snippet.trimEnd() };
}

function createRepoSnapshot(root: string, config: AuditConfig) {
  const textCache = new Map<string, string | undefined>();
  const jsonCache = new Map<string, unknown>();
  const existsCache = new Map<string, boolean>();
  const fileListCache = new Map<string, string[]>();
  const statCache = new Map<string, ReturnType<typeof statSync>>();

  const readText = (filePath: string): string | undefined => {
    if (!textCache.has(filePath)) textCache.set(filePath, safeReadText(filePath));
    return textCache.get(filePath);
  };

  const readJson = <T>(filePath: string): T | undefined => {
    if (!jsonCache.has(filePath)) jsonCache.set(filePath, safeReadJson<T>(filePath));
    return jsonCache.get(filePath) as T | undefined;
  };

  const exists = (filePath: string): boolean => {
    if (!existsCache.has(filePath)) existsCache.set(filePath, fileExists(filePath));
    return existsCache.get(filePath) ?? false;
  };

  const includeFile = (filePath: string): boolean => {
    if (config.includePaths.length === 0) return true;
    return config.includePaths.some((includePath) => filePath === includePath || filePath.startsWith(`${includePath}${path.sep}`));
  };

  const getStat = (filePath: string) => {
    if (!statCache.has(filePath)) statCache.set(filePath, statSync(filePath));
    return statCache.get(filePath)!;
  };

  const walkFiles = (cacheKey = 'default'): string[] => {
    if (fileListCache.has(cacheKey)) return fileListCache.get(cacheKey)!;
    const files: string[] = [];
    let totalBytes = 0;
    const visit = (current: string) => {
      for (const entry of readdirSync(current, { withFileTypes: true })) {
        const full = path.join(current, entry.name);
        if (entry.isDirectory()) {
          if (config.excludeDirs.includes(entry.name)) continue;
          visit(full);
          continue;
        }
        if (!entry.isFile()) continue;
        if (!includeFile(full)) continue;
        const stat = getStat(full);
        totalBytes += stat.size;
        if (files.length >= config.maxFiles || totalBytes > config.maxBytes) {
          throw new Error(
            `Repo scan limits exceeded (files=${files.length}, bytes=${totalBytes}). Increase --max-files/--max-bytes or narrow the scope with --include.`,
          );
        }
        files.push(full);
      }
    };
    visit(root);
    fileListCache.set(cacheKey, files);
    return files;
  };

  return {
    root,
    readText,
    readJson,
    exists,
    getStat,
    walkFiles,
  };
}

function detectVercelBunEnabled(snapshot: RepoSnapshot): boolean {
  const vercelJson = snapshot.readJson<VercelConfig>(path.join(snapshot.root, 'vercel.json'));
  if (vercelJson && typeof vercelJson.bunVersion === 'string') return true;
  const vercelTs = snapshot.readText(path.join(snapshot.root, 'vercel.ts'));
  return Boolean(vercelTs && /bunVersion\s*:\s*["']/.test(vercelTs));
}

function inferSignals(snapshot: RepoSnapshot): RepoSignals {
  const pkg = snapshot.readJson<PackageJson>(path.join(snapshot.root, 'package.json'));
  const vercelBunEnabled = detectVercelBunEnabled(snapshot);
  const scripts = Object.values(pkg?.scripts ?? {});
  const hasWorkspaces =
    Array.isArray(pkg?.workspaces) ||
    (typeof pkg?.workspaces === 'object' && pkg?.workspaces !== null);
  const bunFirst =
    snapshot.exists(path.join(snapshot.root, 'bun.lockb')) ||
    Boolean(pkg?.packageManager?.startsWith('bun@')) ||
    vercelBunEnabled ||
    scripts.some((value) => /\bbun(x)?\b/.test(value));
  return {
    bunFirst,
    vercelBunEnabled,
    hasWorkspaces,
  };
}

function createFinding(
  root: string,
  ruleId: string,
  severity: Severity,
  file: string,
  message: string,
  options: Partial<Pick<Finding, 'line' | 'column' | 'snippet' | 'why' | 'suggestedFix' | 'confidence'>> = {},
): Finding {
  const relativeFile = path.isAbsolute(file) ? path.relative(root, file) : file;
  return {
    ruleId,
    category: getRuleCategory(ruleId),
    severity,
    confidence: options.confidence ?? 'high',
    file: path.join(root, relativeFile),
    line: options.line ?? 1,
    column: options.column ?? 1,
    message,
    why: options.why,
    suggestedFix: options.suggestedFix,
    snippet: options.snippet,
    suppressionKey: `${ruleId}:${relativeFile}`,
  };
}

function normalizeFindings(findings: readonly Finding[], config: AuditConfig): readonly Finding[] {
  const seen = new Set<string>();
  const filtered: Finding[] = [];

  for (const finding of findings) {
    if (config.disabledRules.includes(finding.ruleId)) continue;
    if (config.baselineKeys.includes(finding.suppressionKey)) continue;
    const override = config.severityOverrides[finding.ruleId];
    const normalized: Finding = override ? { ...finding, severity: override } : finding;
    const key = [
      normalized.ruleId,
      normalized.file,
      normalized.line,
      normalized.column,
      normalized.message,
    ].join('::');
    if (seen.has(key)) continue;
    seen.add(key);
    filtered.push(normalized);
  }

  return filtered.sort((left, right) => {
    const severityDelta = severityRank(right.severity) - severityRank(left.severity);
    if (severityDelta !== 0) return severityDelta;
    const fileDelta = left.file.localeCompare(right.file);
    if (fileDelta !== 0) return fileDelta;
    if (left.line !== right.line) return left.line - right.line;
    if (left.column !== right.column) return left.column - right.column;
    return left.ruleId.localeCompare(right.ruleId);
  });
}

const ENGINE_VERSION = 'bun-platform-core-v2';

function getPlatformStateDir(root: string): string {
  return path.join(root, '.bun-platform');
}

function getRollbackDir(root: string): string {
  return path.join(getPlatformStateDir(root), 'rollbacks');
}

function ensurePlatformGitignore(root: string, config: AuditConfig): void {
  if (!config.manageGitignore) return;

  const gitignorePath = path.join(root, '.gitignore');
  const existing = safeReadText(gitignorePath);
  const entryPattern = /^(?:\.bun-platform\/|\/\.bun-platform\/)\s*$/m;
  if (existing && entryPattern.test(existing)) return;

  const block = `${existing && existing.trim().length > 0 ? '\n' : ''}# Bun platform state\n.bun-platform/\n`;
  writeFileSync(gitignorePath, `${existing ?? ''}${block}`, 'utf8');
}

function openCacheDatabase(root: string, config: AuditConfig): Database {
  const stateDir = getPlatformStateDir(root);
  ensurePlatformGitignore(root, config);
  ensureDir(stateDir);
  const db = new Database(path.join(stateDir, 'cache.sqlite'));
  db.exec(`
    CREATE TABLE IF NOT EXISTS scan_cache (
      root TEXT NOT NULL,
      fingerprint TEXT NOT NULL,
      engine_version TEXT NOT NULL,
      findings_json TEXT NOT NULL,
      adapter_ids_json TEXT NOT NULL,
      created_at TEXT NOT NULL,
      PRIMARY KEY (root, fingerprint, engine_version)
    );

    CREATE TABLE IF NOT EXISTS release_sync_runs (
      root TEXT NOT NULL,
      synced_at TEXT NOT NULL,
      report_json TEXT NOT NULL
    );
  `);
  return db;
}

function buildRepoFingerprint(snapshot: RepoSnapshot, config: AuditConfig): string {
  const hash = createHash('sha256');
  hash.update(ENGINE_VERSION);
  hash.update(JSON.stringify({
    disabledRules: [...config.disabledRules].sort(),
    severityOverrides: config.severityOverrides,
    excludeDirs: [...config.excludeDirs].sort(),
    includePaths: [...config.includePaths].sort(),
    adapters: [...config.adapters].sort(),
    maxFiles: config.maxFiles,
    maxBytes: config.maxBytes,
  }));
  for (const file of snapshot.walkFiles('fingerprint').sort((a, b) => a.localeCompare(b))) {
    const stat = snapshot.getStat(file);
    hash.update(path.relative(snapshot.root, file));
    hash.update(String(stat.mtimeMs));
    hash.update(String(stat.size));
  }
  return hash.digest('hex');
}

function resolveAdapterIds(signals: RepoSignals, config: AuditConfig): string[] {
  if (config.adapters.length === 0 || config.adapters.includes('auto')) {
    const auto = ['github-actions', 'docker'];
    if (signals.vercelBunEnabled) auto.push('vercel');
    if (signals.hasWorkspaces) auto.push('monorepo');
    return auto;
  }
  return [...new Set(config.adapters)];
}

function findLine(content: string, regex: RegExp): Partial<Pick<Finding, 'line' | 'column' | 'snippet'>> {
  const hit = findFirst(content, regex);
  if (!hit) return {};
  return { line: hit.line, column: hit.column, snippet: hit.snippet };
}

const adapters: readonly AdapterDefinition[] = [
  {
    id: 'vercel',
    detect: ({ signals }) => signals.vercelBunEnabled,
    run: ({ root, snapshot, signals }) => {
      const findings: Finding[] = [];
      const vercelJsonPath = path.join(root, 'vercel.json');
      const vercelJson = snapshot.readJson<VercelConfig>(vercelJsonPath);
      const vercelJsonText = snapshot.readText(vercelJsonPath);
      if (vercelJsonText && !signals.vercelBunEnabled) {
        findings.push(
          createFinding(
            root,
            'vercel-bun-runtime-enable',
            'info',
            'vercel.json',
            'vercel.json exists but no bunVersion was detected. If you intend to use Bun runtime, set bunVersion.',
          ),
        );
      }
      if (vercelJson && typeof vercelJson.bunVersion === 'string' && vercelJson.bunVersion !== '1.x') {
        findings.push(
          createFinding(
            root,
            'vercel-bun-runtime-enable',
            'warn',
            'vercel.json',
            `bunVersion is "${vercelJson.bunVersion}". Prefer "1.x" unless you have a strong pinning reason.`,
          ),
        );
      }
      if (!snapshot.exists(path.join(root, 'bun.lockb'))) {
        findings.push(
          createFinding(
            root,
            'vercel-bun-install-detection',
            'warn',
            'vercel.json',
            'Bun runtime is enabled but bun.lockb is missing. Add/commit bun.lockb to ensure Bun installs on Vercel.',
          ),
        );
      }
      const middleware = snapshot.readText(path.join(root, 'middleware.ts'));
      if (middleware && !/runtime\s*=\s*['"]nodejs['"]/.test(middleware)) {
        findings.push(
          createFinding(
            root,
            'vercel-bun-runtime-limitations',
            'info',
            'middleware.ts',
            'When using Vercel Routing Middleware with Bun runtime, middleware should declare the nodejs runtime.',
            {
              confidence: 'medium',
            },
          ),
        );
      }
      for (const file of snapshot.walkFiles('vercel')) {
        const extension = path.extname(file).replace(/^\./, '').toLowerCase();
        if (!JS_EXTENSIONS.has(extension)) continue;
        const content = snapshot.readText(file);
        if (!content) continue;
        const hit = findFirst(content, /\bBun\.serve\s*\(/);
        if (!hit) continue;
        findings.push(
          createFinding(
            root,
            'vercel-bun-runtime-limitations',
            'warn',
            file,
            'Bun.serve() is not supported in Vercel Functions; use a supported handler or framework adapter.',
            {
              line: hit.line,
              column: hit.column,
              snippet: hit.snippet,
            },
          ),
        );
      }
      return findings;
    },
  },
  {
    id: 'github-actions',
    detect: ({ root }) => {
      try {
        return readdirSync(path.join(root, '.github', 'workflows')).length >= 0;
      } catch {
        return false;
      }
    },
    run: ({ root, snapshot, signals }) => {
      const findings: Finding[] = [];
      const workflowDir = path.join(root, '.github', 'workflows');
      let files: string[] = [];
      try {
        files = readdirSync(workflowDir)
          .filter((name) => /\.(ya?ml)$/i.test(name))
          .map((name) => path.join(workflowDir, name));
      } catch {
        return findings;
      }
      for (const file of files) {
        const content = snapshot.readText(file);
        if (!content) continue;
        if (signals.bunFirst && /\b(npm ci|npm install|pnpm install|yarn install)\b/.test(content)) {
          findings.push(
            createFinding(
              root,
              'scripts-no-npm-in-bun-repos',
              'warn',
              file,
              'GitHub Actions workflow uses npm/pnpm/yarn install steps in a Bun-first repo.',
              {
                ...findLine(content, /\b(npm ci|npm install|pnpm install|yarn install)\b/),
                suggestedFix: 'Prefer Bun install steps and Bun-native execution in workflows.',
              },
            ),
          );
        }
        if (/\bnpx\b/.test(content)) {
          findings.push(
            createFinding(
              root,
              'pm-bunx-vs-npx',
              'warn',
              file,
              'GitHub Actions workflow uses npx. Prefer bunx in Bun-first repos.',
              {
                ...findLine(content, /\bnpx\b/),
              },
            ),
          );
        }
        if (snapshot.exists(path.join(root, 'bun.lockb')) && /\bbun install\b/.test(content) && !/\bbun install --frozen-lockfile\b|\bbun ci\b/.test(content)) {
          findings.push(
            createFinding(
              root,
              'pm-bun-install-ci-frozen-lockfile',
              'info',
              file,
              'GitHub Actions workflow runs `bun install` without frozen lockfile mode.',
              {
                ...findLine(content, /\bbun install\b/),
                suggestedFix: 'Prefer `bun install --frozen-lockfile` or `bun ci` in CI.',
              },
            ),
          );
        }
      }
      return findings;
    },
  },
  {
    id: 'docker',
    detect: ({ snapshot }) => snapshot.walkFiles('docker-detect').some((file) => /(^|\/)Dockerfile[^/]*$/i.test(file)),
    run: ({ root, snapshot, signals }) => {
      const findings: Finding[] = [];
      for (const file of snapshot.walkFiles('docker')) {
        if (!/(^|\/)Dockerfile[^/]*$/i.test(file)) continue;
        const content = snapshot.readText(file);
        if (!content) continue;
        if (signals.bunFirst && /\bFROM\s+node[:\s]/i.test(content)) {
          findings.push(
            createFinding(
              root,
              'runtime-bun-vs-node-choose',
              'info',
              file,
              'Dockerfile still uses a Node base image in a Bun-first repo.',
              {
                ...findLine(content, /\bFROM\s+node[:\s]/i),
                suggestedFix: 'Use an explicit Bun image or document why Node remains required.',
              },
            ),
          );
        }
        if (signals.bunFirst && /\b(npm ci|npm install|pnpm install|yarn install)\b/i.test(content)) {
          findings.push(
            createFinding(
              root,
              'scripts-no-npm-in-bun-repos',
              'warn',
              file,
              'Dockerfile uses npm/pnpm/yarn install commands in a Bun-first repo.',
              {
                ...findLine(content, /\b(npm ci|npm install|pnpm install|yarn install)\b/i),
              },
            ),
          );
        }
        if (snapshot.exists(path.join(root, 'bun.lockb')) && /\bbun install\b/i.test(content) && !/\bbun install --frozen-lockfile\b|\bbun ci\b/i.test(content)) {
          findings.push(
            createFinding(
              root,
              'pm-bun-install-ci-frozen-lockfile',
              'info',
              file,
              'Dockerfile runs `bun install` without frozen lockfile mode.',
              {
                ...findLine(content, /\bbun install\b/i),
              },
            ),
          );
        }
      }
      return findings;
    },
  },
  {
    id: 'monorepo',
    detect: ({ signals }) => signals.hasWorkspaces,
    run: ({ root, snapshot }) => {
      const findings: Finding[] = [];
      const packageJson = snapshot.readJson<PackageJson>(path.join(root, 'package.json'));
      for (const [name, command] of Object.entries(packageJson?.scripts ?? {})) {
        if (!['build', 'test', 'lint', 'typecheck'].includes(name)) continue;
        if (/^\s*bun run /.test(command) && !/--filter|--workspaces/.test(command)) {
          findings.push(
            createFinding(
              root,
              'scripts-bun-filter-and-workspaces',
              'info',
              'package.json',
              `Root monorepo script "${name}" runs via bun without --filter/--workspaces.`,
              {
                suggestedFix: 'Use `bun run --workspaces <script>` or `bun run --filter <glob> <script>` when coordinating workspace tasks.',
              },
            ),
          );
        }
      }
      return findings;
    },
  },
];

function runAdapters(root: string, snapshot: RepoSnapshot, signals: RepoSignals, config: AuditConfig): { findings: readonly Finding[]; adapterIds: readonly string[] } {
  const requested = resolveAdapterIds(signals, config);
  const active = adapters.filter((adapter) => requested.includes(adapter.id) && adapter.detect({ root, snapshot, signals }));
  return {
    findings: active.flatMap((adapter) => adapter.run({ root, snapshot, signals })),
    adapterIds: active.map((adapter) => adapter.id),
  };
}

export function runAudit(root: string, config: AuditConfig): readonly Finding[] {
  const snapshot = createRepoSnapshot(root, config);
  const fingerprint = buildRepoFingerprint(snapshot, config);
  using cacheDb = openCacheDatabase(root, config);
  const cached = cacheDb
    .query(
      'SELECT findings_json, adapter_ids_json FROM scan_cache WHERE root = ?1 AND fingerprint = ?2 AND engine_version = ?3',
    )
    .get(root, fingerprint, ENGINE_VERSION) as { findings_json: string; adapter_ids_json: string } | null;
  if (cached) {
    return JSON.parse(cached.findings_json) as Finding[];
  }

  const signals = inferSignals(snapshot);
  const findings: Finding[] = [];
  const add = (finding: Finding) => findings.push(finding);

  const packageJsonPath = path.join(root, 'package.json');
  const packageJson = snapshot.readJson<PackageJson>(packageJsonPath);
  const tsconfigPath = path.join(root, 'tsconfig.json');
  const tsconfig = snapshot.readJson<TsConfig>(tsconfigPath);
  const gitignorePath = path.join(root, '.gitignore');
  const bunfigPath = path.join(root, 'bunfig.toml');
  const bunfig = snapshot.readText(bunfigPath);

  const lockfiles = [
    'bun.lockb',
    'package-lock.json',
    'pnpm-lock.yaml',
    'yarn.lock',
  ].filter((name) => snapshot.exists(path.join(root, name)));

  if (lockfiles.length > 1) {
    add(
      createFinding(
        root,
        'pm-no-mixed-lockfiles',
        'error',
        lockfiles[0]!,
        `Multiple lockfiles detected: ${lockfiles.join(', ')}. Pick one package manager and delete the others.`,
        {
          suggestedFix: 'Remove non-Bun lockfiles and keep bun.lockb as the single source of truth.',
        },
      ),
    );
  }

  if (packageJson) {
    if (!packageJson.packageManager) {
      add(
        createFinding(root, 'pm-package-manager-field', 'info', 'package.json', 'Missing `packageManager` field.', {
          suggestedFix: 'Add `"packageManager": "bun@<version>"` when the repo is Bun-first.',
        }),
      );
    } else if (!packageJson.packageManager.startsWith('bun@') && signals.bunFirst) {
      add(
        createFinding(
          root,
          'pm-package-manager-field',
          'warn',
          'package.json',
          `packageManager is "${packageJson.packageManager}". If this repo is Bun-first, set it to "bun@<version>".`,
        ),
      );
    }

    const scripts = packageJson.scripts ?? {};
    for (const [name, command] of Object.entries(scripts)) {
      if (/\bnpx\b/.test(command)) {
        add(
          createFinding(root, 'pm-bunx-vs-npx', 'warn', 'package.json', `Script "${name}" uses npx. Prefer bunx.`, {
            suggestedFix: 'Replace `npx` with `bunx` in package.json scripts.',
          }),
        );
      }
      if (/\b(npm|pnpm|yarn)\b/.test(command)) {
        add(
          createFinding(
            root,
            'scripts-no-npm-in-bun-repos',
            'warn',
            'package.json',
            `Script "${name}" uses another package manager. Prefer bun install/bun run/bunx for consistency.`,
          ),
        );
      }
      if (/\bconcurrently\b|\bnpm-run-all\b|\brun-p\b/.test(command)) {
        add(
          createFinding(
            root,
            'scripts-bun-run-parallel-sequential',
            'info',
            'package.json',
            `Script "${name}" uses an orchestration package. Bun supports --parallel/--sequential.`,
          ),
        );
      }
      if (/\bnodemon\b/.test(command)) {
        add(
          createFinding(
            root,
            'runtime-watch-and-hot-reload',
            'info',
            'package.json',
            `Script "${name}" uses nodemon. Bun has built-in --watch/--hot.`,
          ),
        );
      }
      if (/\b(ts-node|tsx)\b/.test(command)) {
        add(
          createFinding(
            root,
            'runtime-ts-direct-execution',
            'info',
            'package.json',
            `Script "${name}" uses ts-node/tsx. If Bun is your runtime, prefer running TS directly with bun.`,
          ),
        );
      }
      if (signals.vercelBunEnabled && name === 'dev' && /^\s*next dev\s*$/.test(command)) {
        add(
          createFinding(
            root,
            'vercel-nextjs-bun-runtime-scripts',
            'warn',
            'package.json',
            'Vercel Bun runtime is enabled, but the dev script still uses `next dev` directly.',
            {
              suggestedFix: 'Use `bun run --bun next dev`.',
            },
          ),
        );
      }
      if (signals.vercelBunEnabled && name === 'build' && /^\s*next build\s*$/.test(command)) {
        add(
          createFinding(
            root,
            'vercel-nextjs-bun-runtime-scripts',
            'warn',
            'package.json',
            'Vercel Bun runtime is enabled, but the build script still uses `next build` directly.',
            {
              suggestedFix: 'Use `bun run --bun next build`.',
            },
          ),
        );
      }
    }

    const hasBunTypesDependency = Boolean(packageJson.devDependencies?.['@types/bun']);
    if (tsconfig) {
      const compilerOptions = tsconfig.compilerOptions ?? {};
      const moduleResolution = String(compilerOptions.moduleResolution ?? '');
      if (moduleResolution && moduleResolution.toLowerCase() !== 'bundler') {
        add(
          createFinding(
            root,
            'tsconfig-module-resolution-bundler',
            'warn',
            'tsconfig.json',
            `compilerOptions.moduleResolution is "${moduleResolution}". Bun generally expects "Bundler".`,
          ),
        );
      }

      const recommendedChecks: Array<Readonly<[string, (value: unknown) => boolean, string]>> = [
        ['target', (value) => String(value).toLowerCase() === 'esnext', 'target: "ESNext"'],
        ['module', (value) => String(value).toLowerCase() === 'preserve', 'module: "Preserve"'],
        ['allowImportingTsExtensions', (value) => value === true, 'allowImportingTsExtensions: true'],
        ['verbatimModuleSyntax', (value) => value === true, 'verbatimModuleSyntax: true'],
        ['noEmit', (value) => value === true, 'noEmit: true'],
      ];
      const missing = recommendedChecks.filter(([key, predicate]) => !predicate(compilerOptions[key]));
      if (missing.length > 0) {
        add(
          createFinding(
            root,
            'tsconfig-bun-recommended',
            'info',
            'tsconfig.json',
            `tsconfig.json is missing Bun-friendly options: ${missing.map(([, , hint]) => hint).join(', ')}.`,
          ),
        );
      }

      const types = Array.isArray(compilerOptions.types) ? compilerOptions.types.map(String) : [];
      if (!types.includes('bun-types') && (hasBunTypesDependency || signals.bunFirst)) {
        add(
          createFinding(
            root,
            'tsconfig-bun-types',
            'info',
            'tsconfig.json',
            'Consider adding compilerOptions.types: ["bun-types"] for Bun globals/types.',
          ),
        );
      }
    }
  }

  const gitignore = snapshot.readText(gitignorePath);
  if (gitignore && /\bbun\.lockb\b/.test(gitignore)) {
    add(
      createFinding(
        root,
        'pm-commit-bun-lockb',
        'warn',
        '.gitignore',
        'bun.lockb is ignored. Prefer committing bun.lockb for deterministic installs.',
      ),
    );
  }

  if (snapshot.exists(path.join(root, '.nvmrc')) && snapshot.exists(path.join(root, 'bun.lockb'))) {
    add(
      createFinding(
        root,
        'runtime-bun-vs-node-choose',
        'info',
        '.nvmrc',
        'Found both .nvmrc and bun.lockb. If Node is not required, consider removing Node-only runtime pinning.',
      ),
    );
  }

  if (bunfig) {
    if (/^\s*\[test\][\s\S]*^\s*retry\s*=\s*[1-9]/m.test(bunfig)) {
      add(
        createFinding(
          root,
          'test-bun-retry',
          'info',
          'bunfig.toml',
          'bunfig.toml sets a default test retry count. Keep retries low and prefer fixing flaky tests.',
          {
            confidence: 'medium',
          },
        ),
      );
    }
    if (/^\s*\[install\][\s\S]*^\s*frozenLockfile\s*=\s*false/m.test(bunfig) && snapshot.exists(path.join(root, 'bun.lockb'))) {
      add(
        createFinding(
          root,
          'pm-bun-install-ci-frozen-lockfile',
          'info',
          'bunfig.toml',
          'bunfig.toml explicitly disables frozen lockfile installs. CI should prefer frozen lockfile mode.',
          {
            suggestedFix: 'Use `bun install --frozen-lockfile` or `bun ci` in CI.',
          },
        ),
      );
    }
    if (/^\s*\[run\][\s\S]*^\s*bun\s*=\s*false/m.test(bunfig) && signals.bunFirst) {
      add(
        createFinding(
          root,
          'runtime-bun-run-bun-flag',
          'info',
          'bunfig.toml',
          'bunfig.toml disables run.bun. Binaries with Node shebangs may execute under Node instead of Bun.',
          {
            suggestedFix: 'Enable `[run] bun = true` when you intentionally want Bun to execute Node-shebang binaries.',
          },
        ),
      );
    }
  }

  const adapterResult = runAdapters(root, snapshot, signals, config);
  const normalized = normalizeFindings([...findings, ...adapterResult.findings], config);
  cacheDb
    .query(
      'INSERT OR REPLACE INTO scan_cache (root, fingerprint, engine_version, findings_json, adapter_ids_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)',
    )
    .run(
      root,
      fingerprint,
      ENGINE_VERSION,
      JSON.stringify(normalized),
      JSON.stringify(adapterResult.adapterIds),
      new Date().toISOString(),
    );
  return normalized;
}

function makePackageJsonRewriteFix(
  root: string,
  packageJsonPath: string,
  current: PackageJson,
  transform: (value: Record<string, unknown>) => Record<string, unknown>,
  ruleId: string,
  description: string,
): PlannedFix | undefined {
  const next = transform(JSON.parse(JSON.stringify(current)) as Record<string, unknown>);
  const before = `${JSON.stringify(current, null, 2)}\n`;
  const after = `${JSON.stringify(next, null, 2)}\n`;
  if (before === after) return undefined;
  return {
    ruleId,
    ruleIds: [ruleId],
    kind: 'safe',
    file: packageJsonPath,
    description,
    before,
    after,
    apply: () => {
      writeFileSync(packageJsonPath, after, 'utf8');
    },
  };
}

export function planSafeFixes(root: string, config: AuditConfig): readonly PlannedFix[] {
  const snapshot = createRepoSnapshot(root, config);
  const signals = inferSignals(snapshot);
  const fixes: PlannedFix[] = [];
  const packageJsonPath = path.join(root, 'package.json');
  const packageJson = snapshot.readJson<PackageJson>(packageJsonPath);

  if (packageJson) {
    const packageJsonFixRuleIds: string[] = [];
    const packageJsonDescriptions: string[] = [];

    const fix = makePackageJsonRewriteFix(
      root,
      packageJsonPath,
      packageJson,
      (value) => {
        const nextValue = { ...value };
        const scripts = {
          ...((nextValue.scripts as Record<string, string> | undefined) ?? {}),
        };

        if (signals.bunFirst && !nextValue.packageManager) {
          packageJsonFixRuleIds.push('pm-package-manager-field');
          packageJsonDescriptions.push(`add packageManager bun@${Bun.version}`);
          nextValue.packageManager = `bun@${Bun.version}`;
        }

        if (signals.bunFirst && Object.values(scripts).some((command) => /\bnpx\b/.test(command))) {
          packageJsonFixRuleIds.push('pm-bunx-vs-npx');
          packageJsonDescriptions.push('rewrite npx invocations to bunx');
          for (const [name, command] of Object.entries(scripts)) {
            scripts[name] = command.replace(/\bnpx\b/g, 'bunx');
          }
        }

        if (signals.vercelBunEnabled) {
          if (/^\s*next dev\s*$/.test(scripts.dev ?? '') || /^\s*next build\s*$/.test(scripts.build ?? '')) {
            packageJsonFixRuleIds.push('vercel-nextjs-bun-runtime-scripts');
            packageJsonDescriptions.push('normalize Next.js dev/build scripts for Bun runtime');
            if (/^\s*next dev\s*$/.test(scripts.dev ?? '')) scripts.dev = 'bun run --bun next dev';
            if (/^\s*next build\s*$/.test(scripts.build ?? '')) scripts.build = 'bun run --bun next build';
          }
        }

        return {
          ...nextValue,
          scripts,
        };
      },
      packageJsonFixRuleIds[0] ?? 'pm-bunx-vs-npx',
      packageJsonDescriptions.length > 0
        ? `Update package.json to ${packageJsonDescriptions.join('; ')}.`
        : 'No safe package.json updates were needed.',
    );

    if (fix && packageJsonFixRuleIds.length > 0) {
      fixes.push({
        ...fix,
        ruleId: packageJsonFixRuleIds[0]!,
        ruleIds: packageJsonFixRuleIds,
        description: `Update package.json to ${packageJsonDescriptions.join('; ')}.`,
      });
    }
  }

  return fixes;
}

export function applySafeFixes(root: string, config: AuditConfig): readonly PlannedFix[] {
  const fixes = planSafeFixes(root, config);
  if (fixes.length === 0) return fixes;
  ensureDir(getRollbackDir(root));
  const rollbackPath = path.join(
    getRollbackDir(root),
    `${new Date().toISOString().replace(/[:.]/g, '-')}.json`,
  );
  writeFileSync(
    rollbackPath,
    JSON.stringify(
      {
        root,
        createdAt: new Date().toISOString(),
        fixes: fixes.map((fix) => ({
          ruleId: fix.ruleId,
          ruleIds: fix.ruleIds,
          file: fix.file,
          before: fix.before,
          after: fix.after,
        })),
      },
      null,
      2,
    ),
  );
  for (const fix of fixes) fix.apply();
  return fixes;
}

function runValidationCommands(root: string, config: AuditConfig): readonly string[] {
  const packageJson = safeReadJson<PackageJson>(path.join(root, 'package.json'));
  const commands = config.validationCommands.length > 0
    ? [...config.validationCommands]
    : [
        fileExists(path.join(root, 'bun.lockb')) ? 'bun install --frozen-lockfile' : '',
        packageJson?.scripts?.lint ? 'bun run lint' : '',
        packageJson?.scripts?.typecheck ? 'bun run typecheck' : '',
        packageJson?.scripts?.test ? 'bun run test' : '',
        packageJson?.scripts?.build ? 'bun run build' : '',
      ].filter(Boolean);

  for (const command of commands) {
    const proc = Bun.spawnSync({
      cmd: ['zsh', '-lc', command],
      cwd: root,
      stdout: 'pipe',
      stderr: 'pipe',
    });
    const stdout = proc.stdout.toString().trim();
    const stderr = proc.stderr.toString().trim();
    if (stdout) console.log(stdout);
    if (stderr) console.error(stderr);
    if (proc.exitCode !== 0) {
      throw new Error(`Validation command failed: ${command}`);
    }
  }
  return commands;
}

function hashFileOrMissing(filePath: string): string {
  const content = safeReadText(filePath);
  if (content === undefined) return 'missing';
  return createHash('sha256').update(content).digest('hex');
}

export function createReleaseSyncReport(context: SkillContext): {
  syncedAt: string;
  references: readonly { file: string; hash: string }[];
  capabilityMap: readonly {
    topic: string;
    source: string;
    matched: boolean;
    rules: readonly string[];
    classification: 'docs-only' | 'capability-present' | 'missing-rule';
  }[];
} {
  const bunReleasePath = path.join(context.referencesDir, 'ref-bun-release-notes-bun-v1.3.10.md');
  const vercelPath = path.join(context.referencesDir, 'ref-vercel-bun-runtime.md');
  const bunRelease = safeReadText(bunReleasePath) ?? '';
  const vercelDoc = safeReadText(vercelPath) ?? '';
  const ruleIds = new Set(listRuleIds(context));
  const capabilities = [
    {
      topic: 'bun test retry',
      source: 'bun-release-notes',
      matched: /\bbun test --retry\b/.test(bunRelease),
      rules: ['test-bun-retry', 'test-bun-test-runner'],
    },
    {
      topic: 'bun build compile browser',
      source: 'bun-release-notes',
      matched: /\bbun build --compile --target=browser\b/.test(bunRelease),
      rules: ['build-bun-compile-browser', 'build-bun-build-bundler'],
    },
    {
      topic: 'bun parallel sequential scripts',
      source: 'bun-release-notes',
      matched: /--parallel|--sequential/.test(bunRelease),
      rules: ['scripts-bun-run-parallel-sequential'],
    },
    {
      topic: 'vercel bunVersion and next scripts',
      source: 'vercel-bun-runtime',
      matched: /bunVersion|bun run --bun next dev|bun run --bun next build/.test(vercelDoc),
      rules: ['vercel-bun-runtime-enable', 'vercel-nextjs-bun-runtime-scripts'],
    },
  ] as const;

  return {
    syncedAt: new Date().toISOString(),
    references: [
      { file: path.basename(bunReleasePath), hash: hashFileOrMissing(bunReleasePath) },
      { file: path.basename(vercelPath), hash: hashFileOrMissing(vercelPath) },
    ],
    capabilityMap: capabilities.map((capability) => ({
      ...capability,
      classification: capability.matched
        ? capability.rules.every((ruleId) => ruleIds.has(ruleId))
          ? 'capability-present'
          : 'missing-rule'
        : 'docs-only',
    })),
  };
}

function printFindingsText(findings: readonly Finding[]): void {
  if (findings.length === 0) {
    console.log('OK: no Bun platform audit findings.');
    return;
  }

  for (const finding of findings) {
    const location = `${finding.file}:${finding.line}:${finding.column}`;
    const tag = finding.severity.toUpperCase().padEnd(5);
    console.log(`${tag} ${finding.ruleId} ${location}`);
    console.log(`  ${finding.message}`);
    if (finding.why) console.log(`  Why: ${finding.why}`);
    if (finding.suggestedFix) console.log(`  Fix: ${finding.suggestedFix}`);
    if (finding.snippet) console.log(`  ${finding.snippet}`);
  }
}

function printFindingsMd(findings: readonly Finding[]): void {
  if (findings.length === 0) {
    console.log('OK: no Bun platform audit findings.');
    return;
  }
  console.log('# Bun Platform Audit Findings\n');
  for (const finding of findings) {
    const location = `${finding.file}:${finding.line}:${finding.column}`;
    console.log(`- **${finding.severity.toUpperCase()}** \`${finding.ruleId}\` (${location}): ${finding.message}`);
    if (finding.suggestedFix) console.log(`  - Fix: ${finding.suggestedFix}`);
  }
}

function printFixesText(fixes: readonly PlannedFix[], applied: boolean): void {
  if (fixes.length === 0) {
    console.log(applied ? 'OK: no safe fixes were applicable.' : 'OK: no safe fixes were planned.');
    return;
  }
  console.log(applied ? `Applied ${fixes.length} safe fix(es):` : `Planned ${fixes.length} safe fix(es):`);
  for (const fix of fixes) {
    console.log(`- [${fix.kind}] ${fix.ruleId} ${fix.file}`);
    console.log(`  ${fix.description}`);
  }
}

function shouldFail(findings: readonly Finding[], severity: Severity): boolean {
  return findings.some((finding) => severityRank(finding.severity) >= severityRank(severity));
}

async function runReleaseSync(context: SkillContext): Promise<void> {
  const scripts = [
    path.join(context.scriptsDir, 'update-bun-release-notes.ts'),
    path.join(context.scriptsDir, 'update-vercel-bun-docs.ts'),
    path.join(context.scriptsDir, 'build-rules-index.ts'),
    path.join(context.scriptsDir, 'check-skill-integrity.ts'),
  ];

  for (const script of scripts) {
    const proc = Bun.spawnSync({
      cmd: ['bun', script],
      stdout: 'pipe',
      stderr: 'pipe',
    });
    if (proc.exitCode !== 0) {
      throw new Error(`release-sync failed for ${path.basename(script)}:\n${proc.stderr.toString()}`);
    }
    const stdout = proc.stdout.toString().trim();
    if (stdout) console.log(stdout);
  }

  const report = createReleaseSyncReport(context);
  const reportPath = path.join(context.referencesDir, 'release-sync-report.json');
  writeFileSync(reportPath, JSON.stringify(report, null, 2), 'utf8');
  using cacheDb = openCacheDatabase(context.skillRoot, loadAuditConfig(context.skillRoot));
  cacheDb
    .query('INSERT INTO release_sync_runs (root, synced_at, report_json) VALUES (?1, ?2, ?3)')
    .run(context.skillRoot, report.syncedAt, JSON.stringify(report));
  console.log(`Wrote:\n- ${reportPath}`);
}

export async function runCli(context: SkillContext, argv: readonly string[]): Promise<void> {
  const args = parseCliArgs(argv);
  const config = loadAuditConfig(args.root, args.configPath, args);

  switch (args.command) {
    case 'list-rules': {
      for (const id of listRuleIds(context)) console.log(id);
      return;
    }
    case 'explain': {
      if (!args.explain) throw new Error('Missing rule id for explain.');
      console.log(explainRule(context, args.explain));
      return;
    }
    case 'plan-fixes': {
      const fixes = planSafeFixes(args.root, config);
      if (args.format === 'json') {
        console.log(
          JSON.stringify(
            fixes.map((fix) => ({
              ruleId: fix.ruleId,
              ruleIds: fix.ruleIds,
              kind: fix.kind,
              file: fix.file,
              description: fix.description,
            })),
            null,
            2,
          ),
        );
      } else {
        printFixesText(fixes, false);
      }
      return;
    }
    case 'apply-safe-fixes': {
      const fixes = applySafeFixes(args.root, config);
      if (args.format === 'json') {
        console.log(
          JSON.stringify(
            fixes.map((fix) => ({
              ruleId: fix.ruleId,
              ruleIds: fix.ruleIds,
              kind: fix.kind,
              file: fix.file,
              description: fix.description,
            })),
            null,
            2,
          ),
        );
      } else {
        printFixesText(fixes, true);
      }
      return;
    }
    case 'validate': {
      const findings = runAudit(args.root, config);
      const failOn = args.failOn ?? 'warn';
      printFindingsText(findings);
      if (shouldFail(findings, failOn)) process.exit(1);
      const commands = runValidationCommands(args.root, config);
      if (commands.length > 0) {
        console.log(`Validated ${commands.length} command(s).`);
      }
      return;
    }
    case 'benchmark': {
      const snapshot = createRepoSnapshot(args.root, config);
      const signals = inferSignals(snapshot);
      const started = performance.now();
      const findings = runAudit(args.root, config);
      const auditMs = performance.now() - started;
      const fixStarted = performance.now();
      const fixes = planSafeFixes(args.root, config);
      const fixMs = performance.now() - fixStarted;
      using cacheDb = openCacheDatabase(args.root, config);
      const cacheEntries = cacheDb.query('SELECT COUNT(*) AS count FROM scan_cache WHERE root = ?1').get(args.root) as {
        count: number;
      };
      const activeAdapters = runAdapters(args.root, snapshot, signals, config).adapterIds;
      console.log(
        JSON.stringify(
          {
            root: args.root,
            findings: findings.length,
            plannedSafeFixes: fixes.length,
            adapters: activeAdapters,
            cacheEntries: cacheEntries.count,
            timingsMs: {
              audit: Number(auditMs.toFixed(2)),
              planFixes: Number(fixMs.toFixed(2)),
            },
          },
          null,
          2,
        ),
      );
      return;
    }
    case 'release-sync': {
      await runReleaseSync(context);
      return;
    }
    case 'audit':
    default: {
      const findings = runAudit(args.root, config);
      if (args.format === 'json') console.log(JSON.stringify(findings, null, 2));
      else if (args.format === 'md') printFindingsMd(findings);
      else printFindingsText(findings);
      if (args.failOn && shouldFail(findings, args.failOn)) process.exit(1);
    }
  }
}

export function ensureDir(dirPath: string): void {
  mkdirSync(dirPath, { recursive: true });
}
