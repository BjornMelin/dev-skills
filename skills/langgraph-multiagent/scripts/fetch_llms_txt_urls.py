#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
import urllib.request
from pathlib import Path


LINK_RE = re.compile(r"\((https?://[^)\\s]+)\)")


def fetch(url: str, *, timeout_s: int = 20) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "langgraph-multiagent-skill/1.0"})
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return resp.read().decode("utf-8", errors="replace")


def extract_urls(text: str) -> list[str]:
    return [m.group(1) for m in LINK_RE.finditer(text)]


def main() -> int:
    parser = argparse.ArgumentParser(description="Fetch LangGraph llms.txt and extract all documentation URLs.")
    parser.add_argument(
        "--url",
        default="https://langchain-ai.github.io/langgraph/llms.txt",
        help="llms.txt URL to fetch (default: LangGraph llms.txt).",
    )
    parser.add_argument("--out", default=None, help="Write URLs to a file (one per line).")
    parser.add_argument("--print", dest="do_print", action="store_true", help="Print URLs to stdout.")
    parser.add_argument("--unique", action="store_true", help="Deduplicate URLs while preserving order.")
    args = parser.parse_args()

    raw = fetch(args.url)
    urls = extract_urls(raw)
    if args.unique:
        seen: set[str] = set()
        deduped: list[str] = []
        for u in urls:
            if u in seen:
                continue
            seen.add(u)
            deduped.append(u)
        urls = deduped

    if args.out:
        out_path = Path(args.out)
        out_path.write_text("\n".join(urls) + "\n", encoding="utf-8")
        print(f"Wrote: {out_path}", file=sys.stderr)

    if args.do_print or not args.out:
        for u in urls:
            print(u)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

