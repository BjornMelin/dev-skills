import {
  createReleaseSyncReport,
  createSkillContext,
  applySafeFixes,
  loadAuditConfig,
  planSafeFixes,
  runAudit,
} from '../lib/bun-platform-core';
import {
  cpSync,
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { afterEach, describe, expect, test } from 'bun:test';

const fixturesRoot = path.join(import.meta.dir, 'fixtures');
const tempRoots: string[] = [];

function copyFixture(name: string): string {
  const target = mkdtempSync(path.join(os.tmpdir(), `bun-platform-${name}-`));
  cpSync(path.join(fixturesRoot, name), target, { recursive: true });
  tempRoots.push(target);
  return target;
}

afterEach(() => {
  while (tempRoots.length > 0) {
    rmSync(tempRoots.pop()!, { recursive: true, force: true });
  }
});

describe('bun platform audit engine', () => {
  test('reports mixed lockfiles as an error', () => {
    const root = copyFixture('mixed-lockfiles');
    const findings = runAudit(root, loadAuditConfig(root));

    expect(findings.some((finding) => finding.ruleId === 'pm-no-mixed-lockfiles' && finding.severity === 'error')).toBe(
      true,
    );
  });

  test('plans and applies safe package.json fixes', () => {
    const root = copyFixture('safe-fixes');

    const planned = planSafeFixes(root, loadAuditConfig(root));
    expect(planned.flatMap((fix) => [...fix.ruleIds]).sort()).toEqual(['pm-bunx-vs-npx', 'pm-package-manager-field']);

    applySafeFixes(root, loadAuditConfig(root));
    const packageJson = readFileSync(path.join(root, 'package.json'), 'utf8');
    expect(packageJson).toContain(`"packageManager": "bun@${Bun.version}"`);
    expect(packageJson).toContain('"gen": "bunx prisma generate"');
  });

  test('normalizes Next.js scripts when Vercel Bun runtime is enabled', () => {
    const root = copyFixture('vercel-next');

    const findings = runAudit(root, loadAuditConfig(root));
    expect(findings.some((finding) => finding.ruleId === 'vercel-nextjs-bun-runtime-scripts')).toBe(true);

    const planned = planSafeFixes(root, loadAuditConfig(root));
    expect(planned.some((fix) => fix.ruleIds.includes('vercel-nextjs-bun-runtime-scripts'))).toBe(true);

    applySafeFixes(root, loadAuditConfig(root));
    const packageJson = readFileSync(path.join(root, 'package.json'), 'utf8');
    expect(packageJson).toContain('"dev": "bun run --bun next dev"');
    expect(packageJson).toContain('"build": "bun run --bun next build"');
  });

  test('respects disabled rules and baseline suppressions from config', () => {
    const root = copyFixture('safe-fixes');
    const baselinePath = path.join(root, 'baseline.json');
    writeFileSync(baselinePath, JSON.stringify(['pm-package-manager-field:package.json'], null, 2));
    writeFileSync(
      path.join(root, 'bun-platform.config.json'),
      JSON.stringify(
        {
          disabledRules: ['pm-bunx-vs-npx'],
          baseline: './baseline.json',
        },
        null,
        2,
      ),
    );

    const findings = runAudit(root, loadAuditConfig(root));
    expect(findings.map((finding) => finding.ruleId)).not.toContain('pm-package-manager-field');
    expect(findings.map((finding) => finding.ruleId)).not.toContain('pm-bunx-vs-npx');
  });

  test('creates a rollback artifact when safe fixes are applied', () => {
    const root = copyFixture('safe-fixes');
    applySafeFixes(root, loadAuditConfig(root));

    const rollbackDir = path.join(root, '.bun-platform', 'rollbacks');
    const entries = Bun.spawnSync({
      cmd: ['zsh', '-lc', `find "${rollbackDir}" -type f | wc -l`],
      stdout: 'pipe',
      stderr: 'pipe',
    });
    expect(entries.stdout.toString().trim()).toBe('1');
  });

  test('caches audit results on disk', () => {
    const root = copyFixture('mixed-lockfiles');
    runAudit(root, loadAuditConfig(root));

    const cachePath = path.join(root, '.bun-platform', 'cache.sqlite');
    expect(Bun.file(cachePath).size).toBeGreaterThan(0);
  });

  test('auto-adds .bun-platform to .gitignore when platform state is created', () => {
    const root = copyFixture('mixed-lockfiles');
    runAudit(root, loadAuditConfig(root));

    const gitignore = readFileSync(path.join(root, '.gitignore'), 'utf8');
    expect(gitignore).toContain('# Bun platform state');
    expect(gitignore).toContain('.bun-platform/');
  });

  test('does not duplicate the .bun-platform gitignore entry', () => {
    const root = copyFixture('mixed-lockfiles');
    runAudit(root, loadAuditConfig(root));
    runAudit(root, loadAuditConfig(root));

    const gitignore = readFileSync(path.join(root, '.gitignore'), 'utf8');
    expect(gitignore.match(/\.bun-platform\//g)?.length ?? 0).toBe(1);
  });

  test('supports disabling gitignore management in config', () => {
    const root = copyFixture('mixed-lockfiles');
    writeFileSync(
      path.join(root, 'bun-platform.config.json'),
      JSON.stringify(
        {
          manageGitignore: false,
        },
        null,
        2,
      ),
    );

    runAudit(root, loadAuditConfig(root));
    expect(existsSync(path.join(root, '.gitignore'))).toBe(false);
  });

  test('reports GitHub Actions adapter findings', () => {
    const root = copyFixture('github-actions');
    const findings = runAudit(root, loadAuditConfig(root));

    expect(findings.some((finding) => finding.ruleId === 'scripts-no-npm-in-bun-repos')).toBe(true);
    expect(findings.some((finding) => finding.ruleId === 'pm-bun-install-ci-frozen-lockfile')).toBe(true);
  });

  test('reports Docker adapter findings', () => {
    const root = copyFixture('docker');
    const findings = runAudit(root, loadAuditConfig(root));

    expect(findings.some((finding) => finding.ruleId === 'runtime-bun-vs-node-choose')).toBe(true);
    expect(findings.some((finding) => finding.ruleId === 'scripts-no-npm-in-bun-repos')).toBe(true);
  });

  test('reports monorepo adapter findings', () => {
    const root = copyFixture('monorepo');
    const findings = runAudit(root, loadAuditConfig(root));

    expect(findings.some((finding) => finding.ruleId === 'scripts-bun-filter-and-workspaces')).toBe(true);
  });

  test('creates a release sync report from current references', () => {
    const context = createSkillContext(pathToFileURL(path.join(import.meta.dir, '..', 'bun-platform.ts')).href);
    const report = createReleaseSyncReport(context);

    expect(report.references.length).toBeGreaterThan(0);
    expect(report.capabilityMap.some((entry) => entry.topic === 'bun test retry' && entry.classification === 'capability-present')).toBe(
      true,
    );
  });

  test('validate command executes Bun-native repo checks', () => {
    const root = copyFixture('validate-success');
    const proc = Bun.spawnSync({
      cmd: ['bun', path.join(import.meta.dir, '..', 'bun-platform.ts'), 'validate', '--root', root, '--fail-on', 'warn'],
      stdout: 'pipe',
      stderr: 'pipe',
    });

    expect(proc.exitCode).toBe(0);
    expect(proc.stdout.toString()).toContain('Validated 4 command(s).');
  });

  test('benchmark command runs against a temp fixture copy and not the committed fixture', () => {
    const root = copyFixture('github-actions');
    const proc = Bun.spawnSync({
      cmd: ['bun', path.join(import.meta.dir, '..', 'bun-platform.ts'), 'benchmark', '--root', root],
      stdout: 'pipe',
      stderr: 'pipe',
    });

    expect(proc.exitCode).toBe(0);
    expect(proc.stdout.toString()).toContain('"cacheEntries"');
    expect(existsSync(path.join(fixturesRoot, 'github-actions', '.bun-platform'))).toBe(false);
  });

  test('committed fixtures stay free of platform state', () => {
    for (const fixture of ['github-actions', 'validate-success', 'mixed-lockfiles', 'safe-fixes', 'vercel-next', 'docker', 'monorepo']) {
      expect(existsSync(path.join(fixturesRoot, fixture, '.bun-platform'))).toBe(false);
    }
  });
});
