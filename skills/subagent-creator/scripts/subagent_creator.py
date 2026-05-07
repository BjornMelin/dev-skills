#!/usr/bin/env python3
"""Manage Codex custom subagent TOML templates and installs."""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    tomllib = None


SKILL_DIR = Path(__file__).resolve().parents[1]
TEMPLATE_DIR = SKILL_DIR / "templates" / "agents"
NAME_RE = re.compile(r"^[a-z][a-z0-9_]*$")
NICKNAME_RE = re.compile(r"^[A-Za-z0-9 _-]+$")
VALID_EFFORTS = {"minimal", "low", "medium", "high", "xhigh"}
VALID_SANDBOXES = {"read-only", "workspace-write", "danger-full-access"}
RESERVED_BUILTIN_AGENT_NAMES = {"default", "worker", "explorer"}
RESEARCH_CONTRACT_AGENT_NAMES = {
    "deep_researcher",
    "github_researcher",
    "context7_researcher",
    "openai_docs_researcher",
    "source_validator",
    "citation_auditor",
}
RESEARCH_CONTRACT_HEADINGS = (
    "Status",
    "Sources hydrated",
    "Claims",
    "Provider limits",
    "Privacy notes",
    "Recommended next verification",
)
ALLOWED_TOP_LEVEL_KEYS = {
    "name",
    "description",
    "developer_instructions",
    "nickname_candidates",
    "model",
    "model_reasoning_effort",
    "sandbox_mode",
    "mcp_servers",
    "skills",
}

PACKS: dict[str, list[str]] = {
    "core": [
        "reviewer",
        "repo_explorer",
        "docs_researcher",
        "test_runner",
        "implementation_worker",
        "ui_debugger",
        "ci_triager",
    ],
    "docs": [
        "docs_researcher",
        "openai_docs_researcher",
        "context7_researcher",
        "dependency_researcher",
    ],
    "review": [
        "guidance_mapper",
        "shallow_bug_reviewer",
        "history_reviewer",
        "false_positive_validator",
        "reviewer",
    ],
    "audit": [
        "security_reviewer",
        "runtime_bug_reviewer",
        "dependency_researcher",
        "performance_reviewer",
        "docs_auditor",
    ],
    "ops": [
        "ci_triager",
        "release_validator",
        "env_validator",
        "test_runner",
    ],
}


@dataclass
class ValidationIssue:
    path: Path
    message: str

    def to_dict(self) -> dict[str, str]:
        return {"path": str(self.path), "message": self.message}


@dataclass
class CopyResult:
    source: Path
    target: Path
    action: str
    backup: Path | None = None

    def to_dict(self) -> dict[str, str | None]:
        return {
            "source": str(self.source),
            "target": str(self.target),
            "action": self.action,
            "backup": None if self.backup is None else str(self.backup),
        }


@dataclass
class PruneResult:
    """Result for one stale role prune operation.

    Args:
        target: Installed TOML role path considered for pruning.
        action: Action taken or planned for the target.
        backup: Optional backup path created before deletion.
    """

    target: Path
    action: str
    backup: Path | None = None

    def to_dict(self) -> dict[str, str | None]:
        """Return a JSON-serializable prune result.

        Returns:
            dict[str, str | None]: Target, action, and backup path.
        """

        return {
            "target": str(self.target),
            "action": self.action,
            "backup": None if self.backup is None else str(self.backup),
        }


def now_stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


def template_paths() -> dict[str, Path]:
    return {
        path.stem: path
        for path in sorted(TEMPLATE_DIR.glob("*.toml"))
        if path.is_file()
    }


def expand_packs(packs: list[str]) -> list[str]:
    if not packs:
        return []
    templates = template_paths()
    names: list[str] = []
    missing = [pack for pack in packs if pack != "all" and pack not in PACKS]
    if missing:
        available = ", ".join(sorted([*PACKS, "all"]))
        raise SystemExit(f"Unknown pack(s): {', '.join(missing)}. Available: {available}")
    for pack in packs:
        if pack == "all":
            names.extend(sorted(templates))
        else:
            names.extend(PACKS[pack])
    return names


def resolve_templates(names: list[str], packs: list[str] | None = None) -> list[Path]:
    templates = template_paths()
    selected_names = [*expand_packs(packs or []), *names]
    if not selected_names:
        selected_names = sorted(templates)

    missing = [name for name in selected_names if name not in templates]
    if missing:
        available = ", ".join(sorted(templates)) or "none"
        raise SystemExit(
            f"Unknown template(s): {', '.join(sorted(set(missing)))}. Available: {available}"
        )

    selected: dict[str, Path] = {}
    for name in selected_names:
        selected[name] = templates[name]
    return [selected[name] for name in sorted(selected)]


def load_toml(path: Path) -> tuple[dict[str, Any] | None, str | None]:
    if tomllib is None:
        return None, "Python 3.11+ is required for tomllib TOML parsing"
    try:
        with path.open("rb") as handle:
            parsed = tomllib.load(handle)
    except Exception as exc:
        return None, f"failed to parse TOML: {exc}"
    if not isinstance(parsed, dict):
        return None, "TOML root must be a table"
    return parsed, None


def validate_agent_file(path: Path) -> list[ValidationIssue]:
    issues: list[ValidationIssue] = []
    parsed, error = load_toml(path)
    if error:
        return [ValidationIssue(path, error)]
    assert parsed is not None

    for key in ("name", "description", "developer_instructions"):
        value = parsed.get(key)
        if not isinstance(value, str) or not value.strip():
            issues.append(ValidationIssue(path, f"missing non-empty `{key}`"))

    unknown_keys = sorted(set(parsed) - ALLOWED_TOP_LEVEL_KEYS)
    for key in unknown_keys:
        issues.append(ValidationIssue(path, f"unknown top-level key `{key}`"))

    name = parsed.get("name")
    if isinstance(name, str) and name.strip():
        normalized_name = name.strip()
        if not NAME_RE.fullmatch(normalized_name):
            issues.append(ValidationIssue(path, "`name` must be snake_case ASCII"))
        if path.stem != normalized_name:
            issues.append(
                ValidationIssue(
                    path,
                    f"filename stem must match name ({path.stem} != {normalized_name})",
                )
            )
        if normalized_name in RESERVED_BUILTIN_AGENT_NAMES:
            issues.append(
                ValidationIssue(
                    path,
                    f"`name` shadows built-in Codex agent `{normalized_name}`",
                )
            )

    if "instructions" in parsed:
        issues.append(
            ValidationIssue(path, "use `developer_instructions`, not legacy `instructions`")
        )
    if "reasoning_effort" in parsed:
        issues.append(
            ValidationIssue(path, "use `model_reasoning_effort`, not `reasoning_effort`")
        )

    effort = parsed.get("model_reasoning_effort")
    if effort is not None and effort not in VALID_EFFORTS:
        issues.append(
            ValidationIssue(
                path,
                "`model_reasoning_effort` must be one of "
                + ", ".join(sorted(VALID_EFFORTS)),
            )
        )

    sandbox = parsed.get("sandbox_mode")
    if sandbox is not None and sandbox not in VALID_SANDBOXES:
        issues.append(
            ValidationIssue(
                path,
                "`sandbox_mode` must be one of " + ", ".join(sorted(VALID_SANDBOXES)),
            )
        )

    nicknames = parsed.get("nickname_candidates")
    if nicknames is not None:
        if not isinstance(nicknames, list) or not nicknames:
            issues.append(ValidationIssue(path, "`nickname_candidates` must be a non-empty list"))
        else:
            seen: set[str] = set()
            for nickname in nicknames:
                if not isinstance(nickname, str) or not nickname.strip():
                    issues.append(
                        ValidationIssue(path, "`nickname_candidates` cannot contain blank entries")
                    )
                    continue
                value = nickname.strip()
                if value in seen:
                    issues.append(ValidationIssue(path, f"duplicate nickname candidate `{value}`"))
                seen.add(value)
                if not NICKNAME_RE.fullmatch(value):
                    issues.append(
                        ValidationIssue(path, f"nickname `{value}` contains unsupported characters")
                    )

    developer_instructions = parsed.get("developer_instructions")
    if isinstance(developer_instructions, str):
        if "TODO" in developer_instructions:
            issues.append(ValidationIssue(path, "developer_instructions contains TODO"))
        for required in (
            "Do not spawn nested subagents",
            "Treat the parent prompt as the authority",
            "Redact secrets",
            "Return format:",
        ):
            if required not in developer_instructions:
                issues.append(ValidationIssue(path, f"developer_instructions missing `{required}`"))
        if isinstance(name, str) and name.strip() in RESEARCH_CONTRACT_AGENT_NAMES:
            for required in RESEARCH_CONTRACT_HEADINGS:
                heading_pattern = rf"(?m)^\s*[-*+]\s*{re.escape(required)}(?:\s|\(|$)"
                if re.search(heading_pattern, developer_instructions) is None:
                    issues.append(
                        ValidationIssue(
                            path,
                            f"research contract missing `{required}` return heading",
                        )
                    )

    return issues


def agent_files(paths: list[Path]) -> list[Path]:
    out: list[Path] = []
    for path in paths:
        if path.is_dir():
            out.extend(sorted(candidate for candidate in path.rglob("*.toml")))
        else:
            out.append(path)
    return out


def validate_paths(paths: list[Path]) -> list[ValidationIssue]:
    files = agent_files(paths)
    if not files:
        return [ValidationIssue(Path("."), "no TOML files found")]
    issues: list[ValidationIssue] = []
    for path in files:
        issues.extend(validate_agent_file(path))
    return issues


def resolve_destination(args: argparse.Namespace) -> Path:
    """Return the resolved install destination for write commands.

    Args:
        args: Parsed command-line arguments with destination options.

    Returns:
        Path: Resolved destination directory.

    Raises:
        SystemExit: If target is invalid or destination is a file.
    """

    if getattr(args, "dest", None):
        dest = Path(args.dest).expanduser().resolve()
        if dest.exists() and not dest.is_dir():
            raise SystemExit(f"destination must be a directory: {dest}")
        return dest
    if args.target == "global":
        return global_agents_dir()
    if args.target == "project":
        return project_agents_dir(args.project_dir)
    raise SystemExit("target must be global or project")


def backup_file(path: Path, backup_dir: Path | None = None) -> Path:
    destination_dir = backup_dir or path.parent
    destination_dir.mkdir(parents=True, exist_ok=True)
    backup = destination_dir / f"{path.name}.bak-{now_stamp()}"
    shutil.copy2(path, backup)
    return backup


def copy_templates(
    templates: list[Path],
    dest: Path,
    *,
    dry_run: bool,
    overwrite: bool,
    backup: bool,
    backup_dir: Path | None = None,
    quiet: bool = False,
) -> list[CopyResult]:
    copied: list[CopyResult] = []
    if not dry_run:
        dest.mkdir(parents=True, exist_ok=True)

    for src in templates:
        target = dest / src.name
        backup_path: Path | None = None
        if target.exists():
            if not overwrite:
                raise SystemExit(f"Refusing to overwrite existing file: {target}")
            if backup and not dry_run:
                backup_path = backup_file(target, backup_dir)
        action = "would_install" if dry_run else "installed"
        if target.exists() and overwrite:
            action = "would_overwrite" if dry_run else "overwritten"
        copied.append(CopyResult(src, target, action, backup_path))
        if dry_run:
            if not quiet:
                print(f"{action} {src.name} -> {target}")
        else:
            shutil.copy2(src, target)
            if not quiet:
                if backup_path:
                    print(f"backed up {target} -> {backup_path}")
                print(f"{action} {target}")
    return copied


def emit_json(data: Any) -> None:
    print(json.dumps(data, indent=2, sort_keys=True))


def selected_template_rows(selected: list[Path]) -> list[dict[str, str]]:
    rows = []
    for path in selected:
        parsed, error = load_toml(path)
        rows.append(
            {
                "name": path.stem,
                "path": str(path.relative_to(SKILL_DIR)),
                "description": "" if error or parsed is None else str(parsed.get("description", "")),
            }
        )
    return rows


def selected_template_map(
    names: list[str],
    packs: list[str] | None = None,
) -> dict[str, Path]:
    """Resolve selected template paths keyed by role name.

    Args:
        names: Explicit template names.
        packs: Optional pack names to expand before explicit names.

    Returns:
        dict[str, Path]: Template path by template stem.

    Raises:
        SystemExit: If a requested template or pack is unknown.
    """

    return {path.stem: path for path in resolve_templates(names, packs)}


def template_pack_membership() -> dict[str, list[str]]:
    """Build pack membership by template name.

    Returns:
        dict[str, list[str]]: Pack names keyed by template name.
    """

    membership: dict[str, list[str]] = {}
    for pack, names in sorted(PACKS.items()):
        for name in names:
            membership.setdefault(name, []).append(pack)
    return membership


def codex_home() -> Path:
    """Return the configured Codex home directory.

    Returns:
        Path: Resolved CODEX_HOME, defaulting to ~/.codex.
    """

    return Path(os.environ.get("CODEX_HOME", "~/.codex")).expanduser().resolve()


def global_agents_dir() -> Path:
    """Return the global Codex custom agents directory.

    Returns:
        Path: Resolved global agents directory.
    """

    return codex_home() / "agents"


def project_agents_dir(project_dir: str) -> Path:
    """Return the project-scoped Codex custom agents directory.

    Args:
        project_dir: Project root directory.

    Returns:
        Path: Resolved project agents directory.

    Raises:
        SystemExit: If project_dir exists and is not a directory.
    """

    root = Path(project_dir).expanduser().resolve()
    if root.exists() and not root.is_dir():
        raise SystemExit(f"project_dir must be a directory: {root}")
    return root / ".codex" / "agents"


def installed_templates(dest: Path) -> dict[str, Path]:
    """Return installed TOML role files from a destination.

    Args:
        dest: Directory to inspect.

    Returns:
        dict[str, Path]: Regular TOML files keyed by filename stem.
    """

    if not dest.exists():
        return {}
    return {
        path.stem: path
        for path in sorted(dest.glob("*.toml"))
        if path.is_file()
    }


def compare_template_to_target(
    template: Path | None,
    target: Path | None,
) -> str:
    """Compare one bundled template against one installed target.

    Args:
        template: Bundled template path, if any.
        target: Installed target path, if any.

    Returns:
        str: One of extra, unknown, missing, same, or different.
    """

    if template is None:
        return "extra" if target is not None and target.exists() else "unknown"
    if target is None or not target.exists():
        return "missing"
    if template.read_bytes() == target.read_bytes():
        return "same"
    return "different"


def status_rows(
    names: list[str],
    packs: list[str],
    project_dir: str,
    *,
    include_extra: bool,
) -> tuple[list[dict[str, Any]], dict[str, Path]]:
    """Build cross-scope install inventory rows.

    Args:
        names: Explicit template names.
        packs: Pack names to expand.
        project_dir: Project root for project-scoped agents.
        include_extra: Include installed roles outside the selected set.

    Returns:
        tuple[list[dict[str, Any]], dict[str, Path]]: Inventory rows and target
            directories keyed by scope.

    Raises:
        SystemExit: If selection contains unknown templates or packs.
    """

    all_templates = template_paths()
    templates = selected_template_map(names, packs)
    global_dir = global_agents_dir()
    project_dir_path = project_agents_dir(project_dir)
    global_installed = installed_templates(global_dir)
    project_installed = installed_templates(project_dir_path)
    membership = template_pack_membership()

    row_names = set(templates)
    if include_extra:
        row_names.update(global_installed)
        row_names.update(project_installed)

    rows: list[dict[str, Any]] = []
    for name in sorted(row_names):
        selected = name in templates
        template = templates.get(name) or all_templates.get(name)
        global_target = (
            global_installed.get(name) or global_dir / f"{name}.toml"
        )
        project_target = (
            project_installed.get(name) or project_dir_path / f"{name}.toml"
        )
        if selected:
            global_status = compare_template_to_target(
                template,
                global_installed.get(name),
            )
            project_status = compare_template_to_target(
                template,
                project_installed.get(name),
            )
        else:
            global_status = (
                "extra" if name in global_installed else "not_selected"
            )
            project_status = (
                "extra" if name in project_installed else "not_selected"
            )
        rows.append(
            {
                "name": name,
                "selected": selected,
                "template": None if template is None else str(template),
                "packs": membership.get(name, []),
                "global": {
                    "path": str(global_target),
                    "status": global_status,
                },
                "project": {
                    "path": str(project_target),
                    "status": project_status,
                },
            }
        )
    return rows, {"global": global_dir, "project": project_dir_path}


def summarize_status(rows: list[dict[str, Any]]) -> dict[str, dict[str, int]]:
    """Count inventory statuses by install scope.

    Args:
        rows: Inventory rows from status_rows().

    Returns:
        dict[str, dict[str, int]]: Status counts keyed by global/project scope.
    """

    summary: dict[str, dict[str, int]] = {"global": {}, "project": {}}
    for row in rows:
        for target_name in ("global", "project"):
            status = str(row[target_name]["status"])
            target_summary = summary[target_name]
            target_summary[status] = target_summary.get(status, 0) + 1
    return summary


def status_is_drift(status: str) -> bool:
    """Return whether an install status represents drift.

    Args:
        status: Status string from an inventory row.

    Returns:
        bool: True when status needs install, sync, or cleanup action.
    """

    return status in {"missing", "different", "extra"}


def cmd_list(args: argparse.Namespace) -> int:
    templates = [path for _, path in sorted(template_paths().items())]
    rows = selected_template_rows(templates)
    pack_rows = [
        {"name": name, "templates": names}
        for name, names in sorted(PACKS.items())
    ]
    if args.json:
        emit_json({"templates": rows, "packs": pack_rows})
    else:
        for row in rows:
            print(f"{row['name']}: {row['description']}")
        if args.packs:
            print()
            for pack in pack_rows:
                print(f"pack {pack['name']}: {', '.join(pack['templates'])}")
    return 0


def cmd_status(args: argparse.Namespace) -> int:
    """Handle the status subcommand.

    Args:
        args: Parsed command-line arguments.

    Returns:
        int: Process exit status.

    Raises:
        SystemExit: If selection contains unknown templates or packs.
    """

    rows, dirs = status_rows(
        args.names,
        args.pack,
        args.project_dir,
        include_extra=args.include_extra,
    )
    summary = summarize_status(rows)
    report = {
        "template_dir": str(TEMPLATE_DIR),
        "global_agents_dir": str(dirs["global"]),
        "project_agents_dir": str(dirs["project"]),
        "summary": summary,
        "agents": rows,
    }
    if args.json:
        emit_json(report)
    else:
        print(f"templates: {TEMPLATE_DIR}")
        print(f"global: {dirs['global']}")
        print(f"project: {dirs['project']}")
        for row in rows:
            packs = ",".join(row["packs"]) if row["packs"] else "-"
            selected = "selected" if row["selected"] else "extra"
            print(
                f"{row['name']}: "
                f"global={row['global']['status']} "
                f"project={row['project']['status']} "
                f"packs={packs} "
                f"{selected}"
            )
    if not args.fail_on_drift:
        return 0
    targets = ("global", "project") if args.check == "all" else (args.check,)
    has_drift = any(
        status_is_drift(str(row[target]["status"]))
        for row in rows
        for target in targets
    )
    return 1 if has_drift else 0


def cmd_plan_sync(args: argparse.Namespace) -> int:
    """Handle the plan-sync subcommand.

    Args:
        args: Parsed command-line arguments.

    Returns:
        int: Process exit status.

    Raises:
        SystemExit: If selection or destination arguments are invalid.
    """

    all_templates = template_paths()
    selected = selected_template_map(args.names, args.pack)
    dest = resolve_destination(args)
    installed = installed_templates(dest)
    rows: list[dict[str, Any]] = []

    for name, src in sorted(selected.items()):
        target = dest / src.name
        status = compare_template_to_target(src, installed.get(name))
        action = {
            "same": "keep",
            "missing": "install",
            "different": "overwrite_with_backup",
        }.get(status, "inspect")
        rows.append(
            {
                "name": name,
                "template": str(src),
                "target": str(target),
                "status": status,
                "action": action,
            }
        )

    if args.include_extra or args.prune_extra:
        for name, target in sorted(installed.items()):
            if name in selected:
                continue
            template = all_templates.get(name)
            rows.append(
                {
                    "name": name,
                    "template": None if template is None else str(template),
                    "target": str(target),
                    "status": "extra",
                    "action": "prune" if args.prune_extra else "keep_extra",
                }
            )

    summary: dict[str, int] = {}
    for row in rows:
        action = str(row["action"])
        summary[action] = summary.get(action, 0) + 1
    report = {"destination": str(dest), "summary": summary, "plan": rows}
    if args.json:
        emit_json(report)
    else:
        for row in rows:
            print(
                f"{row['action']}: "
                f"{row['name']} ({row['status']}) -> "
                f"{row['target']}"
            )
    return 0


def cmd_prune(args: argparse.Namespace) -> int:
    """Handle the prune subcommand.

    Args:
        args: Parsed command-line arguments.

    Returns:
        int: Process exit status.

    Raises:
        SystemExit: If selection or destination arguments are invalid.
        OSError: If confirmed backup or deletion fails.
    """

    selected = selected_template_map(args.names, args.pack)
    selected_files = {path.name for path in selected.values()}
    dest = resolve_destination(args)
    installed = sorted(installed_templates(dest).values())
    stale = [path for path in installed if path.name not in selected_files]
    dry_run = args.dry_run or not args.confirm
    backup = not args.no_backup
    backup_dir = (
        Path(args.backup_dir).expanduser().resolve()
        if args.backup_dir
        else None
    )
    results: list[PruneResult] = []

    for target in stale:
        backup_path: Path | None = None
        if dry_run:
            action = "would_prune"
        else:
            if backup:
                backup_path = backup_file(target, backup_dir)
            target.unlink()
            action = "pruned"
        results.append(PruneResult(target, action, backup_path))

    report = {
        "destination": str(dest),
        "dry_run": dry_run,
        "selected_templates": sorted(selected_files),
        "results": [result.to_dict() for result in results],
    }
    if args.json:
        emit_json(report)
    else:
        if not stale:
            print("no stale roles")
        for result in results:
            print(f"{result.action}: {result.target}")
            if result.backup:
                print(f"backup: {result.backup}")
        if not args.confirm and not args.dry_run and stale:
            print("dry run only; pass --confirm to delete stale roles")
    return 0


def cmd_render(args: argparse.Namespace) -> int:
    selected = resolve_templates(args.names, args.pack)
    if args.out_dir:
        out_dir = Path(args.out_dir).expanduser().resolve()
        results = copy_templates(
            selected,
            out_dir,
            dry_run=False,
            overwrite=args.overwrite,
            backup=args.backup,
        )
        if args.json:
            emit_json([result.to_dict() for result in results])
        return 0

    for index, path in enumerate(selected):
        if index:
            print()
        print(f"# {path.name}")
        print(path.read_text(encoding="utf-8"))
    return 0


def install_selected(args: argparse.Namespace, *, overwrite: bool, backup_default: bool) -> int:
    selected = resolve_templates(args.names, args.pack)
    dest = resolve_destination(args)
    backup_dir = Path(args.backup_dir).expanduser().resolve() if args.backup_dir else None
    backup = backup_default if args.backup is None else args.backup
    results = copy_templates(
        selected,
        dest,
        dry_run=args.dry_run,
        overwrite=overwrite or args.overwrite,
        backup=backup,
        backup_dir=backup_dir,
        quiet=args.json,
    )
    validation_inputs = selected if args.dry_run else [result.target for result in results]
    issues = validate_paths(validation_inputs)
    output = {
        "destination": str(dest),
        "dry_run": args.dry_run,
        "results": [result.to_dict() for result in results],
        "validation_issues": [issue.to_dict() for issue in issues],
    }
    if args.json:
        emit_json(output)
    elif issues:
        for issue in issues:
            print(f"{issue.path}: {issue.message}", file=sys.stderr)
    else:
        print("validation passed")
    return 1 if issues else 0


def cmd_install(args: argparse.Namespace) -> int:
    return install_selected(args, overwrite=False, backup_default=False)


def cmd_sync(args: argparse.Namespace) -> int:
    if args.no_backup:
        args.backup = False
    elif args.backup is None:
        args.backup = True
    return install_selected(args, overwrite=True, backup_default=True)


def cmd_validate(args: argparse.Namespace) -> int:
    paths = [Path(path).expanduser().resolve() for path in args.paths]
    issues = validate_paths(paths)
    if args.json:
        emit_json([issue.to_dict() for issue in issues])
    elif issues:
        for issue in issues:
            print(f"{issue.path}: {issue.message}")
    else:
        print("validation passed")
    return 1 if issues else 0


def cmd_diff(args: argparse.Namespace) -> int:
    selected = resolve_templates(args.names, args.pack)
    dest = resolve_destination(args)
    rows: list[dict[str, str]] = []
    selected_names = {path.name for path in selected}
    for src in selected:
        target = dest / src.name
        if not target.exists():
            status = "missing"
        elif src.read_bytes() == target.read_bytes():
            status = "same"
        else:
            status = "different"
        rows.append({"name": src.stem, "template": str(src), "target": str(target), "status": status})

    if args.include_extra and dest.exists():
        for target in sorted(dest.glob("*.toml")):
            if target.name not in selected_names:
                rows.append(
                    {"name": target.stem, "template": "", "target": str(target), "status": "extra"}
                )

    if args.json:
        emit_json({"destination": str(dest), "diff": rows})
    else:
        for row in rows:
            print(f"{row['status']}: {row['name']} -> {row['target']}")
    return 1 if any(row["status"] in {"missing", "different"} for row in rows) else 0


def cmd_backup(args: argparse.Namespace) -> int:
    dest = resolve_destination(args)
    out_dir = (
        Path(args.out_dir).expanduser().resolve()
        if args.out_dir
        else dest / f".backup-{now_stamp()}"
    )
    if args.names:
        files = [dest / f"{name}.toml" for name in args.names]
    else:
        files = sorted(dest.glob("*.toml")) if dest.exists() else []
    missing = [path for path in files if not path.exists()]
    if missing:
        for path in missing:
            print(f"missing: {path}", file=sys.stderr)
        return 1
    results = []
    for path in files:
        backup = backup_file(path, out_dir)
        results.append({"source": str(path), "backup": str(backup)})
        if not args.json:
            print(f"backed up {path} -> {backup}")
    if args.json:
        emit_json({"backup_dir": str(out_dir), "files": results})
    return 0


def run_probe(command: list[str], timeout: int = 10) -> dict[str, Any]:
    try:
        result = subprocess.run(
            command,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
    except FileNotFoundError:
        return {"command": command, "found": False, "error": "not found"}
    except subprocess.TimeoutExpired:
        return {"command": command, "found": True, "error": "timeout"}
    return {
        "command": command,
        "found": True,
        "returncode": result.returncode,
        "stdout": result.stdout.strip(),
        "stderr": result.stderr.strip(),
    }


def lookup_path_value(data: dict[str, Any], keys: list[str]) -> Any:
    current: Any = data
    for key in keys:
        if not isinstance(current, dict) or key not in current:
            return None
        current = current[key]
    return current


def project_trust(config: dict[str, Any], project_dir: Path) -> str | None:
    projects = config.get("projects")
    if not isinstance(projects, dict):
        return None
    candidates = [str(project_dir), str(project_dir.resolve())]
    for candidate in candidates:
        value = projects.get(candidate)
        if isinstance(value, dict) and "trust_level" in value:
            return str(value["trust_level"])
    return None


def cmd_doctor(args: argparse.Namespace) -> int:
    codex_bin = shutil.which(args.codex_bin)
    codex_version = run_probe([args.codex_bin, "--version"]) if codex_bin else {
        "command": [args.codex_bin, "--version"],
        "found": False,
        "error": "not found",
    }
    codex_help = run_probe([args.codex_bin, "exec", "--help"]) if codex_bin else {
        "command": [args.codex_bin, "exec", "--help"],
        "found": False,
        "error": "not found",
    }

    codex_home_path = codex_home()
    global_agents = global_agents_dir()
    project_dir = Path(args.project_dir).expanduser().resolve()
    project_agents = project_agents_dir(args.project_dir)
    config_path = codex_home_path / "config.toml"

    config: dict[str, Any] = {}
    config_error: str | None = None
    if config_path.exists():
        parsed, error = load_toml(config_path)
        if error:
            config_error = error
        elif parsed is not None:
            config = parsed

    template_issues = validate_paths([TEMPLATE_DIR])
    report = {
        "codex_bin": codex_bin,
        "codex_version": codex_version,
        "codex_exec_help_available": codex_help.get("returncode") == 0,
        "codex_home": str(codex_home_path),
        "config_path": str(config_path),
        "config_exists": config_path.exists(),
        "config_error": config_error,
        "features.multi_agent": lookup_path_value(config, ["features", "multi_agent"]),
        "agents.max_threads": lookup_path_value(config, ["agents", "max_threads"]),
        "agents.max_depth": lookup_path_value(config, ["agents", "max_depth"]),
        "agents.preferred_agent": lookup_path_value(config, ["agents", "preferred_agent"]),
        "global_agents_dir": str(global_agents),
        "global_agents_count": len(list(global_agents.glob("*.toml"))) if global_agents.exists() else 0,
        "project_dir": str(project_dir),
        "project_agents_dir": str(project_agents),
        "project_agents_count": len(list(project_agents.glob("*.toml"))) if project_agents.exists() else 0,
        "project_trust_level": project_trust(config, project_dir),
        "template_validation_issues": [issue.to_dict() for issue in template_issues],
    }

    if args.json:
        emit_json(report)
    else:
        print(f"codex: {codex_bin or 'not found'}")
        print(f"codex version: {codex_version.get('stdout') or codex_version.get('error')}")
        print(f"codex exec help: {'available' if report['codex_exec_help_available'] else 'unavailable'}")
        print(f"config: {config_path} ({'exists' if config_path.exists() else 'missing'})")
        if config_error:
            print(f"config error: {config_error}")
        print(f"features.multi_agent: {report['features.multi_agent']}")
        print(f"agents.max_threads: {report['agents.max_threads']}")
        print(f"agents.max_depth: {report['agents.max_depth']}")
        print(f"agents.preferred_agent: {report['agents.preferred_agent']}")
        print(f"global agents: {global_agents} ({report['global_agents_count']} TOML files)")
        print(f"project agents: {project_agents} ({report['project_agents_count']} TOML files)")
        print(f"project trust: {report['project_trust_level']}")
        if template_issues:
            print("template validation: failed")
            for issue in template_issues:
                print(f"{issue.path}: {issue.message}")
        else:
            print("template validation: passed")
    return 1 if template_issues else 0


def smoke_prompt(names: list[str]) -> str:
    listed = ", ".join(names)
    return (
        "Use the configured custom subagents for a harmless smoke test. "
        f"Spawn one agent for each of these roles: {listed}. "
        "Ask each agent to reply with exactly its role name and the word READY. "
        "Wait for all agents, then summarize whether every role responded."
    )


def cmd_smoke(args: argparse.Namespace) -> int:
    selected = resolve_templates(args.names, args.pack)
    names = [path.stem for path in selected]
    temp_root = Path(tempfile.mkdtemp(prefix="codex-subagent-smoke-"))
    project = temp_root / "project"
    agents_dir = project / ".codex" / "agents"
    agents_dir.mkdir(parents=True)
    (project / "AGENTS.md").write_text(
        "Subagent smoke project. Do not modify files.\n",
        encoding="utf-8",
    )
    copy_templates(selected, agents_dir, dry_run=False, overwrite=True, backup=False, quiet=True)
    issues = validate_paths([agents_dir])
    prompt = smoke_prompt(names)
    result: dict[str, Any] = {
        "project": str(project),
        "agents": names,
        "prompt": prompt,
        "validation_issues": [issue.to_dict() for issue in issues],
        "codex": None,
    }
    if issues:
        if not args.keep:
            shutil.rmtree(temp_root)
        if args.json:
            emit_json(result)
        else:
            for issue in issues:
                print(f"{issue.path}: {issue.message}", file=sys.stderr)
        return 1

    status = 0
    if args.run_codex:
        command = [
            args.codex_bin,
            "exec",
            "-C",
            str(project),
            "--skip-git-repo-check",
            "--ephemeral",
            prompt,
        ]
        probe = run_probe(command, timeout=args.timeout)
        result["codex"] = probe
        status = int(probe.get("returncode", 1))

    if args.json:
        emit_json(result)
    else:
        print(f"smoke project: {project}")
        print(f"prompt: {prompt}")
        if result["codex"]:
            codex_result = result["codex"]
            if codex_result.get("stdout"):
                print(codex_result["stdout"])
            if codex_result.get("stderr"):
                print(codex_result["stderr"], file=sys.stderr)

    if args.keep:
        if not args.json:
            print(f"kept smoke project: {project}")
    else:
        shutil.rmtree(temp_root)
    return status


def add_target_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--target", choices=("global", "project"), default="global")
    parser.add_argument("--project-dir", default=".")
    parser.add_argument("--dest")


def add_selection_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("names", nargs="*")
    parser.add_argument("--pack", action="append", default=[])


def add_copy_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--overwrite", action="store_true")
    parser.add_argument("--backup", action="store_true", default=None)
    parser.add_argument("--backup-dir")
    parser.add_argument("--json", action="store_true")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Create and validate Codex custom subagent TOML roles.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    list_parser = subparsers.add_parser("list", help="List bundled templates")
    list_parser.add_argument("--json", action="store_true")
    list_parser.add_argument("--packs", action="store_true")
    list_parser.set_defaults(func=cmd_list)

    status_parser = subparsers.add_parser(
        "status",
        help="Inventory template drift across installs",
    )
    add_selection_args(status_parser)
    status_parser.add_argument("--project-dir", default=".")
    status_parser.add_argument("--include-extra", action="store_true")
    status_parser.add_argument("--fail-on-drift", action="store_true")
    status_parser.add_argument(
        "--check",
        choices=("all", "global", "project"),
        default="all",
    )
    status_parser.add_argument("--json", action="store_true")
    status_parser.set_defaults(func=cmd_status)

    render_parser = subparsers.add_parser("render", help="Print or copy templates")
    add_selection_args(render_parser)
    render_parser.add_argument("--out-dir")
    render_parser.add_argument("--overwrite", action="store_true")
    render_parser.add_argument("--backup", action="store_true")
    render_parser.add_argument("--json", action="store_true")
    render_parser.set_defaults(func=cmd_render)

    install_parser = subparsers.add_parser("install", help="Install templates")
    add_selection_args(install_parser)
    add_target_args(install_parser)
    add_copy_args(install_parser)
    install_parser.set_defaults(func=cmd_install)

    sync_parser = subparsers.add_parser("sync", help="Overwrite installed templates with backups")
    add_selection_args(sync_parser)
    add_target_args(sync_parser)
    add_copy_args(sync_parser)
    sync_parser.add_argument("--no-backup", action="store_true")
    sync_parser.set_defaults(func=cmd_sync)

    diff_parser = subparsers.add_parser("diff", help="Compare templates to installed roles")
    add_selection_args(diff_parser)
    add_target_args(diff_parser)
    diff_parser.add_argument("--include-extra", action="store_true")
    diff_parser.add_argument("--json", action="store_true")
    diff_parser.set_defaults(func=cmd_diff)

    plan_sync_parser = subparsers.add_parser(
        "plan-sync",
        help="Plan a sync without writing files",
    )
    add_selection_args(plan_sync_parser)
    add_target_args(plan_sync_parser)
    plan_sync_parser.add_argument("--include-extra", action="store_true")
    plan_sync_parser.add_argument("--prune-extra", action="store_true")
    plan_sync_parser.add_argument("--json", action="store_true")
    plan_sync_parser.set_defaults(func=cmd_plan_sync)

    prune_parser = subparsers.add_parser(
        "prune",
        help="Remove installed roles that are not in the selected template set",
    )
    add_selection_args(prune_parser)
    add_target_args(prune_parser)
    prune_parser.add_argument("--confirm", action="store_true")
    prune_parser.add_argument("--dry-run", action="store_true")
    prune_parser.add_argument("--no-backup", action="store_true")
    prune_parser.add_argument("--backup-dir")
    prune_parser.add_argument("--json", action="store_true")
    prune_parser.set_defaults(func=cmd_prune)

    backup_parser = subparsers.add_parser("backup", help="Back up installed TOML roles")
    backup_parser.add_argument("names", nargs="*")
    add_target_args(backup_parser)
    backup_parser.add_argument("--out-dir")
    backup_parser.add_argument("--json", action="store_true")
    backup_parser.set_defaults(func=cmd_backup)

    validate_parser = subparsers.add_parser("validate", help="Validate TOML roles")
    validate_parser.add_argument("paths", nargs="+")
    validate_parser.add_argument("--json", action="store_true")
    validate_parser.set_defaults(func=cmd_validate)

    doctor_parser = subparsers.add_parser("doctor", help="Inspect Codex subagent environment")
    doctor_parser.add_argument("--codex-bin", default="codex")
    doctor_parser.add_argument("--project-dir", default=".")
    doctor_parser.add_argument("--json", action="store_true")
    doctor_parser.set_defaults(func=cmd_doctor)

    smoke_parser = subparsers.add_parser("smoke", help="Temp-project smoke setup")
    add_selection_args(smoke_parser)
    smoke_parser.add_argument("--run-codex", action="store_true")
    smoke_parser.add_argument("--codex-bin", default="codex")
    smoke_parser.add_argument("--timeout", type=int, default=300)
    smoke_parser.add_argument("--keep", action="store_true")
    smoke_parser.add_argument("--json", action="store_true")
    smoke_parser.set_defaults(func=cmd_smoke)

    pack_parser = subparsers.add_parser("pack", help="List or install template packs")
    pack_subparsers = pack_parser.add_subparsers(dest="pack_command", required=True)

    pack_list = pack_subparsers.add_parser("list", help="List template packs")
    pack_list.add_argument("--json", action="store_true")
    pack_list.set_defaults(func=lambda args: cmd_list(argparse.Namespace(json=args.json, packs=True)))

    pack_install = pack_subparsers.add_parser("install", help="Install one or more packs")
    pack_install.add_argument("packs", nargs="+")
    add_target_args(pack_install)
    add_copy_args(pack_install)
    pack_install.set_defaults(
        func=lambda args: install_selected(
            argparse.Namespace(
                names=[],
                pack=args.packs,
                target=args.target,
                project_dir=args.project_dir,
                dest=args.dest,
                dry_run=args.dry_run,
                overwrite=args.overwrite,
                backup=args.backup,
                backup_dir=args.backup_dir,
                json=args.json,
            ),
            overwrite=False,
            backup_default=False,
        )
    )

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
