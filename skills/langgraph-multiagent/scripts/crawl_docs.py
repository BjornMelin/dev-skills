#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
import time
import urllib.parse
import urllib.request
from dataclasses import dataclass
from html.parser import HTMLParser
from pathlib import Path


class _LinkExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.links: list[str] = []

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        if tag.lower() != "a":
            return
        for k, v in attrs:
            if k.lower() == "href" and v:
                self.links.append(v)


def _fetch(url: str, *, timeout_s: int = 30) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "langgraph-multiagent-skill/1.0"})
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return resp.read().decode("utf-8", errors="replace")


def _normalize_url(base: str, href: str) -> str | None:
    href = href.strip()
    if not href:
        return None
    if href.startswith("#"):
        return None
    if href.startswith(("mailto:", "tel:", "javascript:")):
        return None
    url = urllib.parse.urljoin(base, href)
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme not in {"http", "https"}:
        return None
    # Drop fragments
    parsed = parsed._replace(fragment="")
    return urllib.parse.urlunparse(parsed)


def _sha256(s: str) -> str:
    return hashlib.sha256(s.encode("utf-8")).hexdigest()


def _safe_filename(url: str) -> str:
    parsed = urllib.parse.urlparse(url)
    path = parsed.path.strip("/").replace("/", "__") or "index"
    if len(path) > 180:
        path = path[:180]
    return f"{parsed.netloc}__{path}__{_sha256(url)[:10]}.html"


def _extract_links(url: str, html: str) -> list[str]:
    parser = _LinkExtractor()
    try:
        parser.feed(html)
    except Exception:
        return []
    out: list[str] = []
    for href in parser.links:
        norm = _normalize_url(url, href)
        if norm:
            out.append(norm)
    return out


@dataclass(frozen=True)
class CrawlConfig:
    allow_domains: tuple[str, ...]
    allow_prefixes: tuple[str, ...]
    delay_s: float
    max_pages: int


def _allowed(url: str, cfg: CrawlConfig) -> bool:
    parsed = urllib.parse.urlparse(url)
    if parsed.netloc not in cfg.allow_domains:
        return False
    if not cfg.allow_prefixes:
        return True
    return any(url.startswith(p) for p in cfg.allow_prefixes)


LLMS_LINK_RE = re.compile(r"\((https?://[^)\\s]+)\)")


def _extract_llms_urls(llms_txt: str) -> list[str]:
    return [m.group(1) for m in LLMS_LINK_RE.finditer(llms_txt)]


def main() -> int:
    parser = argparse.ArgumentParser(description="Crawl docs (bounded) and save HTML snapshots for offline inspection.")
    parser.add_argument("--out-dir", default="docs_cache", help="Output directory for snapshots (default: docs_cache).")
    parser.add_argument(
        "--seeds",
        nargs="*",
        default=[],
        help="Seed URLs to crawl (space-separated). If omitted and --llms-txt is set, seeds come from llms.txt.",
    )
    parser.add_argument(
        "--llms-txt",
        default="",
        help="If set, fetch this llms.txt and use it to seed URLs (e.g. https://langchain-ai.github.io/langgraph/llms.txt).",
    )
    parser.add_argument(
        "--allow-domains",
        nargs="+",
        default=["langchain-ai.github.io", "docs.langchain.com"],
        help="Allowed domains for crawling (default: langchain-ai.github.io docs.langchain.com).",
    )
    parser.add_argument(
        "--allow-prefixes",
        nargs="*",
        default=[],
        help="Optional URL prefixes to constrain crawling further (recommended).",
    )
    parser.add_argument("--max-pages", type=int, default=200, help="Max pages to fetch (default: 200).")
    parser.add_argument("--delay-s", type=float, default=0.2, help="Delay between requests (default: 0.2s).")
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    seeds = list(args.seeds)
    if args.llms_txt:
        print(f"[info] Fetching llms.txt: {args.llms_txt}", file=sys.stderr)
        llms = _fetch(args.llms_txt)
        seeds.extend(_extract_llms_urls(llms))

    # Deduplicate while preserving order
    seen: set[str] = set()
    queue: list[str] = []
    for u in seeds:
        if u in seen:
            continue
        seen.add(u)
        queue.append(u)

    cfg = CrawlConfig(
        allow_domains=tuple(args.allow_domains),
        allow_prefixes=tuple(args.allow_prefixes),
        delay_s=max(args.delay_s, 0.0),
        max_pages=max(args.max_pages, 1),
    )

    index_path = out_dir / "index.jsonl"
    fetched: set[str] = set()
    pages = 0

    with index_path.open("a", encoding="utf-8") as index_f:
        while queue and pages < cfg.max_pages:
            url = queue.pop(0)
            if url in fetched:
                continue
            if not _allowed(url, cfg):
                continue

            try:
                html = _fetch(url)
            except Exception as e:
                index_f.write(json.dumps({"url": url, "error": str(e)}) + "\n")
                index_f.flush()
                continue

            filename = _safe_filename(url)
            (out_dir / filename).write_text(html, encoding="utf-8")
            index_f.write(json.dumps({"url": url, "file": filename, "sha256": _sha256(html)}) + "\n")
            index_f.flush()

            fetched.add(url)
            pages += 1

            for link in _extract_links(url, html):
                if link in seen:
                    continue
                seen.add(link)
                queue.append(link)

            if cfg.delay_s:
                time.sleep(cfg.delay_s)

    print(f"[done] Fetched {pages} pages into: {out_dir}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

