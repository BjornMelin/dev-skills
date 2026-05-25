#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';

const ansiPattern = /\u001b\[[0-9;]*m/g;
const expectedCliVersionPattern = /^1\.18\./;
const splitSkillNames = [
  'firecrawl-agent',
  'firecrawl-crawl',
  'firecrawl-download',
  'firecrawl-interact',
  'firecrawl-map',
  'firecrawl-parse',
  'firecrawl-scrape',
  'firecrawl-search',
];

function stripAnsi(value) {
  return String(value ?? '').replace(ansiPattern, '');
}

function run(args) {
  const result = spawnSync('firecrawl', args, {
    encoding: 'utf8',
    env: process.env,
  });
  return {
    command: ['firecrawl', ...args].join(' '),
    status: result.status,
    ok: result.status === 0,
    stdout: stripAnsi(result.stdout).trim(),
    stderr: stripAnsi(result.stderr).trim(),
    error: result.error ? result.error.message : null,
  };
}

function hasGitignoreEntry() {
  const path = resolve('.gitignore');
  if (!existsSync(path)) {
    return false;
  }
  const lines = readFileSync(path, 'utf8')
    .split(/\r?\n/)
    .map((line) => line.trim());
  return lines.includes('.firecrawl') || lines.includes('.firecrawl/');
}

function installedSplitSkills() {
  const skillsDir = join(homedir(), '.agents', 'skills');
  if (!existsSync(skillsDir)) {
    return [];
  }
  const installed = new Set(
    readdirSync(skillsDir, { withFileTypes: true })
      .filter((entry) => entry.isDirectory())
      .map((entry) => entry.name),
  );
  return splitSkillNames.filter((name) => installed.has(name));
}

function firstLine(value) {
  return String(value ?? '').split(/\r?\n/).find(Boolean) ?? '';
}

const commands = [
  ['search', '--help'],
  ['scrape', '--help'],
  ['map', '--help'],
  ['crawl', '--help'],
  ['agent', '--help'],
  ['interact', '--help'],
  ['parse', '--help'],
  ['monitor', '--help'],
  ['x', 'download', '--help'],
];

const version = run(['--version']);
const status = run(['--status']);
const helpChecks = commands.map((args) => run(args));
const splitSkillsInstalled = installedSplitSkills();
const commandAvailability = Object.fromEntries(
  helpChecks.map((check) => [check.command, check.ok]),
);
const gitignoreOk = hasGitignoreEntry();
const warnings = [
  version.ok && !expectedCliVersionPattern.test(firstLine(version.stdout))
    ? `CLI version ${firstLine(version.stdout)} differs from documented 1.18.x behavior; trust local help.`
    : null,
  commandAvailability['firecrawl x download --help'] === false
    ? '`firecrawl x download` is unavailable; site download docs may be stale.'
    : null,
  commandAvailability['firecrawl monitor --help'] === false
    ? '`firecrawl monitor` is unavailable; monitor docs may be stale or account-limited.'
    : null,
  existsSync(resolve('.gitignore')) && !gitignoreOk
    ? '.gitignore does not include .firecrawl; fetched content may be tracked accidentally.'
    : null,
  splitSkillsInstalled.length > 0
    ? `Split Firecrawl CLI skills still installed: ${splitSkillsInstalled.join(', ')}`
    : null,
].filter(Boolean);
const checks = [
  {
    name: 'firecrawl binary',
    required: true,
    ok: version.ok,
    detail: version.ok ? version.stdout : version.error || version.stderr,
  },
  {
    name: 'authenticated status',
    required: true,
    ok: status.ok,
    detail: status.ok ? 'firecrawl --status succeeded' : status.stderr || status.stdout,
  },
  {
    name: '.firecrawl output directory',
    required: false,
    ok: existsSync(resolve('.firecrawl')),
    detail: existsSync(resolve('.firecrawl')) ? 'present' : 'not present in current working directory',
  },
  {
    name: '.firecrawl gitignore',
    required: false,
    ok: gitignoreOk,
    detail: gitignoreOk ? 'ignored' : '.gitignore does not include .firecrawl',
  },
  ...helpChecks.map((check) => ({
    name: `${check.command} help`,
    required: true,
    ok: check.ok,
    detail: check.ok ? 'available' : check.stderr || check.stdout || check.error,
  })),
];

const report = {
  generatedAt: new Date().toISOString(),
  ok: checks.every((check) => !check.required || check.ok),
  version: version.stdout || null,
  status: status.ok ? status.stdout : null,
  drift: {
    expectedCliVersion: '1.18.x',
    commandAvailability,
    firecrawlGitignored: gitignoreOk,
    splitSkillsInstalled,
  },
  warnings,
  checks,
};

console.log(JSON.stringify(report, null, 2));
process.exit(report.ok ? 0 : 1);
