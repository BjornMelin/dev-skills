#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Wrapper around the shared convex-scan inventory command.")
    parser.add_argument("--cwd", default=".")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    cmd = [
        "/home/bjorn/.codex/skill-support/bin/convex-scan",
        "inventory",
        "--cwd",
        str(Path(args.cwd).resolve()),
        "--out",
        args.out,
    ]
    return subprocess.run(cmd, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())

