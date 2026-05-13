#!/usr/bin/env python3
"""GitHub API access for releases/changelogs with rate-limit aware behavior."""

from __future__ import annotations

import base64
import json
import time
from pathlib import Path
from typing import Any

from utils import cmp_version, ensure_dir, read_json, run_cmd, sleep_with_jitter, write_json

CACHE_PATH = Path.home() / ".cache" / "gh-deps-intel" / "github-api-cache.json"
DEFAULT_TTL_SECONDS = 60 * 60 * 6


class GitHubApiError(RuntimeError):
    pass


class GitHubClient:
    def __init__(
        self,
        mode: str = "safe",
        max_retries: int = 4,
        min_interval_seconds: float = 0.2,
        cache_ttl_seconds: int = DEFAULT_TTL_SECONDS,
    ) -> None:
        self.mode = mode
        self.max_retries = max_retries
        self.min_interval_seconds = min_interval_seconds
        self.cache_ttl_seconds = cache_ttl_seconds
        self.cache = read_json(CACHE_PATH, default={}) or {}
        self.last_request_at = 0.0

    def flush_cache(self) -> None:
        ensure_dir(CACHE_PATH.parent)
        write_json(CACHE_PATH, self.cache)

    def _cache_get(self, key: str) -> Any:
        item = self.cache.get(key)
        if not isinstance(item, dict):
            return None
        ts = item.get("fetched_at")
        if not isinstance(ts, (int, float)):
            return None
        if time.time() - float(ts) > self.cache_ttl_seconds:
            return None
        return item.get("data")

    def _cache_set(self, key: str, data: Any) -> None:
        self.cache[key] = {"fetched_at": time.time(), "data": data}

    def _throttle(self) -> None:
        if self.mode != "safe":
            return
        elapsed = time.time() - self.last_request_at
        wait_for = self.min_interval_seconds - elapsed
        if wait_for > 0:
            sleep_with_jitter(wait_for)

    def _exec_api(self, path: str) -> Any:
        cmd = [
            "gh",
            "api",
            path,
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
        ]

        for attempt in range(self.max_retries + 1):
            self._throttle()
            proc = run_cmd(cmd, check=False)
            self.last_request_at = time.time()
            if proc.returncode == 0:
                try:
                    return json.loads(proc.stdout)
                except json.JSONDecodeError as exc:
                    raise GitHubApiError(f"Invalid JSON from gh api {path}: {exc}") from exc

            stderr = (proc.stderr or "").lower()
            stdout = (proc.stdout or "").lower()
            is_secondary = "secondary rate limit" in stderr or "secondary rate limit" in stdout
            is_rate = "rate limit" in stderr or "rate limit" in stdout
            if attempt < self.max_retries and (is_secondary or is_rate or "http 429" in stderr or "http 403" in stderr):
                delay = min(120, 5 * (2**attempt))
                sleep_with_jitter(delay)
                continue

            raise GitHubApiError(
                f"gh api failed for {path} (attempt {attempt + 1}): rc={proc.returncode}\n"
                f"stderr={proc.stderr[-2000:]}"
            )

        raise GitHubApiError(f"gh api failed for {path}: retries exhausted")

    def _exec_graphql(self, query: str, fields: dict[str, str]) -> Any:
        cmd = [
            "gh",
            "api",
            "graphql",
            "-f",
            f"query={query}",
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2022-11-28",
        ]
        for key, value in fields.items():
            cmd.extend(["-F", f"{key}={value}"])

        for attempt in range(self.max_retries + 1):
            self._throttle()
            proc = run_cmd(cmd, check=False)
            self.last_request_at = time.time()
            if proc.returncode == 0:
                try:
                    return json.loads(proc.stdout)
                except json.JSONDecodeError as exc:
                    raise GitHubApiError(f"Invalid JSON from gh graphql: {exc}") from exc

            stderr = (proc.stderr or "").lower()
            if attempt < self.max_retries and ("rate limit" in stderr or "http 429" in stderr or "http 403" in stderr):
                delay = min(120, 5 * (2**attempt))
                sleep_with_jitter(delay)
                continue
            raise GitHubApiError(f"gh graphql failed: rc={proc.returncode} stderr={proc.stderr[-2000:]}")

        raise GitHubApiError("gh graphql failed: retries exhausted")

    def get_json(self, path: str, cache_key: str | None = None) -> Any:
        key = cache_key or f"gh:{path}"
        cached = self._cache_get(key)
        if cached is not None:
            return cached
        data = self._exec_api(path)
        self._cache_set(key, data)
        return data

    def get_paginated_list(self, path: str, max_pages: int = 10) -> list[dict[str, Any]]:
        all_rows: list[dict[str, Any]] = []
        for page in range(1, max_pages + 1):
            sep = "&" if "?" in path else "?"
            page_path = f"{path}{sep}per_page=100&page={page}"
            key = f"gh:{page_path}"
            rows = self.get_json(page_path, cache_key=key)
            if not isinstance(rows, list):
                break
            items = [r for r in rows if isinstance(r, dict)]
            all_rows.extend(items)
            if len(items) < 100:
                break
        return all_rows

    def get_releases(self, owner: str, repo: str) -> list[dict[str, Any]]:
        try:
            rows = self.get_paginated_list(f"repos/{owner}/{repo}/releases")
            if rows:
                return rows
        except GitHubApiError:
            rows = []

        # GraphQL fallback when REST data is unavailable/empty.
        gql = """
query($owner:String!, $repo:String!, $first:Int!) {
  repository(owner:$owner, name:$repo) {
    releases(first:$first, orderBy:{field:CREATED_AT, direction:DESC}) {
      nodes {
        name
        tagName
        publishedAt
        isDraft
        isPrerelease
        url
        description
      }
    }
  }
}
"""
        try:
            data = self._exec_graphql(gql, {"owner": owner, "repo": repo, "first": "100"})
        except GitHubApiError:
            return rows
        nodes = (
            (((data or {}).get("data") or {}).get("repository") or {}).get("releases") or {}
        ).get("nodes")
        if not isinstance(nodes, list):
            return rows
        mapped: list[dict[str, Any]] = []
        for n in nodes:
            if not isinstance(n, dict):
                continue
            mapped.append(
                {
                    "name": n.get("name"),
                    "tag_name": n.get("tagName"),
                    "published_at": n.get("publishedAt"),
                    "draft": bool(n.get("isDraft")),
                    "prerelease": bool(n.get("isPrerelease")),
                    "html_url": n.get("url"),
                    "body": n.get("description") or "",
                }
            )
        return mapped

    def get_tags(self, owner: str, repo: str) -> list[dict[str, Any]]:
        return self.get_paginated_list(f"repos/{owner}/{repo}/tags")

    def get_rate_limit(self) -> dict[str, Any]:
        data = self.get_json("rate_limit")
        return data if isinstance(data, dict) else {}

    def get_compare(self, owner: str, repo: str, base: str, head: str) -> dict[str, Any] | None:
        try:
            data = self.get_json(f"repos/{owner}/{repo}/compare/{base}...{head}")
            return data if isinstance(data, dict) else None
        except Exception:
            return None

    def get_changelog(self, owner: str, repo: str) -> dict[str, Any] | None:
        candidate_paths = [
            "CHANGELOG.md",
            "changelog.md",
            "CHANGES.md",
            "changes.md",
            "docs/CHANGELOG.md",
        ]
        for path in candidate_paths:
            api_path = f"repos/{owner}/{repo}/contents/{path}"
            try:
                data = self.get_json(api_path, cache_key=f"gh:contents:{owner}/{repo}:{path}")
            except Exception:
                continue
            if not isinstance(data, dict):
                continue
            content = data.get("content")
            encoding = data.get("encoding")
            if isinstance(content, str) and encoding == "base64":
                try:
                    decoded = base64.b64decode(content).decode("utf-8", errors="ignore")
                except Exception:
                    continue
                return {
                    "path": path,
                    "text": decoded,
                    "html_url": data.get("html_url"),
                }
        return None


def _coerce_version(tag: str | None) -> str | None:
    if not tag:
        return None
    t = str(tag).strip()
    if not t:
        return None
    t = t.split("/")[-1]
    if t.lower().startswith("release-"):
        t = t[len("release-") :]
    if t.lower().startswith("v"):
        t = t[1:]
    return t


def filter_releases_between(
    releases: list[dict[str, Any]],
    current_version: str | None,
    target_version: str | None,
    max_items: int = 25,
) -> list[dict[str, Any]]:
    current = _coerce_version(current_version)
    target = _coerce_version(target_version)

    normalized: list[dict[str, Any]] = []
    for rel in releases:
        tag = rel.get("tag_name") or rel.get("name")
        ver = _coerce_version(tag)
        normalized.append(
            {
                "name": rel.get("name") or tag,
                "tag_name": rel.get("tag_name"),
                "version": ver,
                "published_at": rel.get("published_at"),
                "html_url": rel.get("html_url"),
                "body": rel.get("body") or "",
                "draft": bool(rel.get("draft")),
                "prerelease": bool(rel.get("prerelease")),
            }
        )

    normalized = [x for x in normalized if not x["draft"]]

    selected: list[dict[str, Any]] = []
    for rel in normalized:
        ver = rel.get("version")
        if not ver:
            continue
        if current and cmp_version(ver, current) <= 0:
            continue
        if target and cmp_version(ver, target) > 0:
            continue
        selected.append(rel)

    if not selected:
        # If we have an explicit version window but cannot confidently map releases
        # into that window, return none instead of injecting potentially unrelated notes.
        if current or target:
            return []
        selected = normalized[:max_items]

    # Stable latest-first ordering.
    selected.sort(key=lambda x: x.get("published_at") or "", reverse=True)
    return selected[:max_items]
