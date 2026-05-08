#!/usr/bin/env python3
"""Install the hardened Codex subagent catalog with backups."""

from __future__ import annotations

import argparse
import json
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
DEFAULT_LOCAL_MANIFEST = ROOT / "overlays.local.json"

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


@dataclass(frozen=True)
class OverlayTarget:
    project_dir: Path
    source_dir: Path | None = None


def stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def resolve_path(value: str, *, base: Path) -> Path:
    path = Path(value).expanduser()
    if not path.is_absolute():
        path = base / path
    return path.resolve()


def load_local_targets(manifest: Path) -> dict[str, OverlayTarget]:
    if not manifest.exists():
        return {}
    try:
        payload = json.loads(manifest.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid local overlay manifest {manifest}: {exc}") from exc
    overlays = payload.get("overlays")
    if not isinstance(overlays, dict):
        raise SystemExit(f"local overlay manifest must contain an object at overlays: {manifest}")

    targets: dict[str, OverlayTarget] = {}
    for name, config in sorted(overlays.items()):
        if not isinstance(name, str) or not name:
            raise SystemExit(f"local overlay names must be non-empty strings: {manifest}")
        if not isinstance(config, dict):
            raise SystemExit(f"local overlay {name} must be an object: {manifest}")
        project_dir = config.get("project_dir")
        if not isinstance(project_dir, str) or not project_dir:
            raise SystemExit(f"local overlay {name} requires project_dir: {manifest}")
        source_dir_value = config.get("source_dir")
        if source_dir_value is not None and not isinstance(source_dir_value, str):
            raise SystemExit(f"local overlay {name} source_dir must be a string: {manifest}")
        targets[name] = OverlayTarget(
            project_dir=resolve_path(project_dir, base=manifest.parent),
            source_dir=(
                resolve_path(source_dir_value, base=manifest.parent)
                if source_dir_value
                else None
            ),
        )
    return targets


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


def validate_sources(label: str, source: Path) -> int:
    ensure_source_exists(source)
    print(f"## validate {label}")
    status = validate(source)
    print(f"{'ok' if status == 0 else 'failed'}: {source}")
    return status


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
    return install_overlay_with_targets(
        name,
        dry_run=dry_run,
        project_dir=project_dir,
        local_targets={},
    )


def install_overlay_with_targets(
    name: str,
    *,
    dry_run: bool,
    project_dir: Path | None,
    local_targets: dict[str, OverlayTarget],
) -> int:
    local_target = local_targets.get(name)
    source = local_target.source_dir if local_target and local_target.source_dir else AGENTS_ROOT / "overlays" / name
    if project_dir is None and name not in PUBLIC_OVERLAY_TARGETS and local_target is None:
        available = ", ".join(sorted(PUBLIC_OVERLAY_TARGETS))
        raise SystemExit(
            f"unknown public overlay {name}; available: {available}; "
            "pass --project-dir or add it to the local overlay manifest"
        )
    default_project = local_target.project_dir if local_target else PUBLIC_OVERLAY_TARGETS.get(name)
    if project_dir is None and default_project is None:
        raise SystemExit(f"overlay {name} requires --project-dir")
    project = (project_dir or default_project).expanduser().resolve()
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
    parser.add_argument("--all-local-overlays", action="store_true")
    parser.add_argument("--local-manifest", type=Path, default=DEFAULT_LOCAL_MANIFEST)
    parser.add_argument("--project-dir", type=Path)
    parser.add_argument("--validate-sources", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    local_targets = load_local_targets(args.local_manifest.expanduser().resolve())
    overlays = list(args.overlay)
    if args.all_overlays:
        overlays = sorted(set([*overlays, *PUBLIC_OVERLAY_TARGETS]))
    if args.all_local_overlays:
        overlays = sorted(set([*overlays, *local_targets]))
    if args.list:
        print("public overlays:")
        for name, target in sorted(PUBLIC_OVERLAY_TARGETS.items()):
            print(f"- {name}: {target}")
        print("local overlays:")
        for name, target in sorted(local_targets.items()):
            source = target.source_dir or AGENTS_ROOT / "overlays" / name
            print(f"- {name}: {target.project_dir} source={source}")
        return 0
    if not args.install_global and not overlays:
        raise SystemExit(
            "select --global, --overlay <name>, --all-overlays, "
            "--all-local-overlays, or --list"
        )

    status = 0
    if args.install_global:
        if args.validate_sources:
            status |= validate_sources("global", AGENTS_ROOT / "global")
        else:
            status |= install_global(dry_run=args.dry_run)
    for overlay in overlays:
        project_dir = args.project_dir if len(overlays) == 1 else None
        if args.validate_sources:
            local_target = local_targets.get(overlay)
            source = (
                local_target.source_dir
                if local_target and local_target.source_dir
                else AGENTS_ROOT / "overlays" / overlay
            )
            status |= validate_sources(overlay, source)
        else:
            status |= install_overlay_with_targets(
                overlay,
                dry_run=args.dry_run,
                project_dir=project_dir,
                local_targets=local_targets,
            )
    return status


if __name__ == "__main__":
    raise SystemExit(main())
