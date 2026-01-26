#!/usr/bin/env python3
"""Run pytest for a newline-delimited list of test files.

Useful for CI sharding where a prior step produces `shard_N.txt` containing a
list of test files to execute.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def _read_filelist(path: Path) -> list[str]:
    files: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        files.append(line)
    return files


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("filelist", type=Path, help="Path to shard_N.txt file.")
    parser.add_argument(
        "--python",
        default=sys.executable,
        help="Python interpreter to run pytest with.",
    )
    parser.add_argument(
        "pytest_args",
        nargs=argparse.REMAINDER,
        help="Extra pytest args (prefix with -- to separate).",
    )
    args = parser.parse_args(argv)

    files = _read_filelist(args.filelist)
    if not files:
        print(f"No files in: {args.filelist}", file=sys.stderr)
        return 2

    pytest_args = args.pytest_args
    if pytest_args and pytest_args[0] == "--":
        pytest_args = pytest_args[1:]

    cmd = [args.python, "-m", "pytest", *files, *pytest_args]
    return subprocess.call(cmd)


if __name__ == "__main__":
    raise SystemExit(main())

