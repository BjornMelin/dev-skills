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
      options[key] = value;
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
    ...extractUrls(strings.join('\n')),
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
  if (['product', 'release', 'package'].includes(key)) return 24 * 60 * 60 * 1000;
  if (['docs', 'reference', 'api'].includes(key)) return 7 * 24 * 60 * 60 * 1000;
  return 24 * 60 * 60 * 1000;
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

function validateIndexedArtifact(record) {
  const artifactPath = record.artifactPath ? resolve(record.artifactPath) : null;
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
  if (record.sourceFileHash && record.lookupSourceFileHash) {
    // Parse records with matching source hashes are valid until the input changes.
    return { fresh: record.sourceFileHash === record.lookupSourceFileHash, expiresAt: null };
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
    if (record.normalizedUrls?.some((url) => withoutQuery(url) === noQuery)) {
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
  };

  const hits = [];
  for (const record of records) {
    const match = scoreRecord(record, lookupOptions);
    if (!match) continue;
    const artifactValidation = validateIndexedArtifact(record);
    if (!artifactValidation.ok) continue;
    const recordTtl = parseDuration(options.ttl) ?? ttlMsFor(options.intent ?? record.intent, record.commandType);
    const enriched = {
      ...record,
      lookupSourceFileHash,
    };
    const freshness = fileOnlyLookup && match.matchType !== 'file-hash'
      ? { fresh: false, expiresAt: null, reason: 'source-file-hash-required' }
      : withFreshness(enriched, recordTtl, now);
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
    command: options.command ?? null,
    commandType: commandTypeFromPath(artifact),
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
    const unrecordedSource = join(dir, 'unrecorded.pdf');
    writeFileSync(unrecordedSource, 'source changed without a recorded hash');
    writeFileSync(join(root, 'parse-unrecorded.md'), '# Unrecorded\n');
    process.chdir(dir);
    const index = join(root, 'index.jsonl');
    const records = scan(root, index);
    writeIndex(index, records);
    appendFileSync(index, '{not valid jsonl}\n');
    recordManual({
      artifact: join(root, 'parse-report.md'),
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
    const rescannedRecords = scan(root, index);
    writeIndex(index, rescannedRecords);
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
    if (urlResult.hits.length === 0 || !urlResult.hits[0].fresh) {
      throw new Error('URL lookup did not find a fresh hit');
    }
    if (fileResult.hits.length === 0 || fileResult.hits[0].matchType !== 'file-hash') {
      throw new Error('File lookup did not find a hash hit');
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
    return {
      ok: true,
      records: records.length,
      urlHit: urlResult.hits[0].artifactPath,
      fileHit: fileResult.hits[0].artifactPath,
      staleSlugHit: staleSlugHit.artifactPath,
      deletedHits: deletedResult.hits.length,
      overwrittenHits: overwrittenResult.hits.length,
      invalidUrlHits: invalidUrlResult.hits.length,
      oldRescanHits: oldRescanResult.hits.length,
      newRescanHit: newRescanResult.hits[0].artifactPath,
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
