from __future__ import annotations

import argparse
import re
import time
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


DOCS_ROOT = "https://docs.streamlit.io"
LLMS_TXT_URL = f"{DOCS_ROOT}/llms.txt"


_MD_LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def _fetch_text(url: str, *, timeout_s: float = 30.0) -> str:
    req = urllib.request.Request(url, headers={"User-Agent": "streamlit-master-architect/0.1"})
    with urllib.request.urlopen(req, timeout=timeout_s) as resp:
        return resp.read().decode("utf-8", errors="replace")


def _normalize_url(raw: str) -> str | None:
    raw = raw.strip()
    if not raw:
        return None
    if raw.startswith("#"):
        return None
    if raw.startswith("mailto:"):
        return None
    if raw.startswith("http://") or raw.startswith("https://"):
        return raw
    if raw.startswith("/"):
        return f"{DOCS_ROOT}{raw}"
    return None


def parse_llms_txt(markdown: str) -> list[str]:
    urls: list[str] = []
    for m in _MD_LINK_RE.finditer(markdown):
        u = _normalize_url(m.group(1))
        if u:
            urls.append(u)
    # Always include llms index itself.
    urls.append(LLMS_TXT_URL)
    deduped = sorted(set(urls))
    return deduped


def _safe_filename(url: str) -> str:
    # Keep stable, filesystem-safe names.
    path = url.replace(DOCS_ROOT, "").lstrip("/")
    if not path:
        path = "root"
    path = re.sub(r"[^a-zA-Z0-9._/-]+", "_", path)
    path = path.strip("_").replace("/", "__")
    return f"{path}.html"


@dataclass(frozen=True)
class FetchResult:
    url: str
    ok: bool
    error: str | None = None
    out_path: Path | None = None


def fetch_pages(
    urls: Iterable[str],
    *,
    out_dir: Path,
    max_pages: int,
    concurrency: int,
    sleep_s: float,
) -> list[FetchResult]:
    out_dir.mkdir(parents=True, exist_ok=True)
    urls_list = list(urls) if max_pages <= 0 else list(urls)[:max_pages]

    def _one(url: str) -> FetchResult:
        if sleep_s:
            time.sleep(sleep_s)
        try:
            html = _fetch_text(url)
        except (urllib.error.URLError, TimeoutError) as e:
            return FetchResult(url=url, ok=False, error=str(e))
        out_path = out_dir / _safe_filename(url)
        out_path.write_text(html, encoding="utf-8")
        return FetchResult(url=url, ok=True, out_path=out_path)

    results: list[FetchResult] = []
    with ThreadPoolExecutor(max_workers=max(1, concurrency)) as ex:
        futs = [ex.submit(_one, u) for u in urls_list]
        for fut in as_completed(futs):
            results.append(fut.result())
    return results


def main() -> int:
    parser = argparse.ArgumentParser(description="Sync Streamlit docs starting from llms.txt.")
    parser.add_argument("--out", type=str, default="docs_snapshot", help="Output directory.")
    parser.add_argument("--fetch", action="store_true", help="Fetch pages (HTML) in addition to URL list.")
    parser.add_argument(
        "--max-pages",
        type=int,
        default=50,
        help="Max pages to fetch when --fetch is set. Use 0 to fetch all URLs from llms.txt.",
    )
    parser.add_argument("--concurrency", type=int, default=8, help="Fetch concurrency for --fetch.")
    parser.add_argument("--sleep-s", type=float, default=0.0, help="Optional sleep between fetches (per worker).")
    args = parser.parse_args()

    out_dir = Path(args.out).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    llms = _fetch_text(LLMS_TXT_URL)
    (out_dir / "llms.txt").write_text(llms, encoding="utf-8")

    urls = parse_llms_txt(llms)
    (out_dir / "urls.txt").write_text("\n".join(urls) + "\n", encoding="utf-8")
    print(f"[ok] Parsed {len(urls)} URLs from {LLMS_TXT_URL}")

    if not args.fetch:
        print(f"[ok] Wrote {out_dir / 'urls.txt'}")
        return 0

    pages_dir = out_dir / "pages"
    results = fetch_pages(
        urls,
        out_dir=pages_dir,
        max_pages=args.max_pages,
        concurrency=max(1, args.concurrency),
        sleep_s=max(0.0, args.sleep_s),
    )
    ok = sum(1 for r in results if r.ok)
    bad = len(results) - ok
    print(f"[ok] Fetched {ok} pages ({bad} failed) into {pages_dir}")
    return 0 if bad == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
