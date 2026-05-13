#!/usr/bin/env python3
"""Quick GitHub rate-limit diagnostic."""

from __future__ import annotations

import json

from gh_release_fetch import GitHubClient


def main() -> None:
    client = GitHubClient(mode="safe")
    data = client.get_rate_limit()
    client.flush_cache()
    print(json.dumps(data, indent=2))


if __name__ == "__main__":
    main()
