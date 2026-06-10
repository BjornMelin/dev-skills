#!/usr/bin/env python3
"""Summarize compare commits between two tags/refs."""

from __future__ import annotations

import argparse
import json

from gh_release_fetch import GitHubApiError, GitHubClient


def main() -> None:
    """Fetch a GitHub compare range and print a compact JSON summary."""
    parser = argparse.ArgumentParser(description="Summarize GitHub compare output.")
    parser.add_argument("repo", help="owner/repo")
    parser.add_argument("base")
    parser.add_argument("head")
    args = parser.parse_args()

    if "/" not in args.repo:
        raise SystemExit("repo must be owner/repo")
    owner, repo = args.repo.split("/", 1)

    client = GitHubClient(mode="safe")
    try:
        data = client.get_compare(owner, repo, args.base, args.head)
    except GitHubApiError as exc:
        print(
            json.dumps(
                {
                    "repo": args.repo,
                    "base": args.base,
                    "head": args.head,
                    "error": str(exc),
                },
                indent=2,
            )
        )
        return
    finally:
        client.flush_cache()
    if not data:
        print(json.dumps({"error": "compare data unavailable"}, indent=2))
        return

    commits = data.get("commits") if isinstance(data, dict) else []
    summary: list[str] = []
    if isinstance(commits, list):
        for c in commits[:40]:
            if not isinstance(c, dict):
                continue
            sha = str(c.get("sha") or "")[:7]
            msg = ""
            commit = c.get("commit")
            if isinstance(commit, dict):
                m = commit.get("message")
                if isinstance(m, str):
                    msg = m.splitlines()[0]
            if msg:
                summary.append(f"{sha} {msg}")

    print(
        json.dumps(
            {
                "repo": args.repo,
                "base": args.base,
                "head": args.head,
                "summary": summary,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
