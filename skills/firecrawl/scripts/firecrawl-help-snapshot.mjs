#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

const ansiPattern = /\u001b\[[0-9;]*m/g;
const defaultCommands = [
  [],
  ['search'],
  ['scrape'],
  ['map'],
  ['crawl'],
  ['agent'],
  ['interact'],
  ['parse'],
  ['monitor'],
  ['x', 'download'],
  ['search-feedback'],
  ['credit-usage'],
];

function stripAnsi(value) {
  return String(value ?? '').replace(ansiPattern, '');
}

function parseArgs(argv) {
  const options = {
    output: null,
    markdown: null,
    commands: defaultCommands,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--output') {
      options.output = argv[++index];
    } else if (arg === '--markdown') {
      options.markdown = argv[++index];
    } else if (arg === '--commands') {
      options.commands = argv[++index]
        .split(',')
        .map((entry) => entry.trim().split(/\s+/).filter(Boolean));
    } else if (arg === '--help') {
      console.log(`Usage: firecrawl-help-snapshot.mjs [--output FILE] [--markdown FILE] [--commands "scrape,search,x download"]`);
      process.exit(0);
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return options;
}

function runFirecrawl(args) {
  const result = spawnSync('firecrawl', [...args, '--help'], {
    encoding: 'utf8',
    env: process.env,
  });
  return {
    command: ['firecrawl', ...args, '--help'].join(' '),
    status: result.status,
    ok: result.status === 0,
    help: stripAnsi(result.stdout).trim(),
    stderr: stripAnsi(result.stderr).trim(),
    error: result.error ? result.error.message : null,
  };
}

function runVersion() {
  const result = spawnSync('firecrawl', ['--version'], {
    encoding: 'utf8',
    env: process.env,
  });
  return {
    status: result.status,
    value: stripAnsi(result.stdout).trim(),
    error: result.error ? result.error.message : stripAnsi(result.stderr).trim() || null,
  };
}

function write(path, contents) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
}

function toMarkdown(snapshot) {
  const sections = [
    `# Firecrawl CLI Help Snapshot`,
    '',
    `Generated: ${snapshot.generatedAt}`,
    `Version: ${snapshot.version.value || 'unknown'}`,
    '',
  ];
  for (const command of snapshot.commands) {
    sections.push(`## ${command.command}`, '', '```text', command.help || command.stderr || command.error || '', '```', '');
  }
  return `${sections.join('\n')}\n`;
}

try {
  const options = parseArgs(process.argv.slice(2));
  const snapshot = {
    generatedAt: new Date().toISOString(),
    version: runVersion(),
    commands: options.commands.map((args) => runFirecrawl(args)),
  };
  snapshot.ok = snapshot.version.status === 0 && snapshot.commands.every((command) => command.ok);

  const json = `${JSON.stringify(snapshot, null, 2)}\n`;
  if (options.output) {
    write(options.output, json);
    console.error(`wrote ${options.output}`);
  } else {
    process.stdout.write(json);
  }
  if (options.markdown) {
    write(options.markdown, toMarkdown(snapshot));
    console.error(`wrote ${options.markdown}`);
  }
  process.exit(snapshot.ok ? 0 : 1);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(2);
}
