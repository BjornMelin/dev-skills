#!/usr/bin/env bun
/**
 * Fetch and snapshot Vercel's Bun runtime docs into `references/`.
 *
 * Default:
 *   - URL: https://vercel.com/docs/functions/runtimes/bun
 *   - Out: references/ref-vercel-bun-runtime.md
 *
 * Run:
 *   bun ~/.agents/skills/bun-dev/scripts/update-vercel-bun-docs.ts
 *
 * Custom:
 *   bun ~/.agents/skills/bun-dev/scripts/update-vercel-bun-docs.ts --url <url> --base <base>
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

  const url = take("--url") ?? "https://vercel.com/docs/functions/runtimes/bun";
  const base = take("--base") ?? "ref-vercel-bun-runtime";
  return { url, base };
}

function stripYamlFrontmatter(md: string): string {
  if (!md.startsWith("---\n")) return md;
  const end = md.indexOf("\n---\n", 4);
  if (end === -1) return md;
  return md.slice(end + "\n---\n".length);
}

function tidyMarkdown(md: string): string {
  let s = md.replace(/\r/g, "");
  s = stripYamlFrontmatter(s);
  s = s.replace(/\n{3,}/g, "\n\n");
  s = s.trim();
  return `${s}\n`;
}

function decodeHtmlEntities(input: string): string {
  return input
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&nbsp;/g, " ");
}

function htmlToMarkdown(html: string): string {
  const mainMatch = html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/i);
  let content = mainMatch?.[1] ?? html;
  content = content.replace(/<script\b[^>]*>[\s\S]*?<\/script>/gi, "");
  content = content.replace(/<style\b[^>]*>[\s\S]*?<\/style>/gi, "");
  content = content.replace(/<svg\b[^>]*>[\s\S]*?<\/svg>/gi, "");
  content = content.replace(/<(h1|h2|h3|h4|h5|h6)\b[^>]*>([\s\S]*?)<\/\1>/gi, (_, tag, inner) => {
    const level = Number(tag.slice(1));
    return `\n${"#".repeat(level)} ${decodeHtmlEntities(inner.replace(/<[^>]+>/g, " ").trim())}\n`;
  });
  content = content.replace(/<li\b[^>]*>([\s\S]*?)<\/li>/gi, (_, inner) => `\n- ${decodeHtmlEntities(inner.replace(/<[^>]+>/g, " ").trim())}`);
  content = content.replace(/<(p|div|section|article|pre|table|tr)\b[^>]*>/gi, "\n");
  content = content.replace(/<\/(p|div|section|article|pre|table|tr)>/gi, "\n");
  content = content.replace(/<code\b[^>]*>([\s\S]*?)<\/code>/gi, (_, inner) => `\`${decodeHtmlEntities(inner)}\``);
  content = content.replace(/<a\b[^>]*href=["']([^"']+)["'][^>]*>([\s\S]*?)<\/a>/gi, (_, href, inner) => {
    const text = decodeHtmlEntities(inner.replace(/<[^>]+>/g, " ").trim());
    return text.length > 0 ? `[${text}](${href})` : href;
  });
  content = content.replace(/<[^>]+>/g, " ");
  content = decodeHtmlEntities(content);
  content = content.replace(/[ \t]+\n/g, "\n");
  content = content.replace(/\n{3,}/g, "\n\n");
  return tidyMarkdown(content);
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
      "user-agent": "bun-dev-skill/1.0 (+https://vercel.com)",
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

async function fetchHtml(url: string): Promise<string> {
  const res = await fetch(url, {
    headers: {
      "user-agent": "bun-dev-skill/1.0 (+https://vercel.com)",
      accept: "text/html,application/xhtml+xml",
    },
  });
  if (!res.ok) {
    throw new Error(`Failed to fetch HTML: ${url} (${res.status} ${res.statusText})`);
  }
  return res.text();
}

async function main() {
  const { url, base } = parseArgs(process.argv.slice(2));
  await mkdir(referencesDir, { recursive: true });

  const html = await fetchHtml(url);
  const mdUrl = await resolveMarkdownUrl(url).catch(() => undefined);
  let md: string;

  if (mdUrl) {
    const res = await fetch(mdUrl, {
      headers: {
        "user-agent": "bun-dev-skill/1.0 (+https://vercel.com)",
        accept: "text/markdown,text/plain;q=0.9,*/*;q=0.8",
      },
    });
    if (res.ok) {
      md = tidyMarkdown(await res.text());
    } else {
      md = htmlToMarkdown(html);
    }
  } else {
    md = htmlToMarkdown(html);
  }

  const outPath = join(referencesDir, `${base}.md`);

  // Clean up legacy formats (older versions of this script wrote html/txt).
  await rm(join(referencesDir, `${base}.html`), { force: true });
  await rm(join(referencesDir, `${base}.txt`), { force: true });

  await writeFile(outPath, md, "utf8");

  console.log(`Wrote:\n- ${outPath}`);
}

await main();
