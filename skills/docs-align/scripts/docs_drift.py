#!/usr/bin/env python3
"""Collect, compare, and render lightweight docs drift signals for a git repo.

Ships inside the docs-align skill so installs do not depend on external CLIs.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path
from typing import Any


def _ensure_dir(path: Path) -> Path:
    path.mkdir(parents=True, exist_ok=True)
    return path


def _write_json(path: Path, payload: Any) -> Path:
    _ensure_dir(path.parent)
    path.write_text(
        json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    return path


def _run_cmd(
    args: list[str],
    *,
    cwd: Path | None = None,
    check: bool = True,
) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(
        args,
        cwd=str(cwd) if cwd else None,
        env=os.environ.copy(),
        text=True,
        capture_output=True,
        check=False,
    )
    if check and proc.returncode != 0:
        raise RuntimeError(
            f"Command failed ({proc.returncode}): {' '.join(args)}\n"
            f"stdout:\n{proc.stdout}\n"
            f"stderr:\n{proc.stderr}"
        )
    return proc


def _infer_repo_root(start: Path) -> Path:
    current = start.resolve()
    for candidate in [current, *current.parents]:
        if (candidate / ".git").exists():
            return candidate
    return current


def collect(repo: Path) -> dict[str, Any]:
    repo_root = _infer_repo_root(repo)
    proc = _run_cmd(["git", "status", "--porcelain=v1"], cwd=repo_root, check=False)
    changed = []
    for line in proc.stdout.splitlines():
        if len(line) < 4:
            continue
        changed.append(line[3:])
    docs = []
    for candidate in [
        "docs",
        "README.md",
        "AGENTS.md",
        "docs/architecture/adr",
        "docs/architecture/spec",
        "docs/architecture/requirements.md",
    ]:
        path = repo_root / candidate
        if path.exists():
            docs.append(str(path.relative_to(repo_root)))
    return {"repo_root": str(repo_root), "changed_files": changed, "doc_paths": docs}


def compare(payload: dict[str, Any]) -> dict[str, Any]:
    changed = payload.get("changed_files") or []
    docs = payload.get("doc_paths") or []
    changed_docs = [
        path
        for path in changed
        if path.startswith("docs/") or path in {"README.md", "AGENTS.md"}
    ]
    non_doc_changed = [path for path in changed if path not in changed_docs]
    likely_impacts = []
    for path in non_doc_changed:
        normalized = path.lstrip("./")
        if normalized.startswith("docs/") or normalized in {"README.md", "AGENTS.md"}:
            continue
        if normalized.startswith("convex/") or "/convex/" in normalized:
            likely_impacts.extend(
                [doc for doc in docs if "architecture" in doc or "AGENTS" in doc]
            )
        elif (
            normalized.startswith("app/")
            or normalized.startswith("src/")
            or "/app/" in normalized
            or "/src/" in normalized
        ):
            likely_impacts.extend(
                [
                    doc
                    for doc in docs
                    if doc.startswith("README")
                    or "spec" in doc
                    or "requirements" in doc
                ]
            )
        else:
            likely_impacts.extend(
                [
                    doc
                    for doc in docs
                    if doc.startswith("README")
                    or "spec" in doc
                    or "requirements" in doc
                    or "AGENTS" in doc
                ]
            )
    unique_impacts = sorted(set(likely_impacts))
    status = "aligned" if not non_doc_changed else "needs-review"
    return {
        "repo_root": payload.get("repo_root"),
        "status": status,
        "changed_files": changed,
        "changed_docs": changed_docs,
        "non_doc_changed": non_doc_changed,
        "likely_impacted_docs": unique_impacts,
        "missing_doc_work": status == "needs-review",
    }


def render(payload: dict[str, Any], fmt: str) -> str:
    if fmt == "json":
        return json.dumps(payload, indent=2, ensure_ascii=False)
    lines = [
        f"# Docs Drift Check: {payload.get('repo_root')}",
        "",
        f"- Status: {payload.get('status')}",
        f"- Changed files: {len(payload.get('changed_files') or [])}",
        f"- Changed docs: {len(payload.get('changed_docs') or [])}",
        "",
        "## Likely Impacted Docs",
    ]
    impacted = payload.get("likely_impacted_docs") or []
    if impacted:
        for doc in impacted:
            lines.append(f"- {doc}")
    else:
        lines.append("- none inferred")
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Collect and compare likely docs drift."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    collect_cmd = subparsers.add_parser("collect")
    collect_cmd.add_argument("--cwd", required=True, type=Path)
    collect_cmd.add_argument("--out", required=True, type=Path)

    compare_cmd = subparsers.add_parser("compare")
    compare_cmd.add_argument("--input", required=True, type=Path)
    compare_cmd.add_argument("--out", required=True, type=Path)

    render_cmd = subparsers.add_parser("render")
    render_cmd.add_argument("--input", required=True, type=Path)
    render_cmd.add_argument("--format", required=True, choices=["md", "json"])

    args = parser.parse_args()
    if args.command == "collect":
        payload = collect(args.cwd)
        _write_json(args.out, payload)
        print(str(args.out))
        return 0
    if args.command == "compare":
        payload = compare(json.loads(args.input.read_text(encoding="utf-8")))
        _write_json(args.out, payload)
        print(str(args.out))
        return 0
    print(render(json.loads(args.input.read_text(encoding="utf-8")), args.format))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
