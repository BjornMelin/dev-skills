#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path


def run(args: list[str]) -> str:
    proc = subprocess.run(args, check=True, capture_output=True, text=True)
    return proc.stdout.strip()


def infer_repo(explicit_repo: str | None) -> str:
    if explicit_repo:
        return explicit_repo
    try:
        return json.loads(run(["gh", "repo", "view", "--json", "nameWithOwner"]))["nameWithOwner"]
    except Exception:
        remote = run(["git", "remote", "get-url", "origin"])
        match = re.search(r"github\.com[:/](.+?)(?:\.git)?$", remote)
        if not match:
            raise RuntimeError("Could not infer GitHub repository from gh or git remote")
        return match.group(1)


def infer_pr(explicit_pr: int | None, explicit_url: str | None, repo: str) -> int:
    if explicit_pr:
        return explicit_pr
    if explicit_url:
        match = re.search(r"/pull/(\d+)", explicit_url)
        if match:
            return int(match.group(1))
        raise RuntimeError("Could not parse PR number from URL")
    try:
        return json.loads(run(["gh", "pr", "view", "--json", "number", "-R", repo]))["number"]
    except Exception:
        branch = run(["git", "rev-parse", "--abbrev-ref", "HEAD"])
        prs = json.loads(run(["gh", "pr", "list", "-R", repo, "--head", branch, "--json", "number", "--limit", "1"]))
        if prs:
            return int(prs[0]["number"])
    raise RuntimeError("Could not infer PR number from explicit input, current branch, or gh pr view")


def main() -> int:
    parser = argparse.ArgumentParser(description="Infer repo/PR and fetch a normalized review bundle.")
    parser.add_argument("--repo")
    parser.add_argument("--pr", type=int)
    parser.add_argument("--url")
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()

    repo = infer_repo(args.repo)
    pr = infer_pr(args.pr, args.url, repo)
    out = args.out or Path(tempfile.gettempdir()) / f"gh-pr-review-fix-{repo.replace('/', '_')}-{pr}.json"

    cmd = [
        "/home/bjorn/.codex/skill-support/bin/review-pack",
        "fetch-pr",
        "--repo",
        repo,
        "--pr",
        str(pr),
        "--out",
        str(out),
    ]
    subprocess.run(cmd, check=True)
    print(json.dumps({"repo": repo, "pr": pr, "bundle": str(out)}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
