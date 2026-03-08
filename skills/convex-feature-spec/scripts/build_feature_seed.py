#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(description="Create a minimal feature-spec seed from repo context.")
    parser.add_argument("--cwd", default=".")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    repo = Path(args.cwd).resolve()
    payload = {
        "repo_root": str(repo),
        "notes": [
            "Read README.md, AGENTS.md, and architecture docs before proposing feature scope.",
            "Use convex-scan inventory output to keep schema and function changes grounded.",
        ],
    }
    Path(args.out).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(args.out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

