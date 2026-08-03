#!/usr/bin/env python3
"""Repo-wide guard: no private-project references in the public catalog.

This repository is PUBLIC and feeds the Agent Skills Lab. Private-project
skills and flavored variants belong in their private repos (for example
signr-skills), never here. This check scans every tracked text file under the
published surfaces for the private-marker patterns and fails on any hit that
is not explicitly allowlisted.

Run: python3 tools/policy/check_public_leaks.py [--staged]
  --staged  scan only files staged in git (pre-commit mode)
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

PRIVATE_MARKERS = re.compile(r"signr|career-os|tripsage|/home/bjorn", re.IGNORECASE)

# Surfaces that get published or copied out of this repo.
SCANNED_PREFIXES = ("skills/", "plugins/", "subagents/", "docs/", "catalog/", "bootstrap/")

TEXT_SUFFIXES = {".md", ".mdx", ".txt", ".yaml", ".yml", ".json", ".toml",
                 ".mjs", ".js", ".ts", ".tsx", ".py", ".sh", ".css", ".html"}

# Files allowed to carry a marker, with the reason on record.
ALLOWLIST = {
    # The guards themselves must name the patterns they reject.
    "tools/policy/check_public_leaks.py",
    "tools/skill/check_expo_motion_public_contract.py",
}


def tracked_files(staged: bool) -> list[str]:
    cmd = ["git", "diff", "--cached", "--name-only", "--diff-filter=ACMR"] if staged \
        else ["git", "ls-files"]
    out = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True, check=True)
    return [line for line in out.stdout.splitlines() if line]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--staged", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    for rel in tracked_files(args.staged):
        if rel in ALLOWLIST:
            continue
        if not (rel.startswith(SCANNED_PREFIXES) or args.staged):
            continue
        path = REPO / rel
        if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError:
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            if PRIVATE_MARKERS.search(line):
                failures.append(f"{rel}:{lineno}: {line.strip()[:120]}")

    if failures:
        print("PUBLIC LEAK CHECK FAILED: private-project references found:",
              file=sys.stderr)
        for f in failures:
            print(f"  {f}", file=sys.stderr)
        print("\nPrivate-project content belongs in its private skills repo "
              "(e.g. signr-skills). If a hit is a legitimate guard or doc, add "
              "it to ALLOWLIST with a reason.", file=sys.stderr)
        return 1
    print("public leak check: clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
