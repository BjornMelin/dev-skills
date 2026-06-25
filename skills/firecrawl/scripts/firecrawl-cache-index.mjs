#!/usr/bin/env node
import {
  appendFileSync,
  closeSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  readSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, extname, join, relative, resolve } from 'node:path';
import { createHash } from 'node:crypto';

const schemaVersion = 1;
const maxIndexedBytes = 2 * 1024 * 1024;
const trackingParams = new Set([
  'fbclid',
  'gclid',
  'mc_cid',
  'mc_eid',
]);

function usage() {
  return `Usage:
  firecrawl-cache-index.mjs scan [--root .firecrawl] [--out .firecrawl/index.jsonl] [--json]
  firecrawl-cache-index.mjs find (--url URL | --query TEXT | --file PATH) [--intent INTENT] [--root .firecrawl] [--index .firecrawl/index.jsonl] [--ttl 24h] [--refresh-index] [--json]
  firecrawl-cache-index.mjs record --artifact PATH [--url URL] [--query TEXT] [--source-file PATH] [--command TEXT] [--intent INTENT] [--index .firecrawl/index.jsonl] [--json]
  firecrawl-cache-index.mjs self-test

This script reads local .firecrawl artifacts only. It never calls Firecrawl.`;
}

function parseArgs(argv) {
  const [command, ...rest] = argv;
  const options = { command };
  for (let index = 0; index < rest.length; index += 1) {
    const arg = rest[index];
    if (arg === '--json') {
      options.json = true;
    } else if (arg === '--refresh-index') {
      options.refreshIndex = true;
    } else if (arg.startsWith('--')) {
      const key = arg.slice(2).replace(/-([a-z])/g, (_, char) => char.toUpperCase());
      const value = rest[index + 1];
      if (value === undefined || value.startsWith('--')) {
        throw new Error(`Missing value for ${arg}`);
      }
      if (key === 'command') {
        options.metadataCommand = value;
      } else {
        options[key] = value;
      }
      index += 1;
    } else {
      throw new Error(`Unknown argument: ${arg}`);
    }
  }
  return options;
}

function sha256(value) {
  return createHash('sha256').update(value).digest('hex');
}

function fileHash(path) {
  const hash = createHash('sha256');
  const fd = openSync(path, 'r');
  const buffer = Buffer.allocUnsafe(1024 * 1024);
  try {
    while (true) {
      const bytesRead = readSync(fd, buffer, 0, buffer.length, null);
      if (bytesRead === 0) break;
      hash.update(buffer.subarray(0, bytesRead));
    }
  } finally {
    closeSync(fd);
  }
  return hash.digest('hex');
}

function slugify(value) {
  return String(value ?? '')
    .toLowerCase()
    .replace(/https?:\/\//g, '')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 120);
}

function normalizeQuery(value) {
  return String(value ?? '').trim().replace(/\s+/g, ' ').toLowerCase();
}

function normalizeUrl(value) {
  try {
    const url = new URL(String(value).trim());
    url.protocol = url.protocol.toLowerCase();
    url.hostname = url.hostname.toLowerCase();
    url.hash = '';
    for (const key of [...url.searchParams.keys()]) {
      if (key.toLowerCase().startsWith('utm_') || trackingParams.has(key.toLowerCase())) {
        url.searchParams.delete(key);
      }
    }
    const sorted = [...url.searchParams.entries()].sort(([left], [right]) =>
      left.localeCompare(right),
    );
    url.search = '';
    for (const [key, paramValue] of sorted) {
      url.searchParams.append(key, paramValue);
    }
    if (url.pathname.length > 1) {
      url.pathname = url.pathname.replace(/\/+$/, '');
    }
    return url.toString();
  } catch {
    return null;
  }
}

function extractPrimaryUrls(parsed) {
  if (!parsed || typeof parsed !== 'object') return [];
  const candidates = [
    parsed.url,
    parsed.sourceUrl,
    parsed.metadata?.sourceURL,
    parsed.metadata?.sourceUrl,
    parsed.metadata?.url,
    parsed.data?.url,
    parsed.data?.sourceUrl,
    parsed.data?.metadata?.sourceURL,
    parsed.data?.metadata?.sourceUrl,
    parsed.data?.metadata?.url,
  ];
  return unique(candidates.filter((value) => typeof value === 'string'));
}

function withoutQuery(normalizedUrl) {
  if (!normalizedUrl) return null;
  try {
    const url = new URL(normalizedUrl);
    url.search = '';
    return url.toString();
  } catch {
    return normalizedUrl;
  }
}

function hasQuery(normalizedUrl) {
  if (!normalizedUrl) return false;
  try {
    return new URL(normalizedUrl).searchParams.toString() !== '';
  } catch {
    return false;
  }
}

function unique(values) {
  return [...new Set(values.filter(Boolean))];
}

function walkFiles(root) {
  if (!existsSync(root)) return [];
  const files = [];
  const stack = [root];
  while (stack.length > 0) {
    const current = stack.pop();
    for (const entry of readdirSync(current, { withFileTypes: true })) {
      if (entry.name === 'scratchpad' || entry.name === 'node_modules') continue;
      const fullPath = join(current, entry.name);
      if (entry.isDirectory()) {
        stack.push(fullPath);
      } else if (entry.isFile()) {
        files.push(fullPath);
      }
    }
  }
  return files.sort();
}

function readPrefix(path, size) {
  const bytesToRead = Math.min(size, maxIndexedBytes);
  if (bytesToRead <= 0) return '';
  const fd = openSync(path, 'r');
  const buffer = Buffer.allocUnsafe(bytesToRead);
  let offset = 0;
  try {
    while (offset < bytesToRead) {
      const bytesRead = readSync(fd, buffer, offset, bytesToRead - offset, offset);
      if (bytesRead === 0) break;
      offset += bytesRead;
    }
  } finally {
    closeSync(fd);
  }
  return buffer.subarray(0, offset).toString('utf8');
}

function collectStringValues(value, output = [], limit = 2000) {
  if (output.length >= limit) return output;
  if (typeof value === 'string') {
    output.push(value);
  } else if (Array.isArray(value)) {
    for (const item of value) collectStringValues(item, output, limit);
  } else if (value && typeof value === 'object') {
    for (const item of Object.values(value)) collectStringValues(item, output, limit);
  }
  return output;
}

function extractUrls(text) {
  const urls = [];
  const pattern = /https?:\/\/[^\s<>"')\]]+/g;
  for (const match of String(text).matchAll(pattern)) {
    urls.push(match[0].replace(/[.,;:!?]+$/g, ''));
  }
  return unique(urls);
}

function commandTypeFromPath(path) {
  const name = basename(path).toLowerCase();
  const parts = path.toLowerCase().split(/[\\/]/);
  const candidates = [
    'search',
    'scrape',
    'map',
    'crawl',
    'agent',
    'monitor',
    'parse',
    'interact',
    'download',
    'research',
    'feedback',
  ];
  for (const candidate of candidates) {
    if (name.startsWith(`${candidate}-`) || name.startsWith(`${candidate}.`)) {
      return candidate;
    }
  }
  if (parts.some((part) => part.includes('motion-docs') || part.includes('agent-skills'))) {
    return 'download';
  }
  return 'artifact';
}

function formatFromExtension(path) {
  const extension = extname(path).slice(1).toLowerCase();
  if (extension === 'md') return 'markdown';
  if (extension === 'json') return 'json';
  if (extension === 'txt') return 'text';
  if (extension === 'html' || extension === 'htm') return 'html';
  if (extension === 'png' || extension === 'jpg' || extension === 'jpeg' || extension === 'webp') {
    return 'image';
  }
  return extension || 'unknown';
}

function loadExistingByArtifact(indexPath) {
  const byArtifact = new Map();
  if (!existsSync(indexPath)) return byArtifact;
  for (const line of readFileSync(indexPath, 'utf8').split(/\r?\n/)) {
    if (!line.trim()) continue;
    try {
      const record = JSON.parse(line);
      if (record.artifactPath) byArtifact.set(record.artifactPath, record);
    } catch {
      // Ignore malformed historical lines; scan will rebuild usable metadata.
    }
  }
  return byArtifact;
}

function buildRecord(path, root, existing = null) {
  const stat = statSync(path);
  const relPath = relative(process.cwd(), path).replace(/\\/g, '/');
  const rootRelPath = relative(root, path).replace(/\\/g, '/');
  const format = formatFromExtension(path);
  const artifactSha256 = fileHash(path);
  const reuseExistingMetadata = Boolean(
    existing && (!existing.artifactSha256 || existing.artifactSha256 === artifactSha256),
  );
  let parsed = null;
  let strings = [];
  let firecrawlId = null;
  let creditsUsed = null;
  let truncatedForIndex = false;

  if (format === 'json' || format === 'markdown' || format === 'text' || format === 'html') {
    const text = readPrefix(path, stat.size);
    truncatedForIndex = stat.size > maxIndexedBytes;
    if (format === 'json') {
      try {
        parsed = JSON.parse(text);
        strings = collectStringValues(parsed);
        if (typeof parsed.id === 'string') firecrawlId = parsed.id;
        if (typeof parsed.creditsUsed === 'number') creditsUsed = parsed.creditsUsed;
      } catch {
        strings = [text];
      }
    } else {
      strings = [text];
    }
  }

  const sourceUrls = unique([
    ...(reuseExistingMetadata ? (existing?.sourceUrls ?? []) : []),
    ...(format === 'json' ? extractPrimaryUrls(parsed) : []),
  ]);
  const normalizedUrls = unique(sourceUrls.map(normalizeUrl));
  const query = reuseExistingMetadata ? (existing?.query ?? null) : null;
  const normalizedQuery = query ? normalizeQuery(query) : null;
  const sourceFile = reuseExistingMetadata ? (existing?.sourceFile ?? null) : null;
  const sourceFileHash = reuseExistingMetadata ? (existing?.sourceFileHash ?? null) : null;
  const command = reuseExistingMetadata ? (existing?.command ?? null) : null;
  const commandType = reuseExistingMetadata
    ? (existing?.commandType ?? commandTypeFromPath(path))
    : commandTypeFromPath(path);

  return {
    schemaVersion,
    indexedAt: new Date().toISOString(),
    artifactPath: relPath,
    rootRelativePath: rootRelPath,
    artifactMtime: stat.mtime.toISOString(),
    artifactSizeBytes: stat.size,
    artifactSha256,
    commandType,
    command,
    formats: [format],
    firecrawlId,
    creditsUsed,
    sourceUrls,
    normalizedUrls,
    urlHashes: normalizedUrls.map((url) => sha256(url)),
    query,
    normalizedQuery,
    queryHash: normalizedQuery ? sha256(normalizedQuery) : null,
    querySlug: normalizedQuery ? slugify(normalizedQuery) : null,
    sourceFile,
    sourceFileHash,
    intent: reuseExistingMetadata
      ? (existing?.intent ?? inferIntent(commandType, sourceUrls, relPath))
      : inferIntent(commandType, sourceUrls, relPath),
    truncatedForIndex,
  };
}

function inferIntent(commandType, urls, path) {
  const haystack = `${commandType} ${path} ${urls.join(' ')}`.toLowerCase();
  if (commandType === 'search') return 'search';
  if (commandType === 'parse') return 'parse';
  if (haystack.includes('pricing')) return 'pricing';
  if (haystack.includes('changelog') || haystack.includes('release')) return 'changelog';
  if (haystack.includes('status')) return 'status';
  if (haystack.includes('docs') || haystack.includes('reference') || haystack.includes('api')) {
    return 'docs';
  }
  if (commandType === 'crawl' || commandType === 'download') return 'docs';
  return 'generic';
}

function ttlMsFor(intent, commandType) {
  const key = String(intent || commandType || 'generic').toLowerCase();
  if (['current', 'latest', 'news', 'pricing', 'status', 'changelog'].includes(key)) {
    return 60 * 60 * 1000;
  }
  if (key === 'search') return 6 * 60 * 60 * 1000;
  if (key === 'parse') return Number.POSITIVE_INFINITY;
  if (key === 'monitor') return 0;
  if (['product', 'release', 'package'].includes(key)) return 24 * 60 * 60 * 1000;
  if (['docs', 'reference', 'api'].includes(key)) return 7 * 24 * 60 * 60 * 1000;
  return 24 * 60 * 60 * 1000;
}

function ttlMsForRecord(options, record) {
  const explicitTtl = parseDuration(options.ttl);
  if (explicitTtl !== null) return explicitTtl;
  return Math.min(
    ttlMsFor(options.intent, null),
    ttlMsFor(record.intent, record.commandType),
  );
}

function parseDuration(value) {
  if (!value) return null;
  const match = String(value).trim().match(/^(\d+(?:\.\d+)?)(ms|s|m|h|d|w)$/);
  if (!match) throw new Error(`Invalid duration: ${value}`);
  const amount = Number(match[1]);
  const unit = match[2];
  const multipliers = {
    ms: 1,
    s: 1000,
    m: 60 * 1000,
    h: 60 * 60 * 1000,
    d: 24 * 60 * 60 * 1000,
    w: 7 * 24 * 60 * 60 * 1000,
  };
  return amount * multipliers[unit];
}

function scan(root, indexPath) {
  const resolvedRoot = resolve(root);
  const existing = loadExistingByArtifact(indexPath);
  return walkFiles(resolvedRoot)
    .filter((path) => basename(path) !== basename(indexPath))
    .filter((path) => basename(path) !== 'index.jsonl')
    .filter((path) => !path.endsWith('.evidence.txt'))
    .map((path) => {
      const relPath = relative(process.cwd(), path).replace(/\\/g, '/');
      return buildRecord(path, resolvedRoot, existing.get(relPath));
    });
}

function writeIndex(indexPath, records) {
  mkdirSync(dirname(indexPath), { recursive: true });
  writeFileSync(indexPath, `${records.map((record) => JSON.stringify(record)).join('\n')}\n`);
}

function loadRecords(root, indexPath, refreshIndex) {
  if (refreshIndex || !existsSync(indexPath)) {
    const records = scan(root, indexPath);
    if (indexPath) writeIndex(indexPath, records);
    return records;
  }
  const records = [];
  for (const line of readFileSync(indexPath, 'utf8').split(/\r?\n/)) {
    if (!line.trim()) continue;
    try {
      records.push(JSON.parse(line));
    } catch {
      // Ignore malformed lines; callers can refresh-index to rebuild.
    }
  }
  return records;
}

function normalizeCommand(value) {
  return String(value ?? '').trim().replace(/\s+/g, ' ');
}

function validateIndexedArtifact(record, root) {
  const artifactPath = record.rootRelativePath
    ? resolve(root, record.rootRelativePath)
    : record.artifactPath
      ? resolve(record.artifactPath)
      : null;
  if (!artifactPath || !existsSync(artifactPath)) {
    return { ok: false, reason: 'artifact-missing' };
  }
  const stat = statSync(artifactPath);
  if (typeof record.artifactSizeBytes === 'number' && record.artifactSizeBytes !== stat.size) {
    return { ok: false, reason: 'artifact-size-changed' };
  }
  if (record.artifactSha256 && record.artifactSha256 !== fileHash(artifactPath)) {
    return { ok: false, reason: 'artifact-hash-changed' };
  }
  return { ok: true, reason: null };
}

function withFreshness(record, ttlMs, now) {
  if (record.commandType === 'monitor' || record.intent === 'monitor') {
    return { fresh: false, expiresAt: null, reason: 'monitor-historical' };
  }
  if (record.commandType === 'parse' && record.sourceFileHash) {
    if (!record.command || !record.lookupCommand) {
      return { fresh: false, expiresAt: null, reason: 'parse-command-required' };
    }
  }
  if (record.sourceFileHash && record.lookupSourceFileHash) {
    if (record.sourceFileHash !== record.lookupSourceFileHash) {
      return { fresh: false, expiresAt: null, reason: 'source-file-hash-changed' };
    }
    if (record.command || record.lookupCommand) {
      if (!record.command || !record.lookupCommand) {
        return { fresh: false, expiresAt: null, reason: 'parse-command-required' };
      }
      if (normalizeCommand(record.command) !== normalizeCommand(record.lookupCommand)) {
        return { fresh: false, expiresAt: null, reason: 'parse-command-changed' };
      }
    }
    return { fresh: true, expiresAt: null };
  }
  if (ttlMs === Number.POSITIVE_INFINITY) return { fresh: true, expiresAt: null };
  const mtime = new Date(record.artifactMtime).getTime();
  const expiresAtMs = mtime + ttlMs;
  return {
    fresh: expiresAtMs >= now,
    expiresAt: new Date(expiresAtMs).toISOString(),
  };
}

function scoreRecord(record, options) {
  const artifactSlug = slugify(record.artifactPath);
  if (options.url) {
    const normalized = normalizeUrl(options.url);
    if (!normalized) return null;
    const noQuery = withoutQuery(normalized);
    if (record.normalizedUrls?.includes(normalized)) {
      return { score: 100, matchType: 'url-exact' };
    }
    if (
      !hasQuery(normalized)
      && record.normalizedUrls?.some((url) => !hasQuery(url) && withoutQuery(url) === noQuery)
    ) {
      return { score: 86, matchType: 'url-path' };
    }
    if (artifactSlug.includes(slugify(normalized))) {
      return { score: 62, matchType: 'artifact-slug' };
    }
  }
  if (options.query) {
    const normalizedQuery = normalizeQuery(options.query);
    const queryHash = sha256(normalizedQuery);
    const querySlug = slugify(normalizedQuery);
    if (record.queryHash === queryHash) {
      return { score: 100, matchType: 'query-exact' };
    }
    if (record.querySlug && querySlug.includes(record.querySlug)) {
      return { score: 76, matchType: 'query-slug' };
    }
    if (record.commandType === 'search' && artifactSlug.includes(querySlug)) {
      return { score: 68, matchType: 'artifact-slug' };
    }
  }
  if (options.file) {
    const sourcePath = options.resolvedLookupFile ?? resolve(options.file);
    const sourceSlug = options.lookupFileSlug ?? slugify(basename(sourcePath, extname(sourcePath)));
    const sourceHash = options.lookupSourceFileHash ?? null;
    if (record.sourceFileHash && sourceHash && record.sourceFileHash === sourceHash) {
      return { score: 100, matchType: 'file-hash' };
    }
    if (record.commandType === 'parse' && artifactSlug.includes(sourceSlug)) {
      return { score: 72, matchType: 'file-slug' };
    }
  }
  return null;
}

function findMatches(options) {
  const root = resolve(options.root ?? '.firecrawl');
  const indexPath = resolve(options.index ?? join(root, 'index.jsonl'));
  const records = loadRecords(root, indexPath, options.refreshIndex);
  const ttlMs = parseDuration(options.ttl) ?? ttlMsFor(options.intent, null);
  const now = Date.now();
  const resolvedLookupFile = options.file ? resolve(options.file) : null;
  const lookupSourceFileHash = resolvedLookupFile && existsSync(resolvedLookupFile)
    ? fileHash(resolvedLookupFile)
    : null;
  const lookupFileSlug = resolvedLookupFile
    ? slugify(basename(resolvedLookupFile, extname(resolvedLookupFile)))
    : null;
  const fileOnlyLookup = Boolean(options.file && !options.url && !options.query);
  const lookupOptions = {
    ...options,
    resolvedLookupFile,
    lookupSourceFileHash,
    lookupFileSlug,
    lookupCommand: options.metadataCommand ?? null,
  };

  const hits = [];
  for (const record of records) {
    const match = scoreRecord(record, lookupOptions);
    if (!match) continue;
    const artifactValidation = validateIndexedArtifact(record, root);
    if (!artifactValidation.ok) continue;
    const recordTtl = ttlMsForRecord(options, record);
    const enriched = {
      ...record,
      lookupSourceFileHash,
      lookupCommand: lookupOptions.lookupCommand,
    };
    let freshness = withFreshness(enriched, recordTtl, now);
    if (fileOnlyLookup && match.matchType !== 'file-hash') {
      freshness = { fresh: false, expiresAt: null, reason: 'source-file-hash-required' };
    } else if (['artifact-slug', 'query-slug', 'url-path', 'file-slug'].includes(match.matchType)) {
      freshness = { ...freshness, fresh: false, reason: 'fuzzy-match-refresh-required' };
    }
    const ageMs = now - new Date(record.artifactMtime).getTime();
    hits.push({
      artifactPath: record.artifactPath,
      commandType: record.commandType,
      intent: record.intent,
      matchType: match.matchType,
      score: match.score,
      fresh: freshness.fresh,
      freshnessReason: freshness.reason ?? null,
      ageMs,
      expiresAt: freshness.expiresAt,
      sourceUrls: record.sourceUrls?.slice(0, 20) ?? [],
      firecrawlId: record.firecrawlId,
      formats: record.formats,
    });
  }
  hits.sort((left, right) => {
    if (left.fresh !== right.fresh) return left.fresh ? -1 : 1;
    if (left.score !== right.score) return right.score - left.score;
    return left.ageMs - right.ageMs;
  });
  return {
    ok: true,
    generatedAt: new Date().toISOString(),
    indexPath: relative(process.cwd(), indexPath).replace(/\\/g, '/'),
    ttlMs,
    lookup: {
      url: options.url ?? null,
      query: options.query ?? null,
      file: options.file ?? null,
      intent: options.intent ?? null,
    },
    hits,
  };
}

function recordManual(options) {
  if (!options.artifact) throw new Error('record requires --artifact PATH');
  const artifact = resolve(options.artifact);
  if (!existsSync(artifact)) throw new Error(`Artifact not found: ${options.artifact}`);
  const indexPath = resolve(options.index ?? '.firecrawl/index.jsonl');
  const sourceFile = options.sourceFile ? resolve(options.sourceFile) : null;
  const existing = {
    command: options.metadataCommand ?? null,
    commandType: options.intent === 'parse' && sourceFile ? 'parse' : commandTypeFromPath(artifact),
    sourceUrls: options.url ? [options.url] : [],
    query: options.query ?? null,
    sourceFile: sourceFile ? relative(process.cwd(), sourceFile).replace(/\\/g, '/') : null,
    sourceFileHash: sourceFile && existsSync(sourceFile) ? fileHash(sourceFile) : null,
    intent: options.intent ?? null,
  };
  const root = resolve(dirname(indexPath));
  const record = buildRecord(artifact, root, existing);
  mkdirSync(dirname(indexPath), { recursive: true });
  appendFileSync(indexPath, `${JSON.stringify(record)}\n`);
  return { ok: true, indexPath: relative(process.cwd(), indexPath), record };
}

function printResult(result, json) {
  if (json) {
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  if (Array.isArray(result.records)) {
    console.log(`Indexed ${result.records.length} artifacts into ${result.indexPath}`);
    return;
  }
  if (Array.isArray(result.hits)) {
    if (result.hits.length === 0) {
      console.log('No matching .firecrawl artifacts found.');
      return;
    }
    for (const hit of result.hits.slice(0, 10)) {
      const state = hit.fresh ? 'fresh' : 'stale';
      console.log(`${state}\t${hit.score}\t${hit.matchType}\t${hit.artifactPath}`);
    }
    return;
  }
  console.log(JSON.stringify(result, null, 2));
}

function selfTest() {
  const dir = mkdtempSync(join(tmpdir(), 'firecrawl-cache-index-'));
  const oldCwd = process.cwd();
  try {
    const root = join(dir, '.firecrawl');
    mkdirSync(root, { recursive: true });
    writeFileSync(
      join(root, 'search-firecrawl-parse.json'),
      JSON.stringify({
        success: true,
        id: 'search_123',
        creditsUsed: 2,
        data: {
          web: [
            {
              title: 'Parse',
              url: 'https://docs.firecrawl.dev/features/parse?utm_source=test',
              markdown: 'Parse docs',
            },
            {
              title: 'Linked only',
              url: 'https://docs.firecrawl.dev/features/linked-only',
              markdown: 'Linked result only',
            },
          ],
        },
      }),
    );
    writeFileSync(
      join(root, 'scrape-firecrawl-parse.md'),
      '# Parse\nhttps://docs.firecrawl.dev/features/parse\n',
    );
    const source = join(dir, 'report.pdf');
    writeFileSync(source, 'fake-pdf');
    writeFileSync(join(root, 'parse-report.md'), '# Report\n');
    writeFileSync(join(root, 'report.md'), '# Generic report\n');
    writeFileSync(join(root, 'monitor-page.json'), '{"url":"https://example.com/status"}\n');
    writeFileSync(join(root, 'pricing-page.md'), '# Pricing\n');
    writeFileSync(join(root, 'docs-firecrawl-dev-features-artifact.md'), '# Artifact slug only\n');
    const unrecordedSource = join(dir, 'unrecorded.pdf');
    writeFileSync(unrecordedSource, 'source changed without a recorded hash');
    writeFileSync(join(root, 'parse-unrecorded.md'), '# Unrecorded\n');
    process.chdir(dir);
    const index = join(root, 'index.jsonl');
    const records = scan(root, index);
    writeIndex(index, records);
    appendFileSync(index, '{not valid jsonl}\n');
    recordManual({
      artifact: join(root, 'scrape-firecrawl-parse.md'),
      url: 'https://docs.firecrawl.dev/features/parse',
      index,
      metadataCommand: 'firecrawl scrape https://docs.firecrawl.dev/features/parse',
      intent: 'docs',
    });
    recordManual({
      artifact: join(root, 'search-firecrawl-parse.json'),
      query: 'firecrawl parse docs',
      index,
      metadataCommand: 'firecrawl search "firecrawl parse docs"',
      intent: 'search',
    });
    const malformedJsonlResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/features/parse',
      intent: 'docs',
    });
    recordManual({
      artifact: join(root, 'parse-report.md'),
      sourceFile: source,
      index,
      metadataCommand: 'firecrawl parse report.pdf',
      intent: 'parse',
    });
    recordManual({
      artifact: join(root, 'report.md'),
      sourceFile: source,
      index,
      intent: 'parse',
    });
    writeFileSync(
      join(root, 'stale-deleted.md'),
      '# Deleted\nhttps://docs.firecrawl.dev/deleted\n',
    );
    writeFileSync(
      join(root, 'stale-overwritten.md'),
      '# Original\nhttps://docs.firecrawl.dev/overwritten\n',
    );
    writeFileSync(
      join(root, 'rescan-reused.md'),
      '# Old\nhttps://docs.firecrawl.dev/old-rescan\n',
    );
    const staleRecords = scan(root, index);
    writeIndex(index, staleRecords);
    appendFileSync(index, '{not valid jsonl}\n');
    rmSync(join(root, 'stale-deleted.md'));
    writeFileSync(join(root, 'stale-overwritten.md'), '# Changed\n');
    writeFileSync(
      join(root, 'rescan-reused.md'),
      '# New\nhttps://docs.firecrawl.dev/new-rescan\n',
    );
    writeFileSync(
      join(root, 'query-old.md'),
      '# Search old\nhttps://example.com/search?q=old\n',
    );
    const rescannedRecords = scan(root, index);
    writeIndex(index, rescannedRecords);
    recordManual({
      artifact: join(root, 'rescan-reused.md'),
      url: 'https://docs.firecrawl.dev/new-rescan',
      index,
      metadataCommand: 'firecrawl scrape https://docs.firecrawl.dev/new-rescan',
      intent: 'docs',
    });
    recordManual({
      artifact: join(root, 'query-old.md'),
      url: 'https://example.com/search?q=old',
      index,
      metadataCommand: 'firecrawl scrape "https://example.com/search?q=old"',
      intent: 'docs',
    });
    recordManual({
      artifact: join(root, 'monitor-page.json'),
      url: 'https://example.com/status',
      index,
      metadataCommand: 'firecrawl monitor check monitor-123 check-456',
      intent: 'monitor',
    });
    recordManual({
      artifact: join(root, 'pricing-page.md'),
      url: 'https://example.com/pricing',
      index,
      metadataCommand: 'firecrawl scrape https://example.com/pricing',
      intent: 'pricing',
    });
    const oldPricingTime = new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString();
    const pricedRecords = loadRecords(root, index, false).map((record) =>
      record.artifactPath?.endsWith('pricing-page.md')
        ? { ...record, artifactMtime: oldPricingTime }
        : record,
    );
    writeIndex(index, pricedRecords);
    process.chdir(oldCwd);
    const urlResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/features/parse',
      intent: 'docs',
    });
    const fileResult = findMatches({
      root,
      index,
      file: source,
      intent: 'parse',
      metadataCommand: 'firecrawl parse report.pdf',
    });
    const fileMissingCommandResult = findMatches({
      root,
      index,
      file: source,
      intent: 'parse',
    });
    const fileDifferentCommandResult = findMatches({
      root,
      index,
      file: source,
      intent: 'parse',
      metadataCommand: 'firecrawl parse report.pdf -Q summary',
    });
    const slugResult = findMatches({
      root,
      index,
      file: unrecordedSource,
      intent: 'parse',
    });
    const deletedResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/deleted',
      intent: 'docs',
    });
    const overwrittenResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/overwritten',
      intent: 'docs',
    });
    const invalidUrlResult = findMatches({
      root,
      index,
      url: 'docs.firecrawl.dev/features/parse',
      intent: 'docs',
    });
    const oldRescanResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/old-rescan',
      intent: 'docs',
    });
    const newRescanResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/new-rescan',
      intent: 'docs',
    });
    const queryExactResult = findMatches({
      root,
      index,
      url: 'https://example.com/search?q=old',
      intent: 'docs',
    });
    const queryChangedResult = findMatches({
      root,
      index,
      url: 'https://example.com/search?q=new',
      intent: 'docs',
    });
    const querylessResult = findMatches({
      root,
      index,
      url: 'https://example.com/search',
      intent: 'docs',
    });
    const linkedOnlyResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/features/linked-only',
      intent: 'docs',
    });
    const monitorResult = findMatches({
      root,
      index,
      url: 'https://example.com/status',
      intent: 'monitor',
    });
    const stalePricingResult = findMatches({
      root,
      index,
      url: 'https://example.com/pricing',
      intent: 'docs',
    });
    const fuzzyQueryResult = findMatches({
      root,
      index,
      query: 'firecrawl parse docs latest release',
      intent: 'search',
    });
    const fuzzyArtifactResult = findMatches({
      root,
      index,
      url: 'https://docs.firecrawl.dev/features/artifact',
      intent: 'docs',
    });
    if (urlResult.hits.length === 0 || !urlResult.hits[0].fresh) {
      throw new Error('URL lookup did not find a fresh hit');
    }
    if (malformedJsonlResult.hits.length === 0 || !malformedJsonlResult.hits[0].fresh) {
      throw new Error('Malformed JSONL index lookup did not find a fresh hit');
    }
    if (fileResult.hits.length === 0 || fileResult.hits[0].matchType !== 'file-hash') {
      throw new Error('File lookup did not find a hash hit');
    }
    if (!fileResult.hits[0].fresh) {
      throw new Error('File lookup with matching parse command did not produce a fresh hit');
    }
    if (
      fileMissingCommandResult.hits.length === 0
      || fileMissingCommandResult.hits[0].fresh
      || fileMissingCommandResult.hits[0].freshnessReason !== 'parse-command-required'
    ) {
      throw new Error('File lookup without required parse command must not be fresh');
    }
    if (
      fileDifferentCommandResult.hits.length === 0
      || fileDifferentCommandResult.hits[0].fresh
      || fileDifferentCommandResult.hits[0].freshnessReason !== 'parse-command-changed'
    ) {
      throw new Error('File lookup with a different parse command must not be fresh');
    }
    const staleSlugHit = slugResult.hits.find((hit) => hit.matchType === 'file-slug');
    if (!staleSlugHit || staleSlugHit.fresh) {
      throw new Error('Unrecorded parse artifact must not be treated as fresh');
    }
    if (deletedResult.hits.length !== 0) {
      throw new Error('Deleted indexed artifact must not be returned');
    }
    if (overwrittenResult.hits.length !== 0) {
      throw new Error('Overwritten indexed artifact must not be returned');
    }
    if (invalidUrlResult.hits.length !== 0) {
      throw new Error('Invalid URL lookup must not match artifact slugs');
    }
    if (oldRescanResult.hits.length !== 0) {
      throw new Error('Refreshed scan must not carry stale URLs from reused artifact paths');
    }
    if (newRescanResult.hits.length === 0 || !newRescanResult.hits[0].fresh) {
      throw new Error('Refreshed scan did not index new reused artifact content');
    }
    if (queryExactResult.hits.length === 0 || queryExactResult.hits[0].matchType !== 'url-exact') {
      throw new Error('Query-bearing URL lookup did not find an exact hit');
    }
    if (queryChangedResult.hits.length !== 0) {
      throw new Error('Different query parameters must not match by URL path');
    }
    if (querylessResult.hits.length !== 0) {
      throw new Error('Queryless URL lookup must not match query-bearing cached URLs by path');
    }
    if (linkedOnlyResult.hits.some((hit) => hit.matchType === 'url-exact' || hit.matchType === 'url-path')) {
      throw new Error('Linked result URLs must not be treated as source URL hits');
    }
    if (
      monitorResult.hits.length === 0
      || monitorResult.hits[0].fresh
      || monitorResult.hits[0].freshnessReason !== 'monitor-historical'
    ) {
      throw new Error('Monitor artifacts must be historical, not fresh cache hits');
    }
    if (stalePricingResult.hits.length === 0 || stalePricingResult.hits[0].fresh) {
      throw new Error('Time-sensitive record intent must keep the stricter freshness window');
    }
    if (
      fuzzyQueryResult.hits.length === 0
      || fuzzyQueryResult.hits[0].fresh
      || fuzzyQueryResult.hits[0].freshnessReason !== 'fuzzy-match-refresh-required'
    ) {
      throw new Error('Fuzzy query matches must not be treated as fresh');
    }
    if (
      fuzzyArtifactResult.hits.length === 0
      || fuzzyArtifactResult.hits[0].fresh
      || fuzzyArtifactResult.hits[0].freshnessReason !== 'fuzzy-match-refresh-required'
    ) {
      throw new Error('Fuzzy artifact matches must not be treated as fresh');
    }
    return {
      ok: true,
      records: records.length,
      urlHit: urlResult.hits[0].artifactPath,
      malformedJsonlHit: malformedJsonlResult.hits[0].artifactPath,
      fileHit: fileResult.hits[0].artifactPath,
      fileMissingCommandFreshnessReason: fileMissingCommandResult.hits[0].freshnessReason,
      fileDifferentCommandFreshnessReason: fileDifferentCommandResult.hits[0].freshnessReason,
      staleSlugHit: staleSlugHit.artifactPath,
      deletedHits: deletedResult.hits.length,
      overwrittenHits: overwrittenResult.hits.length,
      invalidUrlHits: invalidUrlResult.hits.length,
      oldRescanHits: oldRescanResult.hits.length,
      newRescanHit: newRescanResult.hits[0].artifactPath,
      queryExactHit: queryExactResult.hits[0].artifactPath,
      queryChangedHits: queryChangedResult.hits.length,
      querylessHits: querylessResult.hits.length,
      linkedOnlyUrlHits: linkedOnlyResult.hits.length,
      monitorFreshnessReason: monitorResult.hits[0].freshnessReason,
      stalePricingFresh: stalePricingResult.hits[0].fresh,
      fuzzyQueryFreshnessReason: fuzzyQueryResult.hits[0].freshnessReason,
      fuzzyArtifactFreshnessReason: fuzzyArtifactResult.hits[0].freshnessReason,
    };
  } finally {
    process.chdir(oldCwd);
    rmSync(dir, { recursive: true, force: true });
  }
}

try {
  const options = parseArgs(process.argv.slice(2));
  if (!options.command || options.command === '--help' || options.command === 'help') {
    console.log(usage());
    process.exit(0);
  }
  if (options.command === 'scan') {
    const root = resolve(options.root ?? '.firecrawl');
    const indexPath = resolve(options.out ?? join(root, 'index.jsonl'));
    const records = scan(root, indexPath);
    writeIndex(indexPath, records);
    printResult(
      {
        ok: true,
        generatedAt: new Date().toISOString(),
        indexPath: relative(process.cwd(), indexPath).replace(/\\/g, '/'),
        records,
      },
      options.json,
    );
  } else if (options.command === 'find') {
    if (!options.url && !options.query && !options.file) {
      throw new Error('find requires one of --url, --query, or --file');
    }
    printResult(findMatches(options), options.json);
  } else if (options.command === 'record') {
    printResult(recordManual(options), options.json);
  } else if (options.command === 'self-test') {
    printResult(selfTest(), true);
  } else {
    throw new Error(`Unknown command: ${options.command}`);
  }
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(2);
}
