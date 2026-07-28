#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess


def run(args: list[str]) -> str:
    proc = subprocess.run(args, check=True, capture_output=True, text=True)
    return proc.stdout.strip()


def main() -> int:
    status = run(["git", "status", "--porcelain=v1"])
    branch = run(["git", "rev-parse", "--abbrev-ref", "HEAD"])
    remotes = [line.strip() for line in run(["git", "remote"]).splitlines() if line.strip()]
    preferred_remote = "origin" if "origin" in remotes else (remotes[0] if remotes else None)
    print(
        json.dumps(
            {
                "branch": branch,
                "has_changes": bool(status),
                "status_lines": status.splitlines(),
                "remotes": remotes,
                "preferred_remote": preferred_remote,
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
