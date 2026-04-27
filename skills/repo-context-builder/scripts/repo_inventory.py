#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

IGNORE_DIRS = {
    ".git",
    ".hg",
    ".svn",
    ".idea",
    ".vscode",
    ".venv",
    "venv",
    "node_modules",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
    "coverage",
    "htmlcov",
    "target",
    "out",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".pnpm-store",
}

SPECIAL_FILENAMES = {
    "Dockerfile": "docker",
    "docker-compose.yml": "compose",
    "docker-compose.yaml": "compose",
    "Makefile": "make",
    "justfile": "just",
    "Procfile": "procfile",
}

EXTENSION_LANG = {
    ".py": "Python",
    ".pyi": "Python",
    ".ts": "TypeScript",
    ".tsx": "TypeScript",
    ".js": "JavaScript",
    ".jsx": "JavaScript",
    ".mjs": "JavaScript",
    ".cjs": "JavaScript",
    ".json": "JSON",
    ".jsonc": "JSON",
    ".toml": "TOML",
    ".yaml": "YAML",
    ".yml": "YAML",
    ".md": "Markdown",
    ".sql": "SQL",
    ".sh": "Shell",
    ".bash": "Shell",
    ".zsh": "Shell",
    ".ps1": "PowerShell",
    ".go": "Go",
    ".rs": "Rust",
    ".java": "Java",
    ".kt": "Kotlin",
    ".swift": "Swift",
    ".rb": "Ruby",
    ".php": "PHP",
    ".c": "C",
    ".h": "C",
    ".cpp": "C++",
    ".hpp": "C++",
    ".cs": "C#",
    ".tf": "Terraform",
    ".tfvars": "Terraform",
}

KEY_FILE_PATTERNS = {
    "readme": ["README", "README.md", "README.rst"],
    "agents": ["AGENTS.md", "AGENTS.override.md"],
    "python": ["pyproject.toml", "uv.lock", "requirements.txt", "requirements-dev.txt", "setup.py"],
    "javascript": ["package.json", "pnpm-lock.yaml", "package-lock.json", "bun.lock", "bun.lockb", "turbo.json", "nx.json"],
    "rust": ["Cargo.toml", "Cargo.lock"],
    "go": ["go.mod", "go.sum"],
    "docker": ["Dockerfile", "docker-compose.yml", "docker-compose.yaml"],
    "ci": [".github/workflows"],
    "infra": ["template.yaml", "serverless.yml", "serverless.yaml", "cdk.json", "vercel.json", "fly.toml", "render.yaml", "render.yml"],
}

ENTRYPOINT_HINTS = {
    "main.py",
    "app.py",
    "server.py",
    "manage.py",
    "asgi.py",
    "wsgi.py",
    "cli.py",
    "index.ts",
    "index.tsx",
    "index.js",
    "main.ts",
    "main.tsx",
    "main.js",
}


@dataclass
class Inventory:
    root: Path
    total_files: int
    total_dirs: int
    languages: dict[str, int]
    top_level_dirs: list[str]
    key_files: dict[str, list[str]]
    entrypoints: list[str]
    tests: list[str]
    workflows: list[str]


def should_skip_dir(dirname: str) -> bool:
    return dirname in IGNORE_DIRS or dirname.startswith(".ruff_cache")


def detect_language(path: Path) -> str | None:
    if path.name in SPECIAL_FILENAMES:
        return SPECIAL_FILENAMES[path.name]
    return EXTENSION_LANG.get(path.suffix.lower())


def walk_repo(root: Path) -> tuple[list[Path], list[Path]]:
    dirs: list[Path] = []
    files: list[Path] = []
    for current, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if not should_skip_dir(d)]
        current_path = Path(current)
        dirs.extend(current_path / d for d in dirnames)
        files.extend(current_path / f for f in filenames)
    return dirs, files


def relative_paths(paths: Iterable[Path], root: Path) -> list[str]:
    return sorted(str(p.relative_to(root)).replace("\\", "/") for p in paths)


def collect_key_files(files: list[Path], root: Path) -> dict[str, list[str]]:
    rels = relative_paths(files, root)
    found: dict[str, list[str]] = defaultdict(list)
    for rel in rels:
        name = Path(rel).name
        for category, patterns in KEY_FILE_PATTERNS.items():
            for pattern in patterns:
                if rel == pattern or name == pattern or rel.startswith(f"{pattern}/"):
                    found[category].append(rel)
    return {k: sorted(v) for k, v in found.items()}


def collect_entrypoints(files: list[Path], root: Path) -> list[str]:
    rels = relative_paths(files, root)
    matches: list[str] = []
    for rel in rels:
        path = Path(rel)
        if path.name in ENTRYPOINT_HINTS:
            matches.append(rel)
            continue
        lowered = rel.lower()
        if any(token in lowered for token in ["/api/", "/cli/", "/worker", "/jobs/", "/cmd/"]):
            if path.suffix.lower() in {".py", ".ts", ".tsx", ".js", ".go", ".rs"}:
                matches.append(rel)
    return sorted(set(matches))[:40]


def collect_tests(files: list[Path], root: Path) -> list[str]:
    rels = relative_paths(files, root)
    test_files = [
        rel
        for rel in rels
        if "/tests/" in f"/{rel}" or Path(rel).name.startswith("test_") or Path(rel).name.endswith(".test.ts") or Path(rel).name.endswith(".spec.ts") or Path(rel).name.endswith(".spec.tsx") or Path(rel).name.endswith(".test.tsx") or Path(rel).name.endswith(".test.js") or Path(rel).name.endswith(".spec.js")
    ]
    return sorted(test_files)[:60]


def collect_workflows(files: list[Path], root: Path) -> list[str]:
    rels = relative_paths(files, root)
    workflows = [rel for rel in rels if rel.startswith('.github/workflows/')]
    return sorted(workflows)


def make_inventory(root: Path) -> Inventory:
    dirs, files = walk_repo(root)
    lang_counter: Counter[str] = Counter()
    for file in files:
        language = detect_language(file)
        if language:
            lang_counter[language] += 1

    top_level_dirs = sorted(
        p.name for p in root.iterdir() if p.is_dir() and not should_skip_dir(p.name)
    )

    return Inventory(
        root=root,
        total_files=len(files),
        total_dirs=len(dirs),
        languages=dict(lang_counter.most_common()),
        top_level_dirs=top_level_dirs,
        key_files=collect_key_files(files, root),
        entrypoints=collect_entrypoints(files, root),
        tests=collect_tests(files, root),
        workflows=collect_workflows(files, root),
    )


def render_markdown(inventory: Inventory) -> str:
    lines: list[str] = []
    lines.append("# Repository inventory")
    lines.append("")
    lines.append(f"- Root: `{inventory.root}`")
    lines.append(f"- Total files scanned: {inventory.total_files}")
    lines.append(f"- Total directories scanned: {inventory.total_dirs}")
    lines.append("")

    lines.append("## Languages by file count")
    lines.append("")
    lines.append("| Language | Files |")
    lines.append("| --- | ---: |")
    for language, count in inventory.languages.items():
        lines.append(f"| {language} | {count} |")
    if not inventory.languages:
        lines.append("| Unknown | 0 |")
    lines.append("")

    lines.append("## Top-level directories")
    lines.append("")
    for dirname in inventory.top_level_dirs[:50]:
        lines.append(f"- `{dirname}/`")
    if not inventory.top_level_dirs:
        lines.append("- None")
    lines.append("")

    lines.append("## Key files by category")
    lines.append("")
    for category, paths in sorted(inventory.key_files.items()):
        lines.append(f"### {category}")
        lines.append("")
        for rel in paths[:40]:
            lines.append(f"- `{rel}`")
        if not paths:
            lines.append("- None")
        lines.append("")
    if not inventory.key_files:
        lines.append("No key files matched the built-in heuristics.")
        lines.append("")

    lines.append("## Entrypoint candidates")
    lines.append("")
    for rel in inventory.entrypoints:
        lines.append(f"- `{rel}`")
    if not inventory.entrypoints:
        lines.append("- None found by heuristic")
    lines.append("")

    lines.append("## Test files")
    lines.append("")
    for rel in inventory.tests:
        lines.append(f"- `{rel}`")
    if not inventory.tests:
        lines.append("- None found by heuristic")
    lines.append("")

    lines.append("## CI workflows")
    lines.append("")
    for rel in inventory.workflows:
        lines.append(f"- `{rel}`")
    if not inventory.workflows:
        lines.append("- None found")
    lines.append("")

    return "\n".join(lines)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate a fast repository inventory for repo analysis.")
    parser.add_argument("--root", default=".", help="Repository root to scan. Defaults to current directory.")
    parser.add_argument("--format", choices=["markdown", "json"], default="markdown")
    parser.add_argument("--out", default="", help="Optional output file path.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    inventory = make_inventory(root)

    if args.format == "json":
        payload = json.dumps(inventory.__dict__, indent=2, default=str)
    else:
        payload = render_markdown(inventory)

    if args.out:
        output_path = Path(args.out)
        output_path.write_text(payload, encoding="utf-8")
    else:
        print(payload)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
