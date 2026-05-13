#!/usr/bin/env python3
"""Convenience wrapper for single dependency upgrade spec."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


def main() -> None:
    parser = argparse.ArgumentParser(description="Run gh-deps-intel package workflow for one dependency")
    parser.add_argument("dependency", help="Dependency selector/name (e.g. @types/node, workflow)")
    parser.add_argument("--repo", default=".", help="Target repository root")
    parser.add_argument("--out", default="reports", help="Output directory")
    parser.add_argument("--mode", choices=["safe", "fast"], default="safe")
    parser.add_argument(
        "--compatibility-policy",
        default="runtime-pinned",
        choices=["runtime-pinned", "semver-only", "always-latest"],
    )
    args = parser.parse_args()

    script = Path(__file__).resolve().parent / "gh_deps_intel.py"
    cmd = [
        "python3",
        str(script),
        "package",
        "--repo",
        args.repo,
        "--out",
        args.out,
        "--mode",
        args.mode,
        "--compatibility-policy",
        args.compatibility_policy,
        "--dependency",
        args.dependency,
    ]
    raise SystemExit(subprocess.call(cmd))


if __name__ == "__main__":
    main()
