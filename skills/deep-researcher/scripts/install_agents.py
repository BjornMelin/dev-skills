#!/usr/bin/env python3
"""Install deep-researcher subagent templates into Codex agent directories."""

from __future__ import annotations

import argparse
import shutil
from pathlib import Path


def target_dir(target: str, project_dir: Path) -> Path:
    if target == "global":
        return Path.home() / ".codex" / "agents"
    if target == "project":
        return project_dir / ".codex" / "agents"
    raise ValueError(f"unknown target: {target}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", choices=["global", "project"], default="project")
    parser.add_argument("--project-dir", type=Path, default=Path.cwd())
    parser.add_argument("--dest", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()

    skill_dir = Path(__file__).resolve().parents[1]
    templates = skill_dir / "templates" / "agents"
    dest = args.dest or target_dir(args.target, args.project_dir.resolve())

    installed = []
    for src in sorted(templates.glob("*.toml")):
        dst = dest / src.name
        if dst.exists() and not args.overwrite:
            print(f"skip {dst} (exists; pass --overwrite)")
            continue
        installed.append((src, dst))
        if not args.dry_run:
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
        print(f"{'would install' if args.dry_run else 'installed'} {src.name} -> {dst}")

    if not installed:
        print("no templates installed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
