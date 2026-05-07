#!/usr/bin/env python3
"""Check relative Markdown links in tracked documentation."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

LINK_RE = re.compile(r"\[[^\]]+\]\(([^)]+)\)")


def should_skip(target: str) -> bool:
    return (
        "://" in target
        or target.startswith("#")
        or target.startswith("mailto:")
        or target.startswith("tel:")
    )


def check_paths(paths: list[Path], repo_root: Path) -> list[str]:
    errors: list[str] = []
    for base in paths:
        files = sorted(base.rglob("*.md")) if base.is_dir() else [base]
        for path in files:
            if not path.exists() or path.suffix != ".md":
                continue
            text = path.read_text(encoding="utf-8")
            for target in LINK_RE.findall(text):
                if should_skip(target):
                    continue
                target_path = target.split("#", 1)[0]
                if not target_path:
                    continue
                resolved = (path.parent / target_path).resolve()
                try:
                    resolved.relative_to(repo_root)
                except ValueError:
                    errors.append(f"{path}: link target escapes repo root {target}")
                    continue
                if not resolved.exists():
                    errors.append(f"{path}: missing link target {target}")
    return errors


def discover_repo_root() -> Path:
    for parent in Path(__file__).resolve().parents:
        if (parent / ".git").exists() or (
            (parent / "AGENTS.md").is_file() and (parent / "skills").is_dir()
        ):
            return parent
    return Path.cwd().resolve()


def normalize_paths(paths: list[Path], repo_root: Path) -> list[Path]:
    return [(path if path.is_absolute() else repo_root / path).resolve() for path in paths]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", type=Path, default=[Path("docs")])
    args = parser.parse_args()

    repo_root = discover_repo_root()
    errors = check_paths(normalize_paths(args.paths, repo_root), repo_root)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    print("docs-links-ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
