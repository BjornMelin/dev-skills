#!/usr/bin/env python3
"""Runtime-aware target version policy."""

from __future__ import annotations

import re
from typing import Any

from utils import cmp_version, parse_version_tuple


def _parse_python_tuple(raw: str | None) -> tuple[int, int] | None:
    if not raw:
        return None
    m = re.search(r"(\d+)\.(\d+)", raw)
    if not m:
        return None
    return int(m.group(1)), int(m.group(2))


def _tuple_cmp(a: tuple[int, int], b: tuple[int, int]) -> int:
    if a < b:
        return -1
    if a > b:
        return 1
    return 0


def _python_requires_satisfied(requires: str | None, runtime: tuple[int, int] | None) -> bool:
    if not requires or not runtime:
        return True

    clauses = [c.strip() for c in requires.split(",") if c.strip()]
    for clause in clauses:
        m = re.match(r"(<=|>=|==|!=|<|>)\s*(\d+)(?:\.(\d+))?", clause)
        if not m:
            continue
        op = m.group(1)
        major = int(m.group(2))
        minor = int(m.group(3) or 0)
        target = (major, minor)
        c = _tuple_cmp(runtime, target)
        if op == "==" and c != 0:
            return False
        if op == "!=" and c == 0:
            return False
        if op == ">=" and c < 0:
            return False
        if op == ">" and c <= 0:
            return False
        if op == "<=" and c > 0:
            return False
        if op == "<" and c >= 0:
            return False
    return True


def _pick_types_node_target(versions: list[str], node_major: int | None) -> str | None:
    if not versions or node_major is None:
        return None
    candidates = []
    for v in versions:
        parts = parse_version_tuple(v)
        if not parts:
            continue
        if parts[0] == node_major:
            candidates.append(v)
    if not candidates:
        return None
    candidates.sort(key=lambda x: parse_version_tuple(x), reverse=True)
    return candidates[0]


def _pick_python_compatible_target(
    versions: list[str],
    release_requires_python: dict[str, str],
    runtime: tuple[int, int] | None,
) -> str | None:
    if not versions:
        return None
    for v in versions:
        requires = release_requires_python.get(v)
        if _python_requires_satisfied(requires, runtime):
            return v
    return versions[0]


def choose_target_version(
    dep: dict[str, Any],
    outdated_lookup: dict[str, dict[str, str]],
    resolved: dict[str, Any],
    repo_context: dict[str, Any],
    compatibility_policy: str = "runtime-pinned",
) -> dict[str, Any]:
    ecosystem = dep.get("ecosystem")
    name = dep.get("name")

    out_row = outdated_lookup.get(name, {})
    current = out_row.get("current") or dep.get("current_version_hint")
    latest_outdated = out_row.get("latest")

    versions = resolved.get("versions") or []
    latest_registry = resolved.get("latest") or (versions[0] if versions else None)
    latest_available = latest_outdated or latest_registry

    target = latest_available
    reason = "latest available"

    if compatibility_policy == "always-latest":
        absolute_latest = resolved.get("latest") or (versions[0] if versions else latest_available)
        target = absolute_latest
        reason = "always-latest policy"
    elif compatibility_policy == "semver-only":
        # Keep manager-provided latest/wanted signals; do not runtime-pin.
        target = latest_available
        reason = "semver-only policy"
    else:
        if ecosystem == "npm" and name == "@types/node":
            node_major = (repo_context.get("node_runtime") or {}).get("major")
            pinned = _pick_types_node_target(versions, node_major)
            if pinned:
                target = pinned
                reason = f"aligned @types/node major with detected Node runtime ({node_major})"

        if ecosystem == "pypi":
            runtime = _parse_python_tuple((repo_context.get("python_runtime") or {}).get("detected"))
            rel_req = (resolved.get("metadata") or {}).get("release_requires_python")
            rel_req = rel_req if isinstance(rel_req, dict) else {}
            compatible = _pick_python_compatible_target(versions, rel_req, runtime)
            if compatible:
                target = compatible
                if compatible != latest_available:
                    reason = "latest runtime-compatible release"

    if target and latest_available and cmp_version(target, latest_available) < 0 and reason == "latest available":
        reason = "selected by compatibility policy"

    return {
        "current": current,
        "latest_available": latest_available,
        "target": target,
        "reason": reason,
        "outdated_source": out_row.get("source"),
    }
