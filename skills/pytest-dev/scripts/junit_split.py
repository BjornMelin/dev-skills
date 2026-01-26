#!/usr/bin/env python3
"""Split test files into N shards using historical JUnit XML timings.

Goal: make GitHub Actions (or any CI matrix) shards finish at roughly the same
time by assigning *test files* to each shard based on measured durations.

This script intentionally shards by file (not individual nodeids) because JUnit
XML often lacks a stable pytest nodeid. Sharding by file is usually a strong
baseline and works well with `pytest -n auto --dist loadfile`.
"""

from __future__ import annotations

import argparse
import statistics
import sys
import xml.etree.ElementTree as ET
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


def _iter_xml_paths(path: Path) -> Iterable[Path]:
    if path.is_dir():
        yield from sorted(p for p in path.rglob("*.xml") if p.is_file())
        return
    yield path


def _safe_float(value: str | None) -> float:
    if value is None:
        return 0.0
    try:
        return float(value)
    except ValueError:
        return 0.0


def _guess_file_from_classname(classname: str) -> str | None:
    # Usually looks like: "tests.test_mod" or "tests.test_mod.TestClass".
    parts = classname.split(".")
    if len(parts) >= 2 and parts[-1][:1].isupper():
        parts = parts[:-1]
    joined = "/".join(parts)
    if not joined:
        return None
    return f"{joined}.py"


def _timings_by_file(xml_paths: Iterable[Path]) -> dict[str, float]:
    totals: dict[str, float] = {}
    for xml_path in xml_paths:
        try:
            root = ET.parse(xml_path).getroot()
        except (ET.ParseError, OSError) as exc:
            raise RuntimeError(f"Failed to parse {xml_path}: {exc}") from exc

        for testcase in root.iter("testcase"):
            seconds = _safe_float(testcase.get("time"))
            if seconds <= 0:
                continue
            file_id = testcase.get("file")
            if not file_id:
                classname = testcase.get("classname")
                if classname:
                    file_id = _guess_file_from_classname(classname)
            if not file_id:
                continue
            totals[file_id] = totals.get(file_id, 0.0) + seconds
    return totals


@dataclass(frozen=True)
class _WeightedFile:
    path: str
    seconds: float


def _glob_files(globs: list[str]) -> list[str]:
    out: list[str] = []
    for pattern in globs:
        out.extend(str(p) for p in sorted(Path().glob(pattern)) if p.is_file())
    # De-dupe while preserving order.
    seen: set[str] = set()
    deduped: list[str] = []
    for p in out:
        if p in seen:
            continue
        seen.add(p)
        deduped.append(p)
    return deduped


def _binpack(
    weighted: list[_WeightedFile], groups: int
) -> list[list[_WeightedFile]]:
    # Greedy largest-first bin packing.
    bins: list[list[_WeightedFile]] = [[] for _ in range(groups)]
    totals = [0.0 for _ in range(groups)]
    for item in sorted(weighted, key=lambda w: w.seconds, reverse=True):
        idx = min(range(groups), key=lambda i: totals[i])
        bins[idx].append(item)
        totals[idx] += item.seconds
    return bins


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--junitxml",
        required=True,
        type=Path,
        help="JUnit XML file or directory containing XML files.",
    )
    parser.add_argument(
        "--glob",
        action="append",
        default=[],
        help="Glob for test files (repeatable). Example: --glob 'tests/**/*.py'",
    )
    parser.add_argument(
        "--groups",
        type=int,
        required=True,
        help="Number of shards/groups.",
    )
    parser.add_argument(
        "--index",
        type=int,
        default=None,
        help="If set, print only this shard's file list (0-based).",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help="If set, write shard_0.txt..shard_{n-1}.txt into this directory.",
    )
    args = parser.parse_args(argv)

    if args.groups < 1:
        print("--groups must be >= 1", file=sys.stderr)
        return 2
    if args.index is not None and (args.index < 0 or args.index >= args.groups):
        print("--index must be within [0, groups)", file=sys.stderr)
        return 2

    xml_paths = list(_iter_xml_paths(args.junitxml))
    if not xml_paths:
        print(f"No XML files found under: {args.junitxml}", file=sys.stderr)
        return 2

    files = _glob_files(args.glob) if args.glob else []
    if not files:
        print("No test files matched. Provide at least one --glob.", file=sys.stderr)
        return 2

    timings = _timings_by_file(xml_paths)
    known = list(timings.values())
    # If the timing corpus is tiny (e.g., a partial report), using its median as
    # the default can massively overweight unknown files. Prefer a conservative
    # default until we have a meaningful sample size.
    default_seconds = statistics.median(known) if len(known) >= 50 else 1.0

    weighted: list[_WeightedFile] = []
    for f in files:
        seconds = timings.get(f)
        if seconds is None:
            # JUnit may use module-ish paths; try a best-effort normalization.
            alt = f.replace("\\", "/")
            seconds = timings.get(alt, default_seconds)
        weighted.append(_WeightedFile(path=f, seconds=seconds))

    bins = _binpack(weighted, groups=args.groups)

    if args.out_dir is not None:
        args.out_dir.mkdir(parents=True, exist_ok=True)
        for i, shard in enumerate(bins):
            out_path = args.out_dir / f"shard_{i}.txt"
            out_path.write_text("".join(f"{w.path}\n" for w in shard), encoding="utf-8")

    if args.index is not None:
        for w in bins[args.index]:
            print(w.path)
        return 0

    # Print a short summary for humans.
    totals = [sum(w.seconds for w in shard) for shard in bins]
    for i, total in enumerate(totals):
        print(f"shard {i}: {total:.2f}s ({len(bins[i])} files)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
