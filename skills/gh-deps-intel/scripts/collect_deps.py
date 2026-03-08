#!/usr/bin/env python3
"""Dependency extraction from package.json and pyproject.toml files."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any


def _read_package_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def _extract_npm_version_hint(spec: str | None) -> str | None:
    if not spec:
        return None
    s = spec.strip()
    if s.startswith("workspace:"):
        s = s.split(":", 1)[1].strip()
    if s in {"*", "latest", "next"}:
        return None
    m = re.search(r"(\d+\.\d+(?:\.\d+)?)", s)
    return m.group(1) if m else None


def collect_js_dependencies(package_json_files: list[str], repo_root: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    sections = ["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"]
    for file in package_json_files:
        path = Path(file)
        data = _read_package_json(path)
        for section in sections:
            bucket = data.get(section)
            if not isinstance(bucket, dict):
                continue
            for name, spec in bucket.items():
                if not isinstance(name, str) or not isinstance(spec, str):
                    continue
                rows.append(
                    {
                        "ecosystem": "npm",
                        "name": name,
                        "spec": spec,
                        "current_version_hint": _extract_npm_version_hint(spec),
                        "source_file": str(path),
                        "source_rel": str(path.relative_to(repo_root)),
                        "dependency_type": section,
                    }
                )
    return rows


def _parse_pep508_name(req: str) -> str | None:
    req = req.strip()
    if not req or req.startswith("#"):
        return None
    m = re.match(r"^([A-Za-z0-9_.-]+)", req)
    if not m:
        return None
    return m.group(1).lower().replace("_", "-")


def _parse_pep508_spec(req: str) -> str:
    req = req.strip()
    marker_split = req.split(";", 1)[0].strip()
    name_match = re.match(r"^([A-Za-z0-9_.-]+)", marker_split)
    if not name_match:
        return ""
    rest = marker_split[name_match.end() :].strip()
    return rest


def collect_python_dependencies(pyproject_files: list[str], repo_root: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    try:
        import tomllib
    except Exception:
        return rows

    for file in pyproject_files:
        path = Path(file)
        try:
            data = tomllib.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        project = data.get("project") if isinstance(data, dict) else None
        if not isinstance(project, dict):
            continue

        deps = project.get("dependencies")
        if isinstance(deps, list):
            for dep in deps:
                if not isinstance(dep, str):
                    continue
                name = _parse_pep508_name(dep)
                if not name:
                    continue
                rows.append(
                    {
                        "ecosystem": "pypi",
                        "name": name,
                        "spec": _parse_pep508_spec(dep),
                        "current_version_hint": None,
                        "source_file": str(path),
                        "source_rel": str(path.relative_to(repo_root)),
                        "dependency_type": "dependencies",
                    }
                )

        optional_deps = project.get("optional-dependencies")
        if isinstance(optional_deps, dict):
            for group, reqs in optional_deps.items():
                if not isinstance(group, str) or not isinstance(reqs, list):
                    continue
                for dep in reqs:
                    if not isinstance(dep, str):
                        continue
                    name = _parse_pep508_name(dep)
                    if not name:
                        continue
                    rows.append(
                        {
                            "ecosystem": "pypi",
                            "name": name,
                            "spec": _parse_pep508_spec(dep),
                            "current_version_hint": None,
                            "source_file": str(path),
                            "source_rel": str(path.relative_to(repo_root)),
                            "dependency_type": f"optional:{group}",
                        }
                    )

    return rows


def collect_dependencies(repo_context: dict[str, Any]) -> list[dict[str, Any]]:
    repo_root = Path(repo_context["repo_root"])
    deps: list[dict[str, Any]] = []
    deps.extend(collect_js_dependencies(repo_context.get("package_json_files", []), repo_root))
    deps.extend(collect_python_dependencies(repo_context.get("pyproject_files", []), repo_root))
    return deps


def aggregate_dependencies(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    grouped: dict[tuple[str, str], dict[str, Any]] = {}
    for row in rows:
        key = (row["ecosystem"], row["name"])
        target = grouped.setdefault(
            key,
            {
                "ecosystem": row["ecosystem"],
                "name": row["name"],
                "specs": set(),
                "contexts": [],
                "current_version_hint": row.get("current_version_hint"),
            },
        )
        if row.get("spec"):
            target["specs"].add(row["spec"])
        target["contexts"].append(
            {
                "source_file": row.get("source_file"),
                "source_rel": row.get("source_rel"),
                "dependency_type": row.get("dependency_type"),
                "spec": row.get("spec"),
            }
        )
        if not target.get("current_version_hint") and row.get("current_version_hint"):
            target["current_version_hint"] = row["current_version_hint"]

    out: list[dict[str, Any]] = []
    for value in grouped.values():
        value["specs"] = sorted(value["specs"])
        out.append(value)
    out.sort(key=lambda x: (x["ecosystem"], x["name"]))
    return out


def main() -> None:
    import argparse
    from detect_repo import detect_repo_context

    parser = argparse.ArgumentParser(description="Collect dependencies from repo manifests.")
    parser.add_argument("repo", nargs="?", default=".")
    args = parser.parse_args()

    ctx = detect_repo_context(Path(args.repo))
    rows = collect_dependencies(ctx)
    print(json.dumps(aggregate_dependencies(rows), indent=2))


if __name__ == "__main__":
    main()
