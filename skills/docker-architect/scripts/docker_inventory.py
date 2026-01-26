from __future__ import annotations

import argparse
import json
import os
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable


@dataclass(frozen=True)
class Inventory:
    root: str
    stacks: list[str]
    dockerfiles: list[str]
    compose_files: list[str]
    has_dockerignore: bool
    suggested_templates: list[str]


IGNORE_DIR_NAMES = {
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "venv",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "dist",
    "build",
    ".tox",
    ".idea",
    ".vscode",
    "opensrc",
    "vendor",
    "third_party",
    "external",
}


def _walk_files(root: Path) -> Iterable[Path]:
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in IGNORE_DIR_NAMES]
        base = Path(dirpath)
        for filename in filenames:
            yield base / filename


def _is_dockerfile_name(name: str) -> bool:
    if name in {"Dockerfile", "Containerfile"}:
        return True
    if name.startswith("Dockerfile.") or name.startswith("Containerfile."):
        return True
    return False


def _is_compose_name(name: str) -> bool:
    lower = name.lower()
    if lower in {"compose.yml", "compose.yaml", "docker-compose.yml", "docker-compose.yaml"}:
        return True
    if lower.startswith("docker-compose.") and (lower.endswith(".yml") or lower.endswith(".yaml")):
        return True
    return False


def _detect_stacks(files_by_name: set[str]) -> list[str]:
    stacks: list[str] = []

    python_markers = {
        "pyproject.toml",
        "requirements.txt",
        "requirements-dev.txt",
        "requirements-prod.txt",
        "poetry.lock",
        "uv.lock",
        "pipfile",
        "pipfile.lock",
    }
    node_markers = {
        "package.json",
        "pnpm-lock.yaml",
        "yarn.lock",
        "package-lock.json",
        "npm-shrinkwrap.json",
    }
    go_markers = {"go.mod", "go.sum"}
    rust_markers = {"cargo.toml", "cargo.lock"}
    dotnet_markers = {".csproj", ".fsproj", ".vbproj"}
    java_markers = {"pom.xml", "build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"}

    if any(m in files_by_name for m in python_markers):
        stacks.append("python")
    if any(m in files_by_name for m in node_markers):
        stacks.append("node")
    if any(m in files_by_name for m in go_markers):
        stacks.append("go")
    if any(m in files_by_name for m in rust_markers):
        stacks.append("rust")
    if any(m in files_by_name for m in java_markers):
        stacks.append("java")
    if any(name.endswith(tuple(dotnet_markers)) for name in files_by_name):
        stacks.append("dotnet")

    return stacks


def _suggest_templates(stacks: list[str], files_by_name: set[str]) -> list[str]:
    suggestions: list[str] = []

    if "python" in stacks:
        if "uv.lock" in files_by_name or "pyproject.toml" in files_by_name:
            suggestions.append("python/Dockerfile.uv")
        if any(name.startswith("requirements") and name.endswith(".txt") for name in files_by_name):
            suggestions.append("python/Dockerfile.pip")

    if "node" in stacks:
        if "pnpm-lock.yaml" in files_by_name:
            suggestions.append("node/Dockerfile.pnpm")
        else:
            suggestions.append("node/Dockerfile.npm")

    if "go" in stacks:
        suggestions.append("go/Dockerfile")

    suggestions.append(".dockerignore")
    suggestions.append("compose/docker-compose.yml")
    suggestions.append("compose/docker-compose.dev.yml")
    suggestions.append("compose/docker-compose.prod.yml")
    suggestions.append("compose/docker-compose.deps.yml")
    suggestions.append("ci/github-actions-docker-ci.yml")
    suggestions.append("ci/github-actions-docker-publish.yml")
    suggestions.append("docker-bake.hcl")

    # De-duplicate while preserving order
    seen: set[str] = set()
    out: list[str] = []
    for s in suggestions:
        if s not in seen:
            out.append(s)
            seen.add(s)
    return out


def build_inventory(root: Path) -> Inventory:
    root = root.resolve()
    files = list(_walk_files(root))
    files_by_name = {p.name.lower() for p in files}

    dockerfiles = sorted(
        str(p.relative_to(root))
        for p in files
        if _is_dockerfile_name(p.name) and p.is_file()
    )
    compose_files = sorted(
        str(p.relative_to(root))
        for p in files
        if _is_compose_name(p.name) and p.is_file()
    )

    stacks = _detect_stacks(files_by_name)
    suggested_templates = _suggest_templates(stacks, files_by_name)

    return Inventory(
        root=str(root),
        stacks=stacks,
        dockerfiles=dockerfiles,
        compose_files=compose_files,
        has_dockerignore=(root / ".dockerignore").exists(),
        suggested_templates=suggested_templates,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Inventory a repo for Docker/Compose work.")
    parser.add_argument("--root", type=Path, default=Path("."), help="Repo root (default: .)")
    parser.add_argument("--json", action="store_true", help="Output JSON")
    args = parser.parse_args()

    inventory = build_inventory(args.root)

    if args.json:
        print(json.dumps(asdict(inventory), indent=2, sort_keys=True))
        return 0

    print("Docker Architect · Inventory")
    print(f"- Root: {inventory.root}")
    print(f"- Stacks: {', '.join(inventory.stacks) if inventory.stacks else '(unknown)'}")
    print(f"- Dockerfiles: {', '.join(inventory.dockerfiles) if inventory.dockerfiles else '(none)'}")
    print(f"- Compose: {', '.join(inventory.compose_files) if inventory.compose_files else '(none)'}")
    print(f"- .dockerignore: {'yes' if inventory.has_dockerignore else 'no'}")
    print("- Suggested templates:")
    for t in inventory.suggested_templates:
        print(f"  - assets/templates/{t}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
