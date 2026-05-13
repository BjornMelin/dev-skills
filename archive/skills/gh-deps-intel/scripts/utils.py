#!/usr/bin/env python3
"""Shared helpers for gh-deps-intel scripts."""

from __future__ import annotations

import json
import os
import re
import shlex
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Iterable


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def read_json(path: Path, default: Any = None) -> Any:
    if not path.exists():
        return default
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return default


def write_json(path: Path, data: Any) -> None:
    ensure_dir(path.parent)
    path.write_text(json.dumps(data, indent=2, ensure_ascii=True) + "\n", encoding="utf-8")


def run_cmd(
    cmd: list[str],
    cwd: Path | None = None,
    check: bool = False,
    env: dict[str, str] | None = None,
    timeout: int = 300,
) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        cmd,
        cwd=str(cwd) if cwd else None,
        env=env,
        check=False,
        text=True,
        capture_output=True,
        timeout=timeout,
    )
    if check and proc.returncode != 0:
        display = " ".join(shlex.quote(c) for c in cmd)
        raise RuntimeError(f"Command failed ({proc.returncode}): {display}\n{proc.stderr}")
    return proc


def which(name: str) -> str | None:
    return shutil_which(name)


def shutil_which(name: str) -> str | None:
    for path in os.environ.get("PATH", "").split(os.pathsep):
        candidate = Path(path) / name
        if candidate.exists() and os.access(candidate, os.X_OK):
            return str(candidate)
    return None


def parse_version_tuple(raw: str | None) -> tuple[int, ...]:
    if not raw:
        return tuple()
    cleaned = raw.strip().lower().lstrip("v")
    cleaned = cleaned.split("+")[0]
    cleaned = cleaned.split("-")[0]
    nums = re.findall(r"\d+", cleaned)
    if not nums:
        return tuple()
    return tuple(int(n) for n in nums)


def cmp_version(a: str | None, b: str | None) -> int:
    aa = list(parse_version_tuple(a))
    bb = list(parse_version_tuple(b))
    if not aa and not bb:
        return 0
    n = max(len(aa), len(bb))
    aa += [0] * (n - len(aa))
    bb += [0] * (n - len(bb))
    if aa < bb:
        return -1
    if aa > bb:
        return 1
    return 0


def sort_versions_desc(values: Iterable[str]) -> list[str]:
    uniq = {v for v in values if v}
    return sorted(uniq, key=lambda x: parse_version_tuple(x), reverse=True)


def extract_node_major(raw: str | None) -> int | None:
    if not raw:
        return None
    m = re.search(r"(\d{1,2})", raw)
    if not m:
        return None
    try:
        return int(m.group(1))
    except ValueError:
        return None


def sleep_with_jitter(seconds: float) -> None:
    # Keep deterministic enough for CI while adding minor jitter.
    jitter = 0.05
    time.sleep(max(0.0, seconds + jitter))


def compact_str(text: str | None, max_chars: int = 6000) -> str:
    if not text:
        return ""
    text = text.strip()
    if len(text) <= max_chars:
        return text
    return text[: max_chars - 3].rstrip() + "..."


def markdown_escape(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")
