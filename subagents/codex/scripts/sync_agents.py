#!/usr/bin/env python3
"""Install the Codex subagent catalog with backups."""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from collections.abc import Iterable
from dataclasses import dataclass
from datetime import datetime
from datetime import timezone
from pathlib import Path
from pathlib import PureWindowsPath
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = ROOT.parents[1]
AGENTS_ROOT = ROOT / "agents"
VALIDATOR = (
    REPO_ROOT
    / "skills"
    / "subagent-creator"
    / "scripts"
    / "subagent_creator.py"
)
DEFAULT_LOCAL_MANIFEST = ROOT / "overlays.local.json"
RELEASE_MANIFEST = ROOT / "RELEASE_MANIFEST.json"
OVERLAY_ROOT = AGENTS_ROOT / "overlays"

PUBLIC_OVERLAY_TARGETS = {
    "docmind": Path.home() / "repos" / "agents" / "docmind-ai-llm",
    "tooling": Path.home() / "repos" / "agents" / "dev-skills",
}


@dataclass(frozen=True)
class CopyAction:
    """Planned or completed copy operation for one TOML role file.

    Attributes:
        source: Source role file path.
        target: Destination role file path.
        action: Human-readable action label.
        backup: Backup path when an existing destination is overwritten.
    """

    source: Path
    target: Path
    action: str
    backup: Path | None = None


@dataclass(frozen=True)
class OverlayTarget:
    """Local overlay install target loaded from an ignored manifest.

    Attributes:
        project_dir: Repository checkout that receives the overlay.
        source_dir: Optional role source directory override.
    """

    project_dir: Path
    source_dir: Path | None = None


def stamp() -> str:
    """Return a UTC timestamp for backup directory names.

    Returns:
        Timestamp string formatted as YYYYMMDDTHHMMSSZ.
    """

    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def resolve_path(value: str, *, base: Path) -> Path:
    """Resolve a manifest path relative to a base directory.

    Args:
        value: Path string from a manifest.
        base: Base directory for relative paths.

    Returns:
        Absolute resolved path.
    """

    path = Path(value).expanduser()
    if not path.is_absolute():
        path = base / path
    return path.resolve()


def resolve_repo_manifest_path(value: str, *, manifest: Path) -> Path:
    """Resolve a release manifest path under the repository root.

    Args:
        value: Repository-relative path from the manifest.
        manifest: Manifest path for error messages.

    Returns:
        Absolute resolved path.

    Raises:
        SystemExit: If the path is absolute or escapes the repository root.
    """

    relative = Path(value)
    windows_relative = PureWindowsPath(value)
    if (
        relative.is_absolute()
        or relative.drive
        or relative.anchor
        or windows_relative.is_absolute()
        or windows_relative.drive
        or windows_relative.anchor
        or ".." in relative.parts
        or ".." in windows_relative.parts
    ):
        raise SystemExit(
            f"release manifest path must be repo-relative: {manifest}: {value}"
        )
    resolved = (REPO_ROOT / relative).resolve()
    repo_root = REPO_ROOT.resolve()
    if resolved != repo_root and repo_root not in resolved.parents:
        raise SystemExit(
            f"release manifest path escapes repo root: {manifest}: {value}"
        )
    return resolved


def overlay_name_for_path(path: Path) -> str | None:
    """Return the public overlay name for a path under agents/overlays.

    Args:
        path: Path to compare against OVERLAY_ROOT.

    Returns:
        First path segment under OVERLAY_ROOT, or None when the path is
        outside OVERLAY_ROOT.
    """

    try:
        relative = path.resolve().relative_to(OVERLAY_ROOT.resolve())
    except ValueError:
        return None
    return relative.parts[0] if relative.parts else None


def load_local_targets(manifest: Path) -> dict[str, OverlayTarget]:
    """Load ignored local-only overlay install targets.

    Args:
        manifest: Local overlay manifest path.

    Returns:
        Mapping of overlay name to local target configuration.

    Raises:
        SystemExit: If the manifest exists but is malformed.
    """

    if not manifest.exists():
        return {}
    try:
        payload = json.loads(manifest.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(
            f"invalid local overlay manifest {manifest}: {exc}"
        ) from exc
    overlays = payload.get("overlays")
    if not isinstance(overlays, dict):
        raise SystemExit(
            "local overlay manifest must contain an object at overlays: "
            f"{manifest}"
        )

    targets: dict[str, OverlayTarget] = {}
    for name, config in sorted(overlays.items()):
        if not isinstance(name, str) or not name:
            raise SystemExit(
                f"local overlay names must be non-empty strings: {manifest}"
            )
        if not isinstance(config, dict):
            raise SystemExit(
                f"local overlay {name} must be an object: {manifest}"
            )
        project_dir = config.get("project_dir")
        if not isinstance(project_dir, str) or not project_dir:
            raise SystemExit(
                f"local overlay {name} requires project_dir: {manifest}"
            )
        source_dir_value = config.get("source_dir")
        if (
            source_dir_value is not None
            and not isinstance(source_dir_value, str)
        ):
            raise SystemExit(
                f"local overlay {name} source_dir must be a string: {manifest}"
            )
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
    """Ensure a source directory exists and contains TOML roles.

    Args:
        source: Source directory to inspect.

    Raises:
        SystemExit: If the source is missing or has no TOML files.
    """

    if not source.exists():
        raise SystemExit(f"source does not exist: {source}")
    if not any(source.glob("*.toml")):
        raise SystemExit(f"source has no TOML files: {source}")


def copy_tree(
    source: Path,
    target: Path,
    *,
    dry_run: bool,
    backup_dir: Path,
) -> list[CopyAction]:
    """Copy TOML roles from a source directory to a target directory.

    Args:
        source: Directory containing role TOML files.
        target: Destination agent directory.
        dry_run: Whether to report actions without writing.
        backup_dir: Directory for overwritten destination backups.

    Returns:
        Copy actions planned or performed for each source file.

    Raises:
        SystemExit: If the source directory is invalid.
    """

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


def validate(files: Iterable[Path]) -> int:
    """Validate exactly the provided role TOML files.

    Args:
        files: Concrete role files to validate.

    Returns:
        Validator process exit status.
    """

    role_files = sorted(files)
    command = [
        sys.executable,
        str(VALIDATOR),
        "validate",
        *[str(file) for file in role_files],
    ]
    return subprocess.run(command, check=False).returncode


def validate_public_overlay_allowlist(
    payload: dict[str, Any],
) -> tuple[set[str], list[str]]:
    """Validate public overlays that are intentionally redistributable.

    Args:
        payload: Parsed release manifest payload.

    Returns:
        Tuple of valid allowlisted overlay names and validation errors.
    """

    errors: list[str] = []
    allowlist = payload.get("explicit_public_overlay_allowlist")
    if not isinstance(allowlist, list) or not allowlist:
        return set(), [
            "explicit_public_overlay_allowlist must be a non-empty array"
        ]

    public_overlay_allowlist: set[str] = set()
    for item in allowlist:
        if not isinstance(item, str) or not item:
            errors.append(
                "explicit_public_overlay_allowlist entries must be "
                "non-empty strings"
            )
            continue
        public_overlay_allowlist.add(item)
        if item not in PUBLIC_OVERLAY_TARGETS:
            errors.append(
                f"public overlay is not configured in sync script: {item}"
            )
        if not (AGENTS_ROOT / "overlays" / item).is_dir():
            errors.append(f"public overlay directory does not exist: {item}")
    return public_overlay_allowlist, errors


def validate_public_sources(
    payload: dict[str, Any],
    *,
    manifest: Path,
    public_overlay_allowlist: set[str],
) -> list[str]:
    """Validate tracked public release sources.

    Args:
        payload: Parsed release manifest payload.
        manifest: Release manifest path for diagnostics.
        public_overlay_allowlist: Overlay names allowed in public sources.

    Returns:
        Validation error messages.
    """

    errors: list[str] = []
    public_sources = payload.get("public_sources")
    if not isinstance(public_sources, list) or not public_sources:
        return ["public_sources must be a non-empty array"]

    for item in public_sources:
        if not isinstance(item, str) or not item:
            errors.append("public_sources entries must be non-empty strings")
            continue
        try:
            path = resolve_repo_manifest_path(item, manifest=manifest)
        except SystemExit as exc:
            errors.append(str(exc))
            continue
        if not path.exists():
            errors.append(f"public source does not exist: {item}")
        overlay_name = overlay_name_for_path(path)
        if (
            overlay_name is not None
            and overlay_name not in public_overlay_allowlist
        ):
            errors.append(
                "public source references non-allowlisted overlay: "
                f"{item}"
            )
    return errors


def validate_private_local_only(
    payload: dict[str, Any],
    *,
    manifest: Path,
    public_overlay_allowlist: set[str],
) -> list[str]:
    """Validate private path patterns that must remain local-only.

    Args:
        payload: Parsed release manifest payload.
        manifest: Release manifest path for diagnostics.
        public_overlay_allowlist: Overlay names allowed in public sources.

    Returns:
        Validation error messages.
    """

    errors: list[str] = []
    private_local_only = payload.get("private_local_only")
    if not isinstance(private_local_only, list) or not private_local_only:
        return ["private_local_only must be a non-empty array"]

    for item in private_local_only:
        if not isinstance(item, dict):
            errors.append("private_local_only entries must be objects")
            continue
        if not isinstance(item.get("path"), str) or not item["path"]:
            errors.append("private_local_only entries require path")
        else:
            try:
                private_path = resolve_repo_manifest_path(
                    item["path"],
                    manifest=manifest,
                )
            except SystemExit as exc:
                errors.append(str(exc))
                private_path = None
            if private_path is not None:
                overlay_name = overlay_name_for_path(private_path)
                if (
                    overlay_name is not None
                    and overlay_name in public_overlay_allowlist
                ):
                    errors.append(
                        "private_local_only references public overlay "
                        "allowlist: "
                        f"{item['path']}"
                    )
        if not isinstance(item.get("reason"), str) or not item["reason"]:
            errors.append("private_local_only entries require reason")
    return errors


def validate_release_action_lists(payload: dict[str, Any]) -> list[str]:
    """Validate command-list sections in the release manifest.

    Args:
        payload: Parsed release manifest payload.

    Returns:
        Validation error messages.
    """

    errors: list[str] = []
    for key in ("dry_run_first", "apply", "rollback", "smoke_matrix"):
        value = payload.get(key)
        if not isinstance(value, list) or not value:
            errors.append(f"{key} must be a non-empty array")
    return errors


def validate_smoke_matrix(payload: dict[str, Any]) -> list[str]:
    """Validate named smoke checks in the release manifest.

    Args:
        payload: Parsed release manifest payload.

    Returns:
        Validation error messages.
    """

    errors: list[str] = []
    smoke_matrix = payload.get("smoke_matrix", [])
    if not isinstance(smoke_matrix, list):
        return errors
    for item in smoke_matrix:
        if not isinstance(item, dict):
            errors.append("smoke_matrix entries must be objects")
            continue
        if not isinstance(item.get("name"), str) or not item["name"]:
            errors.append("smoke_matrix entries require name")
        if not isinstance(item.get("command"), str) or not item["command"]:
            errors.append("smoke_matrix entries require command")
    return errors


def validate_sources(label: str, source: Path) -> int:
    """Validate all role TOML files in a source directory.

    Args:
        label: Display label for the validation block.
        source: Source directory containing role TOML files.

    Returns:
        Validator process exit status.

    Raises:
        SystemExit: If the source directory is invalid.
    """

    ensure_source_exists(source)
    print(f"## validate {label}")
    status = validate(source.glob("*.toml"))
    print(f"{'ok' if status == 0 else 'failed'}: {source}")
    return status


def validate_release_manifest(manifest: Path = RELEASE_MANIFEST) -> int:
    """Validate the tracked release manifest boundary.

    Args:
        manifest: Release manifest JSON path.

    Returns:
        Zero when the manifest is structurally valid.
    """

    errors: list[str] = []
    try:
        payload = json.loads(manifest.read_text(encoding="utf-8"))
    except FileNotFoundError:
        errors.append(f"missing release manifest: {manifest}")
        payload = {}
    except json.JSONDecodeError as exc:
        errors.append(f"invalid release manifest {manifest}: {exc}")
        payload = {}

    if payload.get("schema") != "dev-skills.codex-subagents-release.v1":
        errors.append("release manifest schema is invalid")

    public_overlay_allowlist, allowlist_errors = (
        validate_public_overlay_allowlist(payload)
    )
    errors.extend(allowlist_errors)
    errors.extend(
        validate_public_sources(
            payload,
            manifest=manifest,
            public_overlay_allowlist=public_overlay_allowlist,
        )
    )
    errors.extend(
        validate_private_local_only(
            payload,
            manifest=manifest,
            public_overlay_allowlist=public_overlay_allowlist,
        )
    )
    errors.extend(validate_release_action_lists(payload))
    errors.extend(validate_smoke_matrix(payload))

    if errors:
        print("## release manifest")
        for error in errors:
            print(f"failed: {error}")
        return 1
    print(f"ok: {manifest}")
    return 0


def print_actions(label: str, actions: list[CopyAction]) -> None:
    """Print copy actions in a stable human-readable format.

    Args:
        label: Section label.
        actions: Copy actions to print.
    """

    print(f"## {label}")
    for action in actions:
        backup_note = f" backup={action.backup}" if action.backup else ""
        print(
            f"{action.action}: "
            f"{action.source.name} -> {action.target}{backup_note}"
        )


def install_global(*, dry_run: bool) -> int:
    """Install or preview global roles under ~/.codex/agents.

    Args:
        dry_run: Whether to report actions without writing.

    Returns:
        Zero on success, or validator failure status.
    """

    source = AGENTS_ROOT / "global"
    target = Path.home() / ".codex" / "agents"
    backup_dir = target.parent / "agent-backups" / f"global-{stamp()}"
    actions = copy_tree(source, target, dry_run=dry_run, backup_dir=backup_dir)
    print_actions("global", actions)
    if dry_run:
        return 0
    return validate(action.target for action in actions)


def install_overlay(
    name: str,
    *,
    dry_run: bool,
    project_dir: Path | None = None,
) -> int:
    """Install or preview one public or explicitly targeted overlay.

    Args:
        name: Overlay name.
        dry_run: Whether to report actions without writing.
        project_dir: Optional destination repository override.

    Returns:
        Zero on success, or validator failure status.

    Raises:
        SystemExit: If the overlay requires a destination.
    """

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
    """Install or preview an overlay using public and local target mappings.

    Args:
        name: Overlay name.
        dry_run: Whether to report actions without writing.
        project_dir: Optional destination repository override.
        local_targets: Local overlay target mappings.

    Returns:
        Zero on success, or validator failure status.

    Raises:
        SystemExit: If the overlay is unknown or lacks a destination.
    """

    local_target = local_targets.get(name)
    source = (
        local_target.source_dir
        if local_target is not None and local_target.source_dir is not None
        else AGENTS_ROOT / "overlays" / name
    )
    if (
        project_dir is None
        and name not in PUBLIC_OVERLAY_TARGETS
        and local_target is None
    ):
        available = ", ".join(sorted(PUBLIC_OVERLAY_TARGETS))
        raise SystemExit(
            f"unknown public overlay {name}; available: {available}; "
            "pass --project-dir or add it to the local overlay manifest"
        )
    default_project = (
        local_target.project_dir
        if local_target is not None
        else PUBLIC_OVERLAY_TARGETS.get(name)
    )
    if project_dir is None and default_project is None:
        raise SystemExit(f"overlay {name} requires --project-dir")
    explicit_project_target = (
        project_dir is not None or local_target is not None
    )
    project = (project_dir or default_project).expanduser().resolve()
    if not project.is_dir() and (explicit_project_target or not dry_run):
        raise SystemExit(
            f"overlay {name} target project does not exist: {project}"
        )
    target = project / ".codex" / "agents"
    backup_dir = target.parent / "agent-backups" / f"{name}-{stamp()}"
    actions = copy_tree(source, target, dry_run=dry_run, backup_dir=backup_dir)
    print_actions(name, actions)
    if dry_run:
        return 0
    return validate(action.target for action in actions)


def build_parser() -> argparse.ArgumentParser:
    """Build the sync command-line parser.

    Returns:
        Configured argument parser.
    """

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--global", dest="install_global", action="store_true")
    parser.add_argument("--overlay", action="append", default=[])
    parser.add_argument("--all-overlays", action="store_true")
    parser.add_argument("--all-local-overlays", action="store_true")
    parser.add_argument(
        "--local-manifest",
        type=Path,
        default=DEFAULT_LOCAL_MANIFEST,
    )
    parser.add_argument("--project-dir", type=Path)
    parser.add_argument("--validate-sources", action="store_true")
    parser.add_argument("--validate-release-manifest", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    return parser


def keep_first_failure(status: int, result: int) -> int:
    """Preserve the first non-zero status code.

    Args:
        status: Current aggregate status.
        result: New command result.

    Returns:
        The first non-zero status, or zero when all results passed.
    """

    return status if status != 0 else result


def main(argv: list[str] | None = None) -> int:
    """Run the sync CLI.

    Args:
        argv: Optional command-line arguments for tests or embedding.

    Returns:
        Process exit status.

    Raises:
        SystemExit: If no operation is selected.
    """

    args = build_parser().parse_args(argv)
    local_manifest = args.local_manifest.expanduser().resolve()
    local_targets = load_local_targets(local_manifest)
    overlays = list(args.overlay)
    status = 0
    if args.all_overlays:
        overlays = sorted({*overlays, *PUBLIC_OVERLAY_TARGETS})
    if args.all_local_overlays:
        overlays = sorted({*overlays, *local_targets})
    if args.project_dir is not None and len(overlays) != 1:
        raise SystemExit(
            "cannot use --project-dir with multiple overlays; specify a "
            "single --overlay or omit --project-dir"
        )
    if args.list:
        print("public overlays:")
        for name, target in sorted(PUBLIC_OVERLAY_TARGETS.items()):
            print(f"- {name}: {target}")
        print("local overlays:")
        for name, target in sorted(local_targets.items()):
            source = target.source_dir or AGENTS_ROOT / "overlays" / name
            print(f"- {name}: {target.project_dir} source={source}")
        return 0
    if args.validate_release_manifest:
        status = keep_first_failure(status, validate_release_manifest())
    if not args.install_global and not overlays:
        if args.validate_release_manifest:
            return status
        raise SystemExit(
            "select --global, --overlay <name>, --all-overlays, "
            "--all-local-overlays, --validate-release-manifest, or --list"
        )

    if args.install_global:
        if args.validate_sources:
            status = keep_first_failure(
                status,
                validate_sources("global", AGENTS_ROOT / "global"),
            )
        else:
            status = keep_first_failure(
                status,
                install_global(dry_run=args.dry_run),
            )
    for overlay in overlays:
        project_dir = args.project_dir if len(overlays) == 1 else None
        if args.validate_sources:
            local_target = local_targets.get(overlay)
            source = (
                local_target.source_dir
                if (
                    local_target is not None
                    and local_target.source_dir is not None
                )
                else AGENTS_ROOT / "overlays" / overlay
            )
            status = keep_first_failure(
                status,
                validate_sources(overlay, source),
            )
        else:
            status = keep_first_failure(
                status,
                install_overlay_with_targets(
                    overlay,
                    dry_run=args.dry_run,
                    project_dir=project_dir,
                    local_targets=local_targets,
                ),
            )
    return status


if __name__ == "__main__":
    raise SystemExit(main())
