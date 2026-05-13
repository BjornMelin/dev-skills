#!/usr/bin/env python3
"""Fetch release notes between two versions for a GitHub repository."""

from __future__ import annotations

import argparse
import json

from gh_release_fetch import GitHubClient, filter_releases_between


def main() -> None:
    parser = argparse.ArgumentParser(description="Get releases between two versions.")
    parser.add_argument("repo", help="owner/repo")
    parser.add_argument("--current", default=None, help="Current version")
    parser.add_argument("--target", default=None, help="Target version")
    parser.add_argument("--mode", choices=["safe", "fast"], default="safe")
    args = parser.parse_args()

    if "/" not in args.repo:
        raise SystemExit("repo must be owner/repo")
    owner, repo = args.repo.split("/", 1)

    client = GitHubClient(mode=args.mode)
    releases = client.get_releases(owner, repo)
    selected = filter_releases_between(releases, args.current, args.target, max_items=30)
    client.flush_cache()
    print(json.dumps(selected, indent=2))


if __name__ == "__main__":
    main()
