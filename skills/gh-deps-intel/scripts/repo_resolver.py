#!/usr/bin/env python3
"""Resolve package metadata and source repositories (prefer GitHub)."""

from __future__ import annotations

import json
import re
import time
from pathlib import Path
from typing import Any
from urllib.parse import quote
from urllib.request import Request, urlopen

from utils import ensure_dir, read_json, sort_versions_desc, write_json

CACHE_PATH = Path.home() / ".cache" / "gh-deps-intel" / "registry-cache.json"
CACHE_TTL_SECONDS = 60 * 60 * 12


class RegistryCache:
    def __init__(self, path: Path = CACHE_PATH, ttl_seconds: int = CACHE_TTL_SECONDS) -> None:
        self.path = path
        self.ttl_seconds = ttl_seconds
        self.data = read_json(path, default={}) or {}

    def get(self, key: str) -> Any:
        item = self.data.get(key)
        if not isinstance(item, dict):
            return None
        ts = item.get("fetched_at")
        if not isinstance(ts, (int, float)):
            return None
        if time.time() - float(ts) > self.ttl_seconds:
            return None
        return item.get("data")

    def set(self, key: str, value: Any) -> None:
        self.data[key] = {"fetched_at": time.time(), "data": value}

    def flush(self) -> None:
        ensure_dir(self.path.parent)
        write_json(self.path, self.data)


CACHE = RegistryCache()


def _http_json(url: str) -> Any:
    cached = CACHE.get(url)
    if cached is not None:
        return cached

    req = Request(url, headers={"User-Agent": "gh-deps-intel/1.0"})
    with urlopen(req, timeout=30) as resp:  # nosec - controlled URLs
        body = resp.read().decode("utf-8", errors="ignore")
    data = json.loads(body)
    CACHE.set(url, data)
    return data


def extract_github_repo(url: str | None) -> str | None:
    if not url:
        return None
    normalized = url.strip()
    patterns = [
        r"github\.com[:/](?P<owner>[A-Za-z0-9_.-]+)/(?P<repo>[A-Za-z0-9_.-]+)",
    ]
    for pat in patterns:
        m = re.search(pat, normalized)
        if not m:
            continue
        owner = m.group("owner")
        repo = m.group("repo")
        repo = repo[:-4] if repo.endswith(".git") else repo
        return f"{owner}/{repo}"
    return None


def _pick_repository_url_from_npm(meta: dict[str, Any]) -> str | None:
    repo = meta.get("repository")
    if isinstance(repo, str):
        return repo
    if isinstance(repo, dict):
        url = repo.get("url")
        if isinstance(url, str):
            return url
    for field in ["homepage", "bugs"]:
        candidate = meta.get(field)
        if isinstance(candidate, str):
            return candidate
        if isinstance(candidate, dict):
            u = candidate.get("url")
            if isinstance(u, str):
                return u
    return None


def resolve_npm(name: str) -> dict[str, Any]:
    encoded = quote(name, safe="")
    url = f"https://registry.npmjs.org/{encoded}"
    try:
        data = _http_json(url)
    except Exception as exc:
        return {
            "ecosystem": "npm",
            "name": name,
            "error": f"npm registry lookup failed: {exc}",
            "source_repo": None,
            "versions": [],
            "latest": None,
            "links": {},
        }

    latest = None
    dist_tags = data.get("dist-tags") if isinstance(data, dict) else None
    if isinstance(dist_tags, dict):
        latest = dist_tags.get("latest")

    versions = sort_versions_desc(list((data.get("versions") or {}).keys()) if isinstance(data, dict) else [])

    repo_url = _pick_repository_url_from_npm(data if isinstance(data, dict) else {})
    source_repo = extract_github_repo(repo_url)

    links = {
        "npm": f"https://www.npmjs.com/package/{name}",
    }
    if source_repo:
        links["github"] = f"https://github.com/{source_repo}"

    return {
        "ecosystem": "npm",
        "name": name,
        "source_repo": source_repo,
        "source_repo_url": f"https://github.com/{source_repo}" if source_repo else None,
        "versions": versions,
        "latest": latest,
        "links": links,
        "metadata": {
            "repository": repo_url,
            "homepage": data.get("homepage") if isinstance(data, dict) else None,
            "description": data.get("description") if isinstance(data, dict) else None,
        },
    }


def _pick_repository_url_from_pypi(info: dict[str, Any]) -> str | None:
    direct_fields = ["project_url", "home_page", "download_url", "package_url"]
    for f in direct_fields:
        v = info.get(f)
        if isinstance(v, str) and v:
            if "github.com" in v.lower() or f in {"project_url", "home_page"}:
                return v
    project_urls = info.get("project_urls")
    if isinstance(project_urls, dict):
        preferred_keys = ["Source", "Homepage", "Repository", "Code", "Changelog", "Documentation"]
        for key in preferred_keys:
            v = project_urls.get(key)
            if isinstance(v, str) and v:
                return v
        for v in project_urls.values():
            if isinstance(v, str) and v:
                return v
    return None


def resolve_pypi(name: str) -> dict[str, Any]:
    url = f"https://pypi.org/pypi/{quote(name)}/json"
    try:
        data = _http_json(url)
    except Exception as exc:
        return {
            "ecosystem": "pypi",
            "name": name,
            "error": f"PyPI lookup failed: {exc}",
            "source_repo": None,
            "versions": [],
            "latest": None,
            "links": {},
        }

    info = data.get("info") if isinstance(data, dict) else {}
    releases = data.get("releases") if isinstance(data, dict) else {}
    latest = info.get("version") if isinstance(info, dict) else None
    versions = sort_versions_desc(list(releases.keys()) if isinstance(releases, dict) else [])
    release_requires_python: dict[str, str] = {}
    if isinstance(releases, dict):
        for ver, files in releases.items():
            if not isinstance(files, list):
                continue
            req = None
            for file in files:
                if not isinstance(file, dict):
                    continue
                value = file.get("requires_python")
                if isinstance(value, str) and value.strip():
                    req = value.strip()
                    break
            if req:
                release_requires_python[ver] = req

    repo_url = _pick_repository_url_from_pypi(info if isinstance(info, dict) else {})
    source_repo = extract_github_repo(repo_url)

    links = {
        "pypi": f"https://pypi.org/project/{name}/",
    }
    if source_repo:
        links["github"] = f"https://github.com/{source_repo}"

    return {
        "ecosystem": "pypi",
        "name": name,
        "source_repo": source_repo,
        "source_repo_url": f"https://github.com/{source_repo}" if source_repo else None,
        "versions": versions,
        "latest": latest,
        "links": links,
        "metadata": {
            "repository": repo_url,
            "summary": info.get("summary") if isinstance(info, dict) else None,
            "requires_python": info.get("requires_python") if isinstance(info, dict) else None,
            "project_urls": info.get("project_urls") if isinstance(info, dict) else None,
            "release_requires_python": release_requires_python,
        },
    }


def resolve_dependency(dep: dict[str, Any]) -> dict[str, Any]:
    ecosystem = dep.get("ecosystem")
    name = dep.get("name")
    if ecosystem == "npm":
        return resolve_npm(name)
    if ecosystem == "pypi":
        return resolve_pypi(name)
    return {
        "ecosystem": ecosystem,
        "name": name,
        "source_repo": None,
        "versions": [],
        "latest": None,
        "links": {},
        "metadata": {},
    }


def flush_cache() -> None:
    CACHE.flush()


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="Resolve package metadata and source repository.")
    parser.add_argument("ecosystem", choices=["npm", "pypi"])
    parser.add_argument("name")
    args = parser.parse_args()

    dep = {"ecosystem": args.ecosystem, "name": args.name}
    print(json.dumps(resolve_dependency(dep), indent=2))
    flush_cache()


if __name__ == "__main__":
    main()
