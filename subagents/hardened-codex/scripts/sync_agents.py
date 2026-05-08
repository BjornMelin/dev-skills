#!/usr/bin/env python3
"""Install the hardened Codex subagent catalog with backups."""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = ROOT.parents[1]
AGENTS_ROOT = ROOT / "agents"
VALIDATOR = REPO_ROOT / "skills" / "subagent-creator" / "scripts" / "subagent_creator.py"

PUBLIC_OVERLAY_TARGETS = {
    "docmind": Path.home() / "repos" / "agents" / "docmind-ai-llm",
    "tooling": Path.home() / "repos" / "agents" / "dev-skills",
}


@dataclass(frozen=True)
class CopyAction:
    source: Path
    target: Path
    action: str
    backup: Path | None = None


def stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def ensure_source_exists(source: Path) -> None:
    if not source.exists():
        raise SystemExit(f"source does not exist: {source}")
    if not any(source.glob("*.toml")):
        raise SystemExit(f"source has no TOML files: {source}")


def copy_tree(source: Path, target: Path, *, dry_run: bool, backup_dir: Path) -> list[CopyAction]:
    ensure_source_exists(source)
    actions: list[CopyAction] = []
    for src in sorted(source.glob("*.toml")):
        dst = target / src.name
        backup: Path | None = None
        if dst.exists():
            backup = backup_dir / dst.name
            action = "would_overwrite" if dry_run else "overwritten"
        else:
            action = "would_install" if dry_run else "installed"
        actions.append(CopyAction(src, dst, action, backup))
        if dry_run:
            continue
        target.mkdir(parents=True, exist_ok=True)
        if backup is not None:
            backup_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(dst, backup)
        shutil.copy2(src, dst)
    return actions


def validate(path: Path) -> int:
    # Validate only live role files. Backup directories intentionally preserve
    # older role contracts and may not satisfy the current validator.
    files = sorted(path.glob("*.toml")) if path.is_dir() else [path]
    command = [sys.executable, str(VALIDATOR), "validate", *[str(file) for file in files]]
    return subprocess.run(command, check=False).returncode


def print_actions(label: str, actions: list[CopyAction]) -> None:
    print(f"## {label}")
    for action in actions:
        backup_note = f" backup={action.backup}" if action.backup else ""
        print(f"{action.action}: {action.source.name} -> {action.target}{backup_note}")


def install_global(*, dry_run: bool) -> int:
    source = AGENTS_ROOT / "global"
    target = Path.home() / ".codex" / "agents"
    backup_dir = target.parent / "agent-backups" / f"global-{stamp()}"
    actions = copy_tree(source, target, dry_run=dry_run, backup_dir=backup_dir)
    print_actions("global", actions)
    if dry_run:
        return 0
    return validate(target)


def install_overlay(name: str, *, dry_run: bool, project_dir: Path | None = None) -> int:
    source = AGENTS_ROOT / "overlays" / name
    if project_dir is None and name not in PUBLIC_OVERLAY_TARGETS:
        available = ", ".join(sorted(PUBLIC_OVERLAY_TARGETS))
        raise SystemExit(
            f"unknown public overlay {name}; available: {available}; "
            "pass --project-dir for local-only overlays"
        )
    project = (project_dir or PUBLIC_OVERLAY_TARGETS[name]).expanduser().resolve()
    target = project / ".codex" / "agents"
    backup_dir = target.parent / "agent-backups" / f"{name}-{stamp()}"
    actions = copy_tree(source, target, dry_run=dry_run, backup_dir=backup_dir)
    print_actions(name, actions)
    if dry_run:
        return 0
    return validate(target)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--global", dest="install_global", action="store_true")
    parser.add_argument("--overlay", action="append", default=[])
    parser.add_argument("--all-overlays", action="store_true")
    parser.add_argument("--project-dir", type=Path)
    parser.add_argument("--dry-run", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    overlays = list(args.overlay)
    if args.all_overlays:
        overlays = sorted(set([*overlays, *PUBLIC_OVERLAY_TARGETS]))
    if not args.install_global and not overlays:
        raise SystemExit("select --global, --overlay <name>, or --all-overlays")

    status = 0
    if args.install_global:
        status |= install_global(dry_run=args.dry_run)
    for overlay in overlays:
        project_dir = args.project_dir if len(overlays) == 1 else None
        status |= install_overlay(overlay, dry_run=args.dry_run, project_dir=project_dir)
    return status


if __name__ == "__main__":
    raise SystemExit(main())
