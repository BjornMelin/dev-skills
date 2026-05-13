#!/usr/bin/env python3
"""Repository and runtime detection for gh-deps-intel."""

from __future__ import annotations

import glob
import json
import os
from pathlib import Path
from typing import Any

from utils import run_cmd
from utils import extract_node_major

IGNORED_DIRS = {
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".next",
    ".turbo",
    ".venv",
    "venv",
    "dist",
    "build",
    "coverage",
    "tmp",
    "out",
    ".cache",
}


def _read_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {}


def _read_text(path: Path) -> str | None:
    if not path.exists():
        return None
    txt = path.read_text(encoding="utf-8", errors="ignore").strip()
    return txt or None


def _workspace_globs_from_package_json(pkg: dict[str, Any]) -> list[str]:
    ws = pkg.get("workspaces")
    if isinstance(ws, list):
        return [x for x in ws if isinstance(x, str)]
    if isinstance(ws, dict):
        packages = ws.get("packages")
        if isinstance(packages, list):
            return [x for x in packages if isinstance(x, str)]
    return []


def _workspace_globs_from_pnpm(path: Path) -> list[str]:
    file = path / "pnpm-workspace.yaml"
    if not file.exists():
        return []
    globs: list[str] = []
    in_packages = False
    for line in file.read_text(encoding="utf-8", errors="ignore").splitlines():
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        if s.startswith("packages:"):
            in_packages = True
            continue
        if in_packages and s.startswith("-"):
            item = s[1:].strip().strip("'\"")
            if item:
                globs.append(item)
        elif in_packages and not s.startswith("-"):
            # Exit package list when indentation/shape changes.
            in_packages = False
    return globs


def _expand_workspace_globs(root: Path, globs_in: list[str]) -> list[Path]:
    ignored = _load_git_ignored_entries(root)
    paths: set[Path] = set()
    for pattern in globs_in:
        pat = pattern.rstrip("/") + "/package.json"
        for match in glob.glob(str(root / pat), recursive=True):
            p = Path(match).resolve()
            if p.is_file() and not _is_git_ignored(root, p, ignored):
                paths.add(p)
    return sorted(paths)


def _recursive_package_scan(root: Path) -> list[Path]:
    ignored = _load_git_ignored_entries(root)
    found: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        current = Path(dirpath)
        if _is_git_ignored(root, current, ignored):
            dirnames[:] = []
            continue
        keep_dirs = []
        for d in dirnames:
            if d in IGNORED_DIRS or d.startswith("."):
                continue
            candidate = current / d
            if _is_git_ignored(root, candidate, ignored):
                continue
            keep_dirs.append(d)
        dirnames[:] = keep_dirs
        if "package.json" in filenames:
            pkg = Path(dirpath) / "package.json"
            if not _is_git_ignored(root, pkg, ignored):
                found.append(pkg)
    return sorted(p.resolve() for p in found)


def _recursive_pyproject_scan(root: Path) -> list[Path]:
    ignored = _load_git_ignored_entries(root)
    found: list[Path] = []
    for dirpath, dirnames, filenames in os.walk(root):
        current = Path(dirpath)
        if _is_git_ignored(root, current, ignored):
            dirnames[:] = []
            continue
        keep_dirs = []
        for d in dirnames:
            if d in IGNORED_DIRS or d.startswith("."):
                continue
            candidate = current / d
            if _is_git_ignored(root, candidate, ignored):
                continue
            keep_dirs.append(d)
        dirnames[:] = keep_dirs
        if "pyproject.toml" in filenames:
            pyproject = Path(dirpath) / "pyproject.toml"
            if not _is_git_ignored(root, pyproject, ignored):
                found.append(pyproject)
    return sorted(p.resolve() for p in found)


def _is_git_repo(root: Path) -> bool:
    proc = run_cmd(["git", "-C", str(root), "rev-parse", "--is-inside-work-tree"], check=False)
    return proc.returncode == 0


def _load_git_ignored_entries(root: Path) -> list[str]:
    """Return ignored paths from .gitignore as normalized relative entries."""
    if not _is_git_repo(root):
        return []
    proc = run_cmd(
        ["git", "-C", str(root), "ls-files", "--ignored", "--exclude-standard", "--others", "--directory"],
        check=False,
    )
    if proc.returncode != 0:
        return []
    entries: list[str] = []
    for raw in proc.stdout.splitlines():
        line = raw.strip()
        if not line:
            continue
        normalized = line.replace("\\", "/").lstrip("./")
        entries.append(normalized)
    return entries


def _is_git_ignored(root: Path, path: Path, ignored_entries: list[str]) -> bool:
    if not ignored_entries:
        return False
    try:
        rel = path.resolve().relative_to(root.resolve()).as_posix().lstrip("./")
    except Exception:
        return False
    for entry in ignored_entries:
        if entry.endswith("/"):
            prefix = entry[:-1]
            if rel == prefix or rel.startswith(entry):
                return True
        else:
            if rel == entry:
                return True
    return False


def _detect_node_manager(root: Path, root_pkg: dict[str, Any]) -> str:
    package_manager = str(root_pkg.get("packageManager") or "")
    if package_manager.startswith("bun@") or (root / "bun.lock").exists() or (root / "bun.lockb").exists():
        return "bun"
    if package_manager.startswith("pnpm@") or (root / "pnpm-lock.yaml").exists():
        return "pnpm"
    if package_manager.startswith("yarn@") or (root / "yarn.lock").exists():
        return "yarn"
    if package_manager.startswith("npm@") or (root / "package-lock.json").exists():
        return "npm"
    return "npm"


def _detect_python_manager(root: Path) -> str:
    if (root / "uv.lock").exists() or (root / "uv.toml").exists():
        return "uv"
    if (root / "poetry.lock").exists():
        return "poetry"
    if (root / "requirements.txt").exists():
        return "pip"
    return "uv"


def _detect_node_runtime(root: Path, root_pkg: dict[str, Any]) -> dict[str, Any]:
    hints: list[dict[str, str]] = []
    engines = root_pkg.get("engines") if isinstance(root_pkg, dict) else None
    if isinstance(engines, dict) and isinstance(engines.get("node"), str):
        hints.append({"source": "package.json#engines.node", "value": engines["node"]})

    for name in [".nvmrc", ".node-version"]:
        txt = _read_text(root / name)
        if txt:
            hints.append({"source": name, "value": txt})

    tool_versions = _read_text(root / ".tool-versions")
    if tool_versions:
        for line in tool_versions.splitlines():
            s = line.strip()
            if s.startswith("nodejs "):
                hints.append({"source": ".tool-versions", "value": s.split(" ", 1)[1].strip()})

    volta = root_pkg.get("volta") if isinstance(root_pkg, dict) else None
    if isinstance(volta, dict) and isinstance(volta.get("node"), str):
        hints.append({"source": "package.json#volta.node", "value": volta["node"]})

    selected = hints[0]["value"] if hints else None
    major = extract_node_major(selected)
    return {"detected": selected, "major": major, "hints": hints}


def _detect_python_runtime(root: Path, pyproject_files: list[Path]) -> dict[str, Any]:
    hints: list[dict[str, str]] = []

    pyver = _read_text(root / ".python-version")
    if pyver:
        hints.append({"source": ".python-version", "value": pyver})

    for pp in pyproject_files:
        try:
            import tomllib

            data = tomllib.loads(pp.read_text(encoding="utf-8"))
            project = data.get("project") if isinstance(data, dict) else None
            if isinstance(project, dict) and isinstance(project.get("requires-python"), str):
                rel = str(pp.relative_to(root))
                hints.append({"source": f"{rel}#project.requires-python", "value": project["requires-python"]})
        except Exception:
            continue

    selected = hints[0]["value"] if hints else None
    return {"detected": selected, "hints": hints}


def detect_repo_context(repo_root: Path) -> dict[str, Any]:
    repo_root = repo_root.resolve()
    root_pkg_path = repo_root / "package.json"
    root_pkg = _read_json(root_pkg_path) if root_pkg_path.exists() else {}

    declared_globs = _workspace_globs_from_package_json(root_pkg) + _workspace_globs_from_pnpm(repo_root)
    workspace_pkgs = _expand_workspace_globs(repo_root, declared_globs)

    recursive_pkgs = _recursive_package_scan(repo_root)
    package_json_files: list[Path] = []
    seen: set[Path] = set()
    for p in [root_pkg_path] + workspace_pkgs + recursive_pkgs:
        if p and p.exists() and p not in seen:
            seen.add(p)
            package_json_files.append(p.resolve())

    pyproject_files = _recursive_pyproject_scan(repo_root)

    has_node = len(package_json_files) > 0
    has_python = len(pyproject_files) > 0

    ctx = {
        "repo_root": str(repo_root),
        "has_node": has_node,
        "has_python": has_python,
        "node_manager": _detect_node_manager(repo_root, root_pkg) if has_node else None,
        "python_manager": _detect_python_manager(repo_root) if has_python else None,
        "package_json_files": [str(p) for p in package_json_files],
        "pyproject_files": [str(p) for p in pyproject_files],
        "is_monorepo": len(package_json_files) > 1 or len(pyproject_files) > 1,
        "workspace_globs": declared_globs,
        "node_runtime": _detect_node_runtime(repo_root, root_pkg) if has_node else {"detected": None, "major": None, "hints": []},
        "python_runtime": _detect_python_runtime(repo_root, pyproject_files) if has_python else {"detected": None, "hints": []},
    }
    return ctx


def main() -> None:
    import argparse

    parser = argparse.ArgumentParser(description="Detect repository context.")
    parser.add_argument("repo", nargs="?", default=".")
    args = parser.parse_args()

    ctx = detect_repo_context(Path(args.repo))
    print(json.dumps(ctx, indent=2))


if __name__ == "__main__":
    main()
