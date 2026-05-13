#!/usr/bin/env python3
"""Fallback metadata links when GitHub data is missing or limited."""

from __future__ import annotations

from typing import Any


def collect_fallback_links(dep: dict[str, Any], resolved: dict[str, Any]) -> list[dict[str, str]]:
    ecosystem = dep.get("ecosystem")
    name = dep.get("name")
    links: list[dict[str, str]] = []

    if ecosystem == "npm":
        links.append({"label": "npm package", "url": f"https://www.npmjs.com/package/{name}"})
    elif ecosystem == "pypi":
        links.append({"label": "PyPI package", "url": f"https://pypi.org/project/{name}/"})

    meta_links = resolved.get("links") if isinstance(resolved, dict) else None
    if isinstance(meta_links, dict):
        for key, value in meta_links.items():
            if not isinstance(value, str) or not value.startswith("http"):
                continue
            label = f"{key}"
            links.append({"label": label, "url": value})

    project_urls = (resolved.get("metadata") or {}).get("project_urls")
    if isinstance(project_urls, dict):
        for k, v in project_urls.items():
            if isinstance(v, str) and v.startswith("http"):
                links.append({"label": f"project_url:{k}", "url": v})

    # Deduplicate by URL while preserving order.
    seen: set[str] = set()
    deduped: list[dict[str, str]] = []
    for item in links:
        url = item.get("url")
        if not url or url in seen:
            continue
        seen.add(url)
        deduped.append(item)
    return deduped
