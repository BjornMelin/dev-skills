#!/usr/bin/env bun
/**
 * Fetch and snapshot Bun release notes into `references/`.
 *
 * Default:
 *   - URL: https://bun.com/blog/release-notes/bun-v1.3.10
 *   - Out: references/ref-bun-release-notes-bun-v1.3.10.md
 *
 * Run:
 *   bun ~/.agents/skills/bun-dev/scripts/update-bun-release-notes.ts
 *
 * Custom:
 *   bun ~/.agents/skills/bun-dev/scripts/update-bun-release-notes.ts --url https://bun.com/blog/release-notes/bun-vX.Y.Z --base ref-bun-release-notes-bun-vX.Y.Z
 */

import { mkdir, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

type Args = Readonly<{
  url: string;
  base: string;
}>;

const __filename = fileURLToPath(import.meta.url);
const skillRoot = join(__filename, "..", "..");
const referencesDir = join(skillRoot, "references");

function parseArgs(argv: readonly string[]): Args {
  const args = [...argv];
  const take = (flag: string): string | undefined => {
    const idx = args.indexOf(flag);
    if (idx === -1) return undefined;
    const val = args[idx + 1];
    if (!val || val.startsWith("--")) return undefined;
    return val;
  };

  const url = take("--url") ?? "https://bun.com/blog/release-notes/bun-v1.3.10";
  const base = take("--base") ?? "ref-bun-release-notes-bun-v1.3.10";
  return { url, base };
}

function stripYamlFrontmatter(md: string): string {
  if (!md.startsWith("---\n")) return md;
  const end = md.indexOf("\n---\n", 4);
  if (end === -1) return md;
  return md.slice(end + "\n---\n".length);
}

function stripLiquidTags(md: string): string {
  const out: string[] = [];
  for (const line of md.split("\n")) {
    const t = line.trim();
    if (/^\{%\s*.*\s*%\}$/.test(t)) continue;
    out.push(line);
  }
  return out.join("\n");
}

function tidyMarkdown(md: string): string {
  let s = md.replace(/\r/g, "");
  s = stripYamlFrontmatter(s);
  s = stripLiquidTags(s);
  s = s.replace(/\n{3,}/g, "\n\n");
  s = s.trim();
  return `${s}\n`;
}

function extractMarkdownAlternateHref(html: string): string | undefined {
  const linkRe = /<link\b[^>]*\btype=["']text\/markdown["'][^>]*>/gi;
  for (const m of html.matchAll(linkRe)) {
    const tag = m[0];
    const hrefMatch = tag.match(/\bhref=["']([^"']+)["']/i);
    if (hrefMatch) return hrefMatch[1];
  }
  return undefined;
}

async function resolveMarkdownUrl(url: string): Promise<string> {
  if (url.endsWith(".md")) return url;

  const res = await fetch(url, {
    headers: {
      "user-agent": "bun-dev-skill/1.0 (+https://bun.com)",
      accept: "text/html,application/xhtml+xml",
    },
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch HTML: ${url} (${res.status} ${res.statusText})`);
  }

  const html = await res.text();
  const href = extractMarkdownAlternateHref(html);
  if (!href) {
    throw new Error(`Could not find <link rel=\"alternate\" type=\"text/markdown\"> in ${url}`);
  }

  return new URL(href, url).toString();
}

async function main() {
  const { url, base } = parseArgs(process.argv.slice(2));
  await mkdir(referencesDir, { recursive: true });

  const mdUrl = await resolveMarkdownUrl(url);
  const res = await fetch(mdUrl, {
    headers: {
      "user-agent": "bun-dev-skill/1.0 (+https://bun.com)",
      accept: "text/markdown,text/plain;q=0.9,*/*;q=0.8",
    },
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch markdown: ${mdUrl} (${res.status} ${res.statusText})`);
  }

  const raw = await res.text();
  const md = tidyMarkdown(raw);

  const outPath = join(referencesDir, `${base}.md`);

  // Clean up legacy formats (older versions of this script wrote html/txt).
  await rm(join(referencesDir, `${base}.html`), { force: true });
  await rm(join(referencesDir, `${base}.txt`), { force: true });

  await writeFile(outPath, md, "utf8");

  console.log(`Wrote:\n- ${outPath}`);
}

await main();
