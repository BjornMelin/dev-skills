#!/usr/bin/env python3
"""Convenience wrapper: `collect` using the vendored docs_drift CLI next to this file."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Run vendored docs drift collect.")
    parser.add_argument("--cwd", default=".")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    script = Path(__file__).resolve().parent / "docs_drift.py"
    cmd = [
        sys.executable,
        str(script),
        "collect",
        "--cwd",
        str(Path(args.cwd).resolve()),
        "--out",
        args.out,
    ]
    return subprocess.run(cmd, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
