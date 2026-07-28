#!/usr/bin/env python3
"""Safe Codex session artifact scanner and quarantine tool."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import shutil
import sqlite3
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

VERSION = "2.0.0"
HIGH = "high"
MEDIUM = "medium"
DEFAULT_MIN_AGE_HOURS = 24
BROAD_MIN_AGE_HOURS = 72
AUTO_RISK_LIMIT = 10
SESSION_FAMILY = "sessions"
MEMORY_FAMILY = "memory"
ALL_FAMILIES = [
    SESSION_FAMILY,
    MEMORY_FAMILY,
    "quarantine",
    "logs",
    "cache",
    "generated",
    "skills-agents-config",
]
SUBREPO_SKIP_DIRS = {
    ".git",
    ".hg",
    ".svn",
    "node_modules",
    ".next",
    ".turbo",
    ".venv",
    "venv",
    "__pycache__",
    "target",
    "dist",
    "build",
    "output",
    ".cache",
    "cache",
    "vendor",
}
SECRET_OR_AUTH_NAMES = {
    "auth.json",
    "internal_storage",
    ".env",
    ".env.local",
    ".env.production",
}


@dataclass(frozen=True)
class ScopeRoot:
    label: str
    path: Path | None
    root_kind: str
    match_mode: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "label": self.label,
            "path": str(self.path) if self.path else None,
            "root_kind": self.root_kind,
            "match_mode": self.match_mode,
        }


@dataclass
class ThreadRow:
    id: str
    rollout_path: str
    created_at: int
    updated_at: int
    cwd: str
    title: str
    tokens_used: int
    has_user_event: int
    archived: int
    first_user_message: str
    agent_role: str | None = None
    agent_path: str | None = None


@dataclass
class Candidate:
    thread: ThreadRow
    confidence: str | None
    reasons: list[str] = field(default_factory=list)
    protected_reasons: list[str] = field(default_factory=list)
    scope_label: str | None = None
    risk_score: int = 100
    age_hours: float = 0.0

    @property
    def selected(self) -> bool:
        return bool(self.confidence and not self.protected_reasons)


@dataclass
class MemoryFinding:
    path: Path
    recommendation: str
    reasons: list[str]
    linked_thread_ids: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "path": str(self.path),
            "recommendation": self.recommendation,
            "reasons": self.reasons,
            "linked_thread_ids": self.linked_thread_ids,
        }


@dataclass
class ArtifactFinding:
    family: str
    path: Path
    action: str
    reasons: list[str]
    bytes: int = 0
    protected_reasons: list[str] = field(default_factory=list)

    def to_dict(self) -> dict[str, Any]:
        return {
            "family": self.family,
            "path": str(self.path),
            "action": self.action,
            "reasons": self.reasons,
            "protected_reasons": self.protected_reasons,
            "bytes": self.bytes,
        }


class CleanupError(Exception):
    def __init__(self, message: str, code: int = 2) -> None:
        super().__init__(message)
        self.code = code


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Safely scan, quarantine, restore, and purge Codex session artifacts."
    )
    parser.add_argument(
        "--codex-home",
        default=os.environ.get("CODEX_HOME", str(Path.home() / ".codex")),
        help="Codex home directory. Defaults to CODEX_HOME or ~/.codex.",
    )
    parser.add_argument("--version", action="version", version=f"%(prog)s {VERSION}")
    sub = parser.add_subparsers(dest="command", required=True)

    scan = sub.add_parser("scan", help="Create a dry-run cleanup manifest and report.")
    scan.add_argument(
        "--scope",
        choices=["current", "root", "roots", "cwd-subrepos", "codex-home", "all"],
        default="current",
    )
    scan.add_argument("--cwd", default=os.getcwd(), help="Directory used for scope matching.")
    scan.add_argument("--root", action="append", default=[], help="Explicit root path; repeatable.")
    scan.add_argument("--roots-file", help="Text, JSON, or JSONL file with explicit roots.")
    scan.add_argument("--include-parent-overlap", action="store_true")
    scan.add_argument("--artifact-family", choices=ALL_FAMILIES, action="append")
    scan.add_argument("--include-medium", action="store_true")
    scan.add_argument("--min-age-hours", type=int)
    scan.add_argument(
        "--first-prompt-prefix",
        action="append",
        default=[],
        help="Limit scan results to threads whose first user prompt starts with this prefix. Repeatable.",
    )
    scan.add_argument(
        "--user-prompt-prefix",
        action="append",
        default=[],
        help="Select threads where any user prompt in the session file starts with this prefix. Repeatable.",
    )
    scan.add_argument(
        "--contains-text",
        action="append",
        default=[],
        help="Select threads whose session file contains this exact text. Repeatable.",
    )
    scan.add_argument(
        "--exclude-thread-id",
        action="append",
        default=[],
        help="Exclude an exact thread id from scan candidates. Repeatable.",
    )
    scan.add_argument(
        "--thread-id",
        action="append",
        default=[],
        help="Select an exact thread id. Repeatable.",
    )
    scan.add_argument("--output-dir", help="Directory for scan reports.")
    scan.add_argument("--report-name", help="Report basename without extension.")

    apply = sub.add_parser("apply", help="Apply a scan manifest into quarantine.")
    apply.add_argument("--manifest", required=True)
    apply.add_argument("--confirm", required=True, help="Manifest id to confirm.")
    apply.add_argument("--execute", action="store_true", help="Actually mutate artifacts.")
    apply.add_argument("--memory-policy", choices=["copy", "move", "ignore"], default="copy")
    apply.add_argument("--confirm-memory-move")
    apply.add_argument("--confirm-memory-ignore")
    apply.add_argument("--max-manifest-age-hours", type=int, default=24)
    apply.add_argument("--quarantine-dir", help="Override quarantine directory.")
    apply.add_argument(
        "--artifact-policy",
        choices=["report", "quarantine"],
        default="report",
        help="Whether manual artifact candidates remain report-only or move into quarantine.",
    )

    restore = sub.add_parser("restore", help="Restore files from a quarantine.")
    restore.add_argument("--quarantine", required=True)
    restore.add_argument("--execute", action="store_true")
    restore.add_argument("--restore-db", action="store_true")

    purge = sub.add_parser("purge", help="Purge old quarantine directories.")
    purge.add_argument("--older-than-days", type=int)
    purge.add_argument(
        "--quarantine",
        action="append",
        default=[],
        help="Exact quarantine directory or bundle name to purge. Repeatable.",
    )
    purge.add_argument("--confirm", required=True)
    purge.add_argument("--execute", action="store_true")

    args = parser.parse_args()
    codex_home = Path(args.codex_home).expanduser().resolve()

    try:
        if args.command == "scan":
            return command_scan(args, codex_home)
        if args.command == "apply":
            return command_apply(args, codex_home)
        if args.command == "restore":
            return command_restore(args, codex_home)
        if args.command == "purge":
            return command_purge(args, codex_home)
    except CleanupError as error:
        print(f"error: {error}", file=sys.stderr)
        return error.code
    except BrokenPipeError:
        return 1
    return 2


def command_scan(args: argparse.Namespace, codex_home: Path) -> int:
    require_dir(codex_home, "Codex home")
    now = int(dt.datetime.now(dt.UTC).timestamp())
    roots = resolve_scope_roots(args, codex_home)
    families = normalize_families(args.artifact_family)
    min_age_hours = effective_min_age_hours(args.scope, args.min_age_hours)

    threads = load_threads(codex_home)
    active_risks = collect_active_thread_risks(codex_home, now, min_age_hours)
    scoped = scope_threads(threads, roots, args.scope, args.include_parent_overlap)
    candidates = [
        classify_thread(row, now, min_age_hours, active_risks.get(row.id, []), scope_label)
        for row, scope_label in scoped
    ]
    thread_ids = set(args.thread_id)
    if thread_ids:
        candidates = mark_exact_thread_id_matches(candidates, thread_ids)
    first_prompt_prefixes = [prefix for prefix in args.first_prompt_prefix if prefix]
    if first_prompt_prefixes:
        candidates = [
            item
            for item in candidates
            if thread_matches_first_prompt_prefix(item.thread, first_prompt_prefixes)
        ]
    user_prompt_prefixes = [prefix for prefix in args.user_prompt_prefix if prefix]
    if user_prompt_prefixes:
        candidates = mark_user_prompt_prefix_matches(candidates, user_prompt_prefixes)
    contains_texts = [text for text in args.contains_text if text]
    if contains_texts:
        candidates = mark_contains_text_matches(candidates, contains_texts)
    excluded_thread_ids = set(args.exclude_thread_id)
    if excluded_thread_ids:
        candidates = [item for item in candidates if item.thread.id not in excluded_thread_ids]

    selected = [
        item
        for item in candidates
        if item.selected and (item.confidence == HIGH or args.include_medium)
    ]
    selected_ids = {item.thread.id for item in selected}
    selected_paths = {item.thread.rollout_path for item in selected if item.thread.rollout_path}
    medium_not_selected = [
        item for item in candidates if item.selected and item.confidence == MEDIUM and not args.include_medium
    ]
    protected = [item for item in candidates if item.protected_reasons]

    history = inspect_jsonl_ids(codex_home / "history.jsonl", "session_id", selected_ids)
    session_index = inspect_jsonl_ids(codex_home / "session_index.jsonl", "id", selected_ids)
    state_counts = inspect_state_counts(codex_home / "state_5.sqlite", selected_ids)
    log_counts = inspect_log_counts(codex_home / "logs_2.sqlite", selected_ids)
    session_files = inspect_session_files(selected) if SESSION_FAMILY in families else []
    memory = inspect_memory(codex_home, selected_ids, selected_paths) if MEMORY_FAMILY in families else empty_memory(codex_home)
    artifacts = inspect_artifact_families(codex_home, families, now, min_age_hours)
    automation = build_automation_summary(selected, memory, history, session_index, state_counts, log_counts)

    target_label = build_target_label(args.scope, roots)
    manifest_id = build_manifest_id(args.scope, target_label)
    output_dir = (
        Path(args.output_dir).expanduser().resolve()
        if args.output_dir
        else codex_home / "prune-quarantine" / "reports"
    )
    report_name = args.report_name or manifest_id
    output_dir.mkdir(parents=True, exist_ok=True)

    manifest = {
        "version": VERSION,
        "manifest_id": manifest_id,
        "generated_at": iso_now(),
        "codex_home": str(codex_home),
        "scope": args.scope,
        "scope_roots": [root.to_dict() for root in roots],
        "target_root": str(roots[0].path) if len(roots) == 1 and roots[0].path else target_label,
        "target_label": target_label,
        "artifact_families": families,
        "rule_profile": "universal-low-value-copy-first",
        "include_medium": args.include_medium,
        "include_parent_overlap": args.include_parent_overlap,
        "min_age_hours": min_age_hours,
        "first_prompt_prefixes": first_prompt_prefixes,
        "user_prompt_prefixes": user_prompt_prefixes,
        "contains_texts": contains_texts,
        "thread_ids": sorted(thread_ids),
        "excluded_thread_ids": sorted(excluded_thread_ids),
        "candidate_threads": [candidate_to_dict(item) for item in candidates],
        "selected_threads": [candidate_to_dict(item) for item in selected],
        "medium_not_selected": [candidate_to_dict(item) for item in medium_not_selected],
        "excluded_threads": [candidate_to_dict(item) for item in protected],
        "session_files": session_files,
        "history": history,
        "session_index": session_index,
        "state_db": state_counts,
        "logs_db": log_counts,
        "memory": memory,
        "artifacts": artifacts,
        "automation": automation,
        "quarantine_hint": str(codex_home / "prune-quarantine" / f"{manifest_id}-quarantine"),
    }
    json_path = output_dir / f"{report_name}.json"
    md_path = output_dir / f"{report_name}.md"
    manifest["manifest_path"] = str(json_path)
    manifest["report_path"] = str(md_path)
    json_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    md_path.write_text(render_markdown(manifest))

    print(json.dumps({
        "manifest": str(json_path),
        "report": str(md_path),
        "scope": args.scope,
        "artifact_families": families,
        "selected_threads": len(selected),
        "medium_not_selected": len(medium_not_selected),
        "excluded_threads": len(protected),
        "automation_eligible_threads": len(automation["eligible_thread_ids"]),
        "manifest_id": manifest_id,
    }, indent=2))
    return 0


def command_apply(args: argparse.Namespace, codex_home: Path) -> int:
    manifest_path = Path(args.manifest).expanduser().resolve()
    manifest = load_manifest(manifest_path)
    if args.confirm != manifest["manifest_id"]:
        raise CleanupError("confirmation does not match manifest_id", 3)
    if args.memory_policy == "move" and args.confirm_memory_move != manifest["manifest_id"]:
        raise CleanupError("memory move requires --confirm-memory-move <manifest-id>", 3)
    if args.memory_policy == "ignore" and args.confirm_memory_ignore != manifest["manifest_id"]:
        raise CleanupError("memory ignore requires --confirm-memory-ignore <manifest-id>", 3)

    selected = manifest.get("selected_threads", [])
    if not selected:
        print(json.dumps({
            "dry_run": not args.execute,
            "noop": True,
            "manifest_id": manifest["manifest_id"],
            "reason": "manifest has no selected threads",
        }, indent=2))
        return 0

    selected_ids = {item["id"] for item in selected}
    quarantine_dir = (
        Path(args.quarantine_dir).expanduser().resolve()
        if args.quarantine_dir
        else Path(manifest.get("quarantine_hint") or codex_home / "prune-quarantine" / f"{manifest['manifest_id']}-quarantine")
    )
    preflight = preflight_apply(
        manifest,
        manifest_path,
        codex_home,
        selected_ids,
        args.max_manifest_age_hours,
        args.artifact_policy,
    )

    planned = {
        "manifest_id": manifest["manifest_id"],
        "execute": args.execute,
        "quarantine": str(quarantine_dir),
        "selected_threads": len(selected_ids),
        "memory_policy": args.memory_policy,
        "artifact_policy": args.artifact_policy,
        "preflight": preflight,
    }
    if not args.execute:
        print(json.dumps({"dry_run": True, **planned}, indent=2, sort_keys=True))
        return 0

    quarantine_dir.mkdir(parents=True, exist_ok=False)
    for name in ["db_backups", "session_files", "memory_rollout_summaries", "artifact_files", "manifests"]:
        (quarantine_dir / name).mkdir()
    shutil.copy2(manifest_path, quarantine_dir / "manifests" / "manifest.json")

    result: dict[str, Any] = {
        "manifest_id": manifest["manifest_id"],
        "quarantine": str(quarantine_dir),
        "preflight": preflight,
    }
    result["backups"] = backup_core_files(codex_home, quarantine_dir)
    result["backup_sha256"] = write_sha256s(quarantine_dir)
    verify_sha256s(quarantine_dir)
    result["session_files"] = quarantine_session_files(manifest, quarantine_dir)
    result["history"] = rewrite_jsonl(codex_home / "history.jsonl", "session_id", selected_ids)
    result["session_index"] = rewrite_jsonl(codex_home / "session_index.jsonl", "id", selected_ids)
    result["state_db"] = mutate_state_db(codex_home / "state_5.sqlite", selected_ids)
    result["logs_db"] = mutate_logs_db(codex_home / "logs_2.sqlite", selected_ids)
    result["memory"] = handle_memory_files(manifest, quarantine_dir, args.memory_policy)
    result["artifacts"] = handle_artifact_files(manifest, quarantine_dir, args.artifact_policy)
    result["sha256"] = write_sha256s(quarantine_dir)
    (quarantine_dir / "manifests" / "result.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n"
    )
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def command_restore(args: argparse.Namespace, codex_home: Path) -> int:
    quarantine = Path(args.quarantine).expanduser().resolve()
    require_dir(quarantine, "quarantine")
    result_path = quarantine / "manifests" / "result.json"
    if not result_path.exists():
        raise CleanupError("result.json not found in quarantine", 3)
    result = json.loads(result_path.read_text())
    restored: dict[str, Any] = {"execute": args.execute, "quarantine": str(quarantine)}
    if not args.execute:
        print(json.dumps({"dry_run": True, **restored}, indent=2))
        return 0
    verify_sha256s(quarantine)
    restored["session_files"] = restore_session_files(result)
    restored["memory_files"] = restore_memory_files(result)
    restored["artifacts"] = restore_artifact_files(result)
    if args.restore_db:
        restored["db_backups"] = restore_core_files(codex_home, quarantine)
    print(json.dumps(restored, indent=2, sort_keys=True))
    return 0


def command_purge(args: argparse.Namespace, codex_home: Path) -> int:
    root = codex_home / "prune-quarantine"
    if not root.exists():
        print(json.dumps({"purged": []}, indent=2))
        return 0
    if args.quarantine:
        victims = resolve_exact_quarantine_victims(root, args.quarantine)
    else:
        if args.older_than_days is None:
            raise CleanupError("purge requires --older-than-days or --quarantine", 2)
        cutoff = dt.datetime.now(dt.UTC).timestamp() - (args.older_than_days * 86400)
        victims = [
            path
            for path in root.iterdir()
            if path.is_dir()
            and path.stat().st_mtime < cutoff
            and path.name != "reports"
            and valid_quarantine(path)
        ]
    victims = sorted(victims, key=lambda path: path.name)
    victim_names = ",".join(path.name for path in victims) or "none"
    expected_confirm = f"purge:{victim_names}"
    if args.confirm != expected_confirm:
        raise CleanupError(f"purge requires --confirm {expected_confirm}", 3)
    if not args.execute:
        print(json.dumps({"dry_run": True, "purge_candidates": [str(path) for path in victims]}, indent=2))
        return 0
    for path in victims:
        verify_sha256s(path)
        shutil.rmtree(path)
    print(json.dumps({"purged": [str(path) for path in victims]}, indent=2))
    return 0


def resolve_exact_quarantine_victims(root: Path, values: list[str]) -> list[Path]:
    victims: list[Path] = []
    for value in values:
        raw = Path(value).expanduser()
        path = raw.resolve() if raw.is_absolute() else (root / raw).resolve()
        if path.parent != root.resolve():
            raise CleanupError(f"quarantine is outside prune-quarantine root: {path}", 3)
        if path.name == "reports":
            raise CleanupError("reports directory cannot be purged as a quarantine", 3)
        if not valid_quarantine(path):
            raise CleanupError(f"invalid quarantine bundle: {path}", 3)
        if path not in victims:
            victims.append(path)
    return victims


def normalize_families(values: list[str] | None) -> list[str]:
    if not values:
        return [SESSION_FAMILY, MEMORY_FAMILY]
    out = []
    for value in values:
        if value not in out:
            out.append(value)
    if SESSION_FAMILY not in out and MEMORY_FAMILY not in out:
        out.append(MEMORY_FAMILY)
    return out


def effective_min_age_hours(scope: str, requested: int | None) -> int:
    if requested is not None:
        return requested
    if scope in {"all", "codex-home", "cwd-subrepos"}:
        return BROAD_MIN_AGE_HOURS
    return DEFAULT_MIN_AGE_HOURS


def resolve_scope_roots(args: argparse.Namespace, codex_home: Path) -> list[ScopeRoot]:
    cwd = Path(args.cwd).expanduser().resolve()
    if args.scope == "current":
        root = resolve_target_root(cwd)
        return [ScopeRoot(label=root.name or "root", path=root, root_kind="current", match_mode="descendant")]
    if args.scope in {"root", "roots"}:
        paths = [Path(value).expanduser().resolve() for value in args.root]
        if args.roots_file:
            paths.extend(load_roots_file(Path(args.roots_file).expanduser().resolve()))
        if not paths:
            raise CleanupError(f"--scope {args.scope} requires --root or --roots-file", 2)
        return [
            ScopeRoot(label=path.name or f"root-{index}", path=path, root_kind="explicit", match_mode="descendant")
            for index, path in enumerate(dedupe_paths(paths), start=1)
        ]
    if args.scope == "cwd-subrepos":
        roots = discover_git_roots(cwd)
        return [
            ScopeRoot(label=path.name or f"repo-{index}", path=path, root_kind="subrepo", match_mode="descendant")
            for index, path in enumerate(roots, start=1)
        ]
    if args.scope == "codex-home":
        return [ScopeRoot(label="codex-home", path=codex_home, root_kind="codex_home", match_mode="codex-home")]
    if args.scope == "all":
        return [ScopeRoot(label="all", path=None, root_kind="global", match_mode="global")]
    raise CleanupError(f"unknown scope: {args.scope}", 2)


def load_roots_file(path: Path) -> list[Path]:
    if not path.exists():
        raise CleanupError(f"roots file not found: {path}", 2)
    text = path.read_text(encoding="utf8")
    if path.suffix == ".json":
        data = json.loads(text)
        if not isinstance(data, list):
            raise CleanupError("JSON roots file must contain a list", 2)
        return [Path(str(item)).expanduser().resolve() for item in data]
    roots = []
    for line in text.splitlines():
        value = line.strip()
        if not value or value.startswith("#"):
            continue
        if path.suffix == ".jsonl":
            row = json.loads(value)
            value = row.get("path") or row.get("root") or row.get("cwd")
            if not value:
                continue
        roots.append(Path(value).expanduser().resolve())
    return roots


def dedupe_paths(paths: list[Path]) -> list[Path]:
    out = []
    seen = set()
    for path in paths:
        real = str(path.resolve())
        if real not in seen:
            out.append(Path(real))
            seen.add(real)
    return out


def discover_git_roots(cwd: Path) -> list[Path]:
    roots = []
    for root, dirnames, _filenames in os.walk(cwd):
        if ".git" in dirnames:
            roots.append(Path(root).resolve())
            dirnames.remove(".git")
        dirnames[:] = [
            dirname
            for dirname in dirnames
            if dirname not in SUBREPO_SKIP_DIRS and not dirname.endswith(".egg-info")
        ]
    if (cwd / ".git").is_dir():
        roots.append(cwd)
    roots = dedupe_paths(roots)
    roots.sort(key=lambda path: (len(path.parts), str(path)))
    return roots or [resolve_target_root(cwd)]


def scope_threads(
    threads: list[ThreadRow],
    roots: list[ScopeRoot],
    scope: str,
    include_parent_overlap: bool,
) -> list[tuple[ThreadRow, str | None]]:
    if scope == "all":
        return [(row, "all") for row in threads]
    if scope == "codex-home":
        return [(row, "codex-home") for row in threads if not row.cwd or cwd_in_scope(row.cwd, roots[0].path or Path.home())]

    out = []
    for row in threads:
        matches = [root for root in roots if root.path and cwd_in_scope(row.cwd, root.path)]
        if not matches:
            continue
        if include_parent_overlap:
            for root in matches:
                out.append((row, root.label))
        else:
            root = max(matches, key=lambda item: len((item.path or Path()).parts))
            out.append((row, root.label))
    return out


def load_threads(codex_home: Path) -> list[ThreadRow]:
    db = codex_home / "state_5.sqlite"
    if db.exists():
        conn = connect_sqlite(db, readonly=True)
        conn.row_factory = sqlite3.Row
        try:
            rows = conn.execute(
                "select id, rollout_path, created_at, updated_at, cwd, title, tokens_used, "
                "has_user_event, archived, first_user_message, agent_role, agent_path from threads"
            ).fetchall()
            return [
                ThreadRow(
                    id=row["id"],
                    rollout_path=row["rollout_path"],
                    created_at=int(row["created_at"] or 0),
                    updated_at=int(row["updated_at"] or 0),
                    cwd=row["cwd"] or "",
                    title=row["title"] or "",
                    tokens_used=int(row["tokens_used"] or 0),
                    has_user_event=int(row["has_user_event"] or 0),
                    archived=int(row["archived"] or 0),
                    first_user_message=row["first_user_message"] or "",
                    agent_role=row["agent_role"],
                    agent_path=row["agent_path"],
                )
                for row in rows
            ]
        finally:
            conn.close()
    return load_threads_from_session_files(codex_home)


def load_threads_from_session_files(codex_home: Path) -> list[ThreadRow]:
    out = []
    for file in list_session_files(codex_home):
        thread_id = extract_id_from_filename(file.name)
        if not thread_id:
            continue
        stat = file.stat()
        title = read_first_user_text(file)
        out.append(
            ThreadRow(
                id=thread_id,
                rollout_path=str(file),
                created_at=int(stat.st_ctime),
                updated_at=int(stat.st_mtime),
                cwd="",
                title=title,
                tokens_used=0,
                has_user_event=1 if title else 0,
                archived=1 if "archived_sessions" in file.parts else 0,
                first_user_message=title,
            )
        )
    return out


def collect_active_thread_risks(codex_home: Path, now: int, min_age_hours: int) -> dict[str, list[str]]:
    db = codex_home / "state_5.sqlite"
    risks: dict[str, list[str]] = {}
    if not db.exists():
        return risks
    conn = connect_sqlite(db, readonly=True)
    conn.row_factory = sqlite3.Row
    try:
        tables = set(table_names(conn))
        if "thread_goals" in tables:
            for row in conn.execute("select thread_id, status from thread_goals where status != 'complete'"):
                risks.setdefault(row["thread_id"], []).append(f"protected-active-goal-{row['status']}")
        active_children: set[str] = set()
        if "agent_jobs" in tables and "agent_job_items" in tables:
            active_jobs = [
                row["id"]
                for row in conn.execute(
                    "select id from agent_jobs where status not in ('completed','failed','cancelled','canceled')"
                )
            ]
            for job_id in active_jobs:
                for row in conn.execute(
                    "select assigned_thread_id from agent_job_items where job_id = ? and assigned_thread_id is not null",
                    (job_id,),
                ):
                    active_children.add(row["assigned_thread_id"])
                    risks.setdefault(row["assigned_thread_id"], []).append("protected-active-agent-job")
        if "thread_spawn_edges" in tables:
            recent_cutoff = now - (min_age_hours * 3600)
            recent = {
                row["id"]
                for row in conn.execute("select id from threads where updated_at >= ?", (recent_cutoff,))
            } if "threads" in tables else set()
            active = set(risks) | active_children | recent
            for row in conn.execute("select parent_thread_id, child_thread_id, status from thread_spawn_edges"):
                parent = row["parent_thread_id"]
                child = row["child_thread_id"]
                status = row["status"] or ""
                if parent in active or child in active or status not in {"completed", "complete", "shutdown", "closed"}:
                    risks.setdefault(parent, []).append("protected-spawn-edge-active")
                    risks.setdefault(child, []).append("protected-spawn-edge-active")
    except sqlite3.Error:
        return risks
    finally:
        conn.close()
    return {key: sorted(set(value)) for key, value in risks.items()}


def classify_thread(
    row: ThreadRow,
    now: int,
    min_age_hours: int,
    active_reasons: list[str] | None = None,
    scope_label: str | None = None,
) -> Candidate:
    text = f"{row.title}\n{row.first_user_message}".lower()
    role = (row.agent_role or row.agent_path or "").lower()
    item = Candidate(thread=row, confidence=None, scope_label=scope_label)

    high_patterns = [
        ("gh-pr-review-fix-title", r"\$?gh-pr-review-fix|review-pack"),
        ("resolve-pr-review-comments-title", r"/resolve-pr-review-comments\b"),
        ("local-code-review-title", r"review the current code changes|staged, unstaged, and untracked"),
        ("hosted-review-fix-title", r"review comments|review remediation|address.*review|fix.*review"),
        ("third-party-review-title", r"coderabbit|sourcery|copilot review|code rabbit"),
        ("ci-triage-title", r"\bci\b.*(triage|fix|failure)|failing checks|workflow run"),
    ]
    medium_patterns = [
        ("stale-exploratory-title", r"\b(test|scratch|probe|experiment|temp|temporary)\b"),
    ]

    for reason, pattern in high_patterns:
        if re.search(pattern, text):
            item.reasons.append(reason)
            item.confidence = HIGH
    if "reviewer" in role or "triager" in role or "auditor" in role:
        item.reasons.append("reviewer-agent-role")
        item.confidence = HIGH
    if not row.has_user_event:
        item.reasons.append("no-user-event")
        item.confidence = item.confidence or MEDIUM
    if row.tokens_used and row.tokens_used <= 5000:
        item.reasons.append("short-low-token-session")
        item.confidence = item.confidence or MEDIUM
    for reason, pattern in medium_patterns:
        if re.search(pattern, text):
            item.reasons.append(reason)
            item.confidence = item.confidence or MEDIUM

    item.age_hours = (now - row.updated_at) / 3600 if row.updated_at else 999999
    if item.age_hours < min_age_hours:
        item.protected_reasons.append("protected-recent")
    if looks_like_main_dev(row) and "resolve-pr-review-comments-title" not in item.reasons:
        item.protected_reasons.append("protected-main-dev-likely")
    if not row.id:
        item.protected_reasons.append("protected-missing-id")
    if scope_label == "codex-home" and not row.cwd:
        item.protected_reasons.append("protected-unknown-cwd-codex-home")
    for reason in active_reasons or []:
        item.protected_reasons.append(reason)
    item.protected_reasons = sorted(set(item.protected_reasons))
    item.reasons = sorted(set(item.reasons))
    item.risk_score = score_candidate(item)
    return item


def thread_matches_first_prompt_prefix(row: ThreadRow, prefixes: list[str]) -> bool:
    prompt = row.first_user_message or row.title
    if any(prompt.startswith(prefix) for prefix in prefixes):
        return True
    normalized_prompt = normalize_prompt_start(prompt)
    return any(normalized_prompt.startswith(normalize_prompt_start(prefix)) for prefix in prefixes)


def mark_user_prompt_prefix_matches(candidates: list[Candidate], prefixes: list[str]) -> list[Candidate]:
    matched: list[Candidate] = []
    for item in candidates:
        if not session_file_has_user_prompt_prefix(Path(item.thread.rollout_path), prefixes):
            continue
        item.confidence = HIGH
        item.reasons.append("user-prompt-prefix-match")
        item.reasons = sorted(set(item.reasons))
        item.protected_reasons = [
            reason
            for reason in item.protected_reasons
            if reason != "protected-main-dev-likely"
        ]
        item.risk_score = score_candidate(item)
        matched.append(item)
    return matched


def mark_contains_text_matches(candidates: list[Candidate], texts: list[str]) -> list[Candidate]:
    matched: list[Candidate] = []
    for item in candidates:
        if not session_file_contains_text(Path(item.thread.rollout_path), texts):
            continue
        item.confidence = HIGH
        item.reasons.append("session-file-contains-text")
        item.reasons = sorted(set(item.reasons))
        item.protected_reasons = [
            reason
            for reason in item.protected_reasons
            if reason in {"protected-active-agent-job", "protected-spawn-edge-active"}
        ]
        item.risk_score = score_candidate(item)
        matched.append(item)
    return matched


def mark_exact_thread_id_matches(candidates: list[Candidate], ids: set[str]) -> list[Candidate]:
    matched: list[Candidate] = []
    for item in candidates:
        if item.thread.id not in ids:
            continue
        item.confidence = HIGH
        item.reasons.append("exact-thread-id-match")
        item.reasons = sorted(set(item.reasons))
        item.protected_reasons = [
            reason
            for reason in item.protected_reasons
            if reason in {"protected-active-agent-job", "protected-spawn-edge-active"}
        ]
        item.risk_score = score_candidate(item)
        matched.append(item)
    return matched


def session_file_contains_text(file: Path, texts: list[str]) -> bool:
    if not file.exists():
        return False
    try:
        content = file.read_text(encoding="utf8", errors="replace")
    except OSError:
        return False
    return any(text in content for text in texts)


def session_file_has_user_prompt_prefix(file: Path, prefixes: list[str]) -> bool:
    if not file.exists():
        return False
    try:
        with file.open("r", encoding="utf8", errors="replace") as handle:
            for line in handle:
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                for text in extract_user_texts(row):
                    if prompt_matches_prefix(text, prefixes):
                        return True
    except OSError:
        return False
    return False


def extract_user_texts(row: dict[str, Any]) -> list[str]:
    if row.get("type") == "message" and row.get("role") == "user":
        return texts_from_content(row.get("content"))
    if row.get("type") == "response_item":
        payload = row.get("payload") or {}
        if isinstance(payload, dict) and payload.get("type") == "message" and payload.get("role") == "user":
            return texts_from_content(payload.get("content"))
    if row.get("type") == "event_msg":
        payload = row.get("payload") or {}
        if isinstance(payload, dict) and payload.get("type") == "user_message":
            message = payload.get("message")
            return [message] if isinstance(message, str) else []
    return []


def texts_from_content(content: Any) -> list[str]:
    if isinstance(content, str):
        return [content]
    if not isinstance(content, list):
        return []
    texts: list[str] = []
    for part in content:
        if isinstance(part, str):
            texts.append(part)
        elif isinstance(part, dict):
            value = part.get("text") or part.get("input_text")
            if isinstance(value, str):
                texts.append(value)
    return texts


def prompt_matches_prefix(prompt: str, prefixes: list[str]) -> bool:
    if any(prompt.startswith(prefix) for prefix in prefixes):
        return True
    normalized_prompt = normalize_prompt_start(prompt)
    return any(normalized_prompt.startswith(normalize_prompt_start(prefix)) for prefix in prefixes)


def normalize_prompt_start(value: str) -> str:
    normalized = re.sub(r"\s+", " ", value.strip()).lower()
    return re.sub(r"\s*---\s*", "---", normalized)


def score_candidate(item: Candidate) -> int:
    if item.protected_reasons:
        return 100
    if item.confidence == HIGH:
        risk = 5
    elif item.confidence == MEDIUM:
        risk = 45
    else:
        risk = 100
    if item.thread.tokens_used > 200_000:
        risk += 20
    if "ci-triage-title" in item.reasons:
        risk += 5
    return min(risk, 100)


def looks_like_main_dev(row: ThreadRow) -> bool:
    text = f"{row.title}\n{row.first_user_message}".lower()
    implementation_words = [
        "implement",
        "build",
        "create a plan",
        "next branch",
        "ship",
        "feature",
        "refactor",
    ]
    if row.tokens_used > 1_000_000 and any(word in text for word in implementation_words):
        return True
    return False


def inspect_session_files(selected: list[Candidate]) -> list[dict[str, Any]]:
    files = []
    for item in selected:
        path = Path(item.thread.rollout_path)
        files.append(
            {
                "thread_id": item.thread.id,
                "path": str(path),
                "exists": path.exists(),
                "bytes": path.stat().st_size if path.exists() else 0,
                "scope_label": item.scope_label,
            }
        )
    return files


def inspect_jsonl_ids(file: Path, key: str, ids: set[str]) -> dict[str, Any]:
    if not file.exists():
        return {"path": str(file), "exists": False, "matching_lines": 0, "total_lines": 0, "parse_errors": 0}
    total = 0
    matched = 0
    parse_errors = 0
    with file.open("r", encoding="utf8", errors="replace") as handle:
        for line in handle:
            total += 1
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                parse_errors += 1
                continue
            if row.get(key) in ids:
                matched += 1
    return {
        "path": str(file),
        "exists": True,
        "matching_lines": matched,
        "total_lines": total,
        "parse_errors": parse_errors,
    }


def inspect_state_counts(db: Path, ids: set[str]) -> dict[str, Any]:
    if not db.exists() or not ids:
        return {"path": str(db), "exists": db.exists(), "matching_rows": 0, "tables": {}}
    conn = connect_sqlite(db, readonly=True)
    try:
        integrity = conn.execute("pragma integrity_check").fetchone()[0]
        counts = count_matching_rows(conn, ids)
        return {
            "path": str(db),
            "exists": True,
            "integrity_check": integrity,
            "matching_rows": sum(counts.values()),
            "tables": counts,
        }
    finally:
        conn.close()


def inspect_log_counts(db: Path, ids: set[str]) -> dict[str, Any]:
    if not db.exists() or not ids:
        return {"path": str(db), "exists": db.exists(), "matching_rows": 0}
    conn = connect_sqlite(db, readonly=True)
    try:
        integrity = conn.execute("pragma integrity_check").fetchone()[0]
        count = count_where_in(conn, "logs", "thread_id", ids) if "logs" in table_names(conn) else 0
        return {"path": str(db), "exists": True, "integrity_check": integrity, "matching_rows": count}
    finally:
        conn.close()


def inspect_memory(codex_home: Path, ids: set[str], rollout_paths: set[str]) -> dict[str, Any]:
    registry = codex_home / "memories" / "MEMORY.md"
    summaries = find_memory_files(codex_home, ids, rollout_paths)
    triage = triage_memory_files(registry, summaries, ids, rollout_paths)
    return {
        "registry": {"path": str(registry), "exists": registry.exists()},
        "rollout_summaries": [str(path) for path in summaries],
        "triage": [item.to_dict() for item in triage],
        "policy_default": "copy",
        "active_memory_deleted": False,
    }


def empty_memory(codex_home: Path) -> dict[str, Any]:
    registry = codex_home / "memories" / "MEMORY.md"
    return {
        "registry": {"path": str(registry), "exists": registry.exists()},
        "rollout_summaries": [],
        "triage": [],
        "policy_default": "copy",
        "active_memory_deleted": False,
    }


def find_memory_files(codex_home: Path, ids: set[str], rollout_paths: set[str] | None = None) -> list[Path]:
    if not ids and not rollout_paths:
        return []
    root = codex_home / "memories" / "rollout_summaries"
    if not root.exists():
        return []
    found = []
    needles = set(ids) | set(rollout_paths or set())
    for file in root.glob("*.md"):
        try:
            text = file.read_text(encoding="utf8", errors="replace")
        except OSError:
            continue
        if any(needle and needle in text for needle in needles):
            found.append(file)
    return sorted(found)


def triage_memory_files(
    registry: Path,
    summaries: list[Path],
    ids: set[str],
    rollout_paths: set[str],
) -> list[MemoryFinding]:
    findings: list[MemoryFinding] = []
    if registry.exists():
        reasons, recommendation = classify_memory_text(registry.read_text(encoding="utf8", errors="replace"), ids, rollout_paths)
        if reasons:
            findings.append(MemoryFinding(registry, recommendation, ["registry-scan", *reasons], sorted(ids)))
    for path in summaries:
        text = path.read_text(encoding="utf8", errors="replace")
        reasons, recommendation = classify_memory_text(text, ids, rollout_paths)
        linked = sorted(thread_id for thread_id in ids if thread_id in text)
        findings.append(MemoryFinding(path, recommendation, reasons or ["linked-selected-thread"], linked))
    return findings


def classify_memory_text(text: str, ids: set[str], rollout_paths: set[str]) -> tuple[list[str], str]:
    lower = text.lower()
    reasons = []
    if any(thread_id in text for thread_id in ids):
        reasons.append("linked-selected-thread")
    if any(path and path in text for path in rollout_paths):
        reasons.append("linked-selected-rollout-path")
    durable_patterns = [
        r"\breusable knowledge\b",
        r"\brunbook\b",
        r"\bworkflow\b",
        r"\bmigration\b",
        r"\brelease\b",
        r"\bauth\b",
        r"\bvalidation\b",
        r"\bci\b",
        r"\bgovernance\b",
        r"\bskill\b",
    ]
    if any(re.search(pattern, lower) for pattern in durable_patterns):
        reasons.append("durable-memory-marker")
    if re.search(r"outdated|stale|conflict|conflicting|drift-prone|old pr|old review", lower):
        reasons.append("drift-or-conflict-marker")
    if re.search(r"docs:arch:validate|web:next:|(?:^|[^a-z])npx(?:[^a-z]|$)|(?:^|[^a-z])npm(?:[^a-z]|$)", lower):
        reasons.append("stale-command-marker")
    if "durable-memory-marker" in reasons:
        recommendation = "preserve"
    elif "drift-or-conflict-marker" in reasons or "stale-command-marker" in reasons:
        recommendation = "audit"
    elif reasons:
        recommendation = "copy"
    else:
        recommendation = "preserve"
    return sorted(set(reasons)), recommendation


def inspect_artifact_families(codex_home: Path, families: list[str], now: int, min_age_hours: int) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for family in families:
        if family in {SESSION_FAMILY, MEMORY_FAMILY}:
            continue
        findings: list[ArtifactFinding] = []
        for path in family_roots(codex_home, family):
            if not path.exists():
                continue
            protected = []
            if path.name in SECRET_OR_AUTH_NAMES:
                protected.append("protected-secret-or-auth")
            age_hours = (now - int(path.stat().st_mtime)) / 3600
            reasons = [f"age-hours:{age_hours:.1f}"]
            action = "report-only"
            if family in {"cache", "generated"} and age_hours >= min_age_hours:
                action = "manual-quarantine-candidate"
            if family == "quarantine":
                action = "manual-purge-candidate"
            findings.append(
                ArtifactFinding(
                    family=family,
                    path=path,
                    action=action,
                    reasons=reasons,
                    protected_reasons=protected,
                    bytes=path_size(path),
                )
            )
        out[family] = {
            "count": len(findings),
            "bytes": sum(item.bytes for item in findings),
            "items": [item.to_dict() for item in findings[:200]],
            "truncated": max(0, len(findings) - 200),
        }
    return out


def family_roots(codex_home: Path, family: str) -> list[Path]:
    if family == "quarantine":
        root = codex_home / "prune-quarantine"
        return [path for path in root.iterdir()] if root.exists() else []
    if family == "logs":
        return [path for path in [codex_home / "log", codex_home / "codex-tui.log"] if path.exists()]
    if family == "cache":
        return [path for path in [codex_home / "cache", codex_home / ".tmp", codex_home / "tmp", codex_home / "plugins" / "cache"] if path.exists()]
    if family == "generated":
        return [path for path in [codex_home / "generated_images"] if path.exists()]
    if family == "skills-agents-config":
        return [
            path
            for path in [
                codex_home / "skills",
                codex_home / "agents",
                codex_home / "agents-bak",
                codex_home / "config.toml",
                codex_home / "hooks.json",
                codex_home / "AGENTS.md",
            ]
            if path.exists()
        ]
    return []


def path_size(path: Path) -> int:
    if path.is_file():
        return path.stat().st_size
    return sum(item.stat().st_size for item in path.rglob("*") if item.is_file())


def build_automation_summary(
    selected: list[Candidate],
    memory: dict[str, Any],
    history: dict[str, Any],
    session_index: dict[str, Any],
    state_counts: dict[str, Any],
    log_counts: dict[str, Any],
) -> dict[str, Any]:
    blockers = []
    if history.get("parse_errors", 0) or session_index.get("parse_errors", 0):
        blockers.append("jsonl-parse-errors")
    for counts in [state_counts, log_counts]:
        integrity = counts.get("integrity_check")
        if integrity and integrity != "ok":
            blockers.append("sqlite-integrity-not-ok")
    durable_memory = any(
        item.get("recommendation") == "preserve" and item.get("linked_thread_ids")
        for item in memory.get("triage", [])
    )
    if durable_memory:
        blockers.append("linked-durable-memory")
    eligible = [
        item.thread.id
        for item in selected
        if item.confidence == HIGH
        and item.risk_score <= AUTO_RISK_LIMIT
        and item.age_hours >= BROAD_MIN_AGE_HOURS
        and not blockers
    ]
    return {
        "policy": "validator-consensus-required",
        "risk_limit": AUTO_RISK_LIMIT,
        "minimum_age_hours": BROAD_MIN_AGE_HOURS,
        "eligible_thread_ids": eligible,
        "manual_review_required_reasons": sorted(set(blockers)),
        "memory_policy_required": "copy",
    }


def preflight_apply(
    manifest: dict[str, Any],
    manifest_path: Path,
    codex_home: Path,
    selected_ids: set[str],
    max_manifest_age_hours: int,
    artifact_policy: str = "report",
) -> dict[str, Any]:
    generated_at = parse_iso(manifest.get("generated_at", ""))
    now = dt.datetime.now(dt.UTC)
    if generated_at and (now - generated_at).total_seconds() > max_manifest_age_hours * 3600:
        raise CleanupError("manifest is stale; rerun scan before apply", 3)
    if not manifest_path.exists():
        raise CleanupError("manifest path missing during preflight", 3)

    history = inspect_jsonl_ids(codex_home / "history.jsonl", "session_id", selected_ids)
    session_index = inspect_jsonl_ids(codex_home / "session_index.jsonl", "id", selected_ids)
    if history.get("parse_errors", 0) or session_index.get("parse_errors", 0):
        raise CleanupError("JSONL parse errors present; refusing apply", 4)
    state_counts = inspect_state_counts(codex_home / "state_5.sqlite", selected_ids)
    log_counts = inspect_log_counts(codex_home / "logs_2.sqlite", selected_ids)
    for label, counts in [("state DB", state_counts), ("logs DB", log_counts)]:
        integrity = counts.get("integrity_check")
        if integrity and integrity != "ok":
            raise CleanupError(f"{label} integrity check failed before apply: {integrity}", 4)

    active = collect_active_thread_risks(codex_home, int(now.timestamp()), manifest.get("min_age_hours", DEFAULT_MIN_AGE_HOURS))
    active_hits = {thread_id: active[thread_id] for thread_id in selected_ids if thread_id in active}
    if active_hits:
        raise CleanupError(f"selected threads have active state risk: {active_hits}", 4)

    missing = []
    unreadable = []
    for row in manifest.get("session_files", []):
        if not row.get("exists"):
            continue
        path = Path(row["path"])
        if not path.exists():
            missing.append(str(path))
        elif not os.access(path, os.R_OK | os.W_OK):
            unreadable.append(str(path))
    if missing:
        raise CleanupError(f"selected session files missing: {missing[:5]}", 4)
    if unreadable:
        raise CleanupError(f"selected session files unreadable/unwritable: {unreadable[:5]}", 4)
    artifact_candidates = selected_artifact_candidates(manifest)
    if artifact_policy == "quarantine":
        missing_artifacts = []
        unreadable_artifacts = []
        for row in artifact_candidates:
            path = Path(row["path"])
            if not path.exists():
                missing_artifacts.append(str(path))
            elif not os.access(path, os.R_OK | os.W_OK):
                unreadable_artifacts.append(str(path))
        if missing_artifacts:
            raise CleanupError(f"artifact candidates missing: {missing_artifacts[:5]}", 4)
        if unreadable_artifacts:
            raise CleanupError(f"artifact candidates unreadable/unwritable: {unreadable_artifacts[:5]}", 4)

    return {
        "manifest_age_hours": round((now - generated_at).total_seconds() / 3600, 3) if generated_at else None,
        "history_matching_lines": history.get("matching_lines", 0),
        "session_index_matching_lines": session_index.get("matching_lines", 0),
        "state_matching_rows": state_counts.get("matching_rows", 0),
        "logs_matching_rows": log_counts.get("matching_rows", 0),
        "integrity": {
            "state": state_counts.get("integrity_check"),
            "logs": log_counts.get("integrity_check"),
        },
        "artifact_policy": artifact_policy,
        "artifact_candidates": len(artifact_candidates),
    }


def parse_iso(value: str) -> dt.datetime | None:
    if not value:
        return None
    try:
        return dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def count_matching_rows(conn: sqlite3.Connection, ids: set[str]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for table in table_names(conn):
        columns = column_names(conn, table)
        table_count = 0
        if table == "threads" and "id" in columns:
            table_count += count_where_in(conn, table, "id", ids)
        for column in thread_id_columns(columns):
            table_count += count_where_in(conn, table, column, ids)
        if table_count:
            counts[table] = table_count
    return counts


def mutate_state_db(db: Path, ids: set[str]) -> dict[str, Any]:
    if not db.exists() or not ids:
        return {"exists": db.exists(), "deleted_rows": 0}
    conn = connect_sqlite(db, readonly=False)
    try:
        before = conn.execute("pragma integrity_check").fetchone()[0]
        if before != "ok":
            raise CleanupError(f"state DB integrity check failed before mutation: {before}", 4)
        deleted = delete_matching_rows(conn, ids)
        conn.commit()
        conn.execute("pragma wal_checkpoint(truncate)").fetchall()
        after = conn.execute("pragma integrity_check").fetchone()[0]
        if after != "ok":
            raise CleanupError(f"state DB integrity check failed after mutation: {after}", 4)
        return {"integrity_before": before, "integrity_after": after, "deleted_rows": deleted}
    finally:
        conn.close()


def mutate_logs_db(db: Path, ids: set[str]) -> dict[str, Any]:
    if not db.exists() or not ids:
        return {"exists": db.exists(), "deleted_rows": 0}
    conn = connect_sqlite(db, readonly=False)
    try:
        before = conn.execute("pragma integrity_check").fetchone()[0]
        if before != "ok":
            raise CleanupError(f"logs DB integrity check failed before mutation: {before}", 4)
        deleted = delete_where_in(conn, "logs", "thread_id", ids) if "logs" in table_names(conn) else 0
        conn.commit()
        conn.execute("pragma wal_checkpoint(truncate)").fetchall()
        after = conn.execute("pragma integrity_check").fetchone()[0]
        if after != "ok":
            raise CleanupError(f"logs DB integrity check failed after mutation: {after}", 4)
        return {"integrity_before": before, "integrity_after": after, "deleted_rows": deleted}
    finally:
        conn.close()


def delete_matching_rows(conn: sqlite3.Connection, ids: set[str]) -> int:
    deleted = 0
    for table in table_names(conn):
        columns = column_names(conn, table)
        if table == "threads" and "id" in columns:
            deleted += delete_where_in(conn, table, "id", ids)
        for column in thread_id_columns(columns):
            deleted += delete_where_in(conn, table, column, ids)
    return deleted


def rewrite_jsonl(file: Path, key: str, ids: set[str]) -> dict[str, Any]:
    if not file.exists() or not ids:
        return {"path": str(file), "exists": file.exists(), "removed": 0}
    removed = 0
    kept = 0
    parse_errors = 0
    tmp = file.with_suffix(file.suffix + ".tmp")
    with file.open("r", encoding="utf8", errors="replace") as src, tmp.open("w", encoding="utf8") as dst:
        for line in src:
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                parse_errors += 1
                dst.write(line)
                kept += 1
                continue
            if row.get(key) in ids:
                removed += 1
            else:
                dst.write(line)
                kept += 1
    os.replace(tmp, file)
    return {"path": str(file), "removed": removed, "kept": kept, "parse_errors": parse_errors}


def quarantine_session_files(manifest: dict[str, Any], quarantine_dir: Path) -> list[dict[str, Any]]:
    out = []
    dest_dir = quarantine_dir / "session_files"
    for row in manifest.get("session_files", []):
        src = Path(row["path"])
        if not src.exists():
            out.append({**row, "quarantined": False, "reason": "missing"})
            continue
        dest = unique_dest(dest_dir, src.name)
        shutil.copy2(src, dest)
        src.unlink()
        out.append({**row, "quarantined": True, "quarantine_path": str(dest)})
    return out


def handle_memory_files(manifest: dict[str, Any], quarantine_dir: Path, policy: str) -> dict[str, Any]:
    memory = manifest.get("memory", {})
    files = [Path(path) for path in memory.get("rollout_summaries", [])]
    registry_path = Path(memory.get("registry", {}).get("path", ""))
    out: dict[str, Any] = {"policy": policy, "files": [], "registry": None, "active_memory_deleted": False}
    if policy == "ignore":
        return out
    dest_dir = quarantine_dir / "memory_rollout_summaries"
    if registry_path and registry_path.exists():
        dest = unique_dest(dest_dir, registry_path.name)
        shutil.copy2(registry_path, dest)
        out["registry"] = {"path": str(registry_path), "copied_to": str(dest)}
    for src in files:
        row = {"path": str(src), "exists": src.exists()}
        if src.exists():
            dest = unique_dest(dest_dir, src.name)
            if policy == "move":
                shutil.move(str(src), dest)
                row["moved_to"] = str(dest)
                out["active_memory_deleted"] = True
            else:
                shutil.copy2(src, dest)
                row["copied_to"] = str(dest)
        out["files"].append(row)
    return out


def selected_artifact_candidates(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    out = []
    for family in manifest.get("artifacts", {}).values():
        for row in family.get("items", []):
            if row.get("action") != "manual-quarantine-candidate":
                continue
            if row.get("protected_reasons"):
                continue
            out.append(row)
    return out


def handle_artifact_files(manifest: dict[str, Any], quarantine_dir: Path, policy: str) -> dict[str, Any]:
    candidates = selected_artifact_candidates(manifest)
    out: dict[str, Any] = {"policy": policy, "files": []}
    if policy == "report":
        return out
    dest_dir = quarantine_dir / "artifact_files"
    for row in candidates:
        src = Path(row["path"])
        item = {**row, "exists": src.exists(), "quarantined": False}
        if not src.exists():
            item["reason"] = "missing"
            out["files"].append(item)
            continue
        family_dir = dest_dir / row.get("family", "unknown")
        family_dir.mkdir(parents=True, exist_ok=True)
        dest = unique_dest(family_dir, src.name)
        shutil.move(str(src), dest)
        item["quarantined"] = True
        item["quarantine_path"] = str(dest)
        out["files"].append(item)
    return out


def backup_core_files(codex_home: Path, quarantine_dir: Path) -> dict[str, Any]:
    backup_dir = quarantine_dir / "db_backups"
    names = [
        "state_5.sqlite",
        "state_5.sqlite-shm",
        "state_5.sqlite-wal",
        "logs_2.sqlite",
        "logs_2.sqlite-shm",
        "logs_2.sqlite-wal",
        "history.jsonl",
        "history.json",
        "session_index.jsonl",
    ]
    copied = []
    missing = []
    for name in names:
        src = codex_home / name
        if src.exists():
            shutil.copy2(src, backup_dir / name)
            copied.append(name)
        else:
            missing.append(name)
    required = [name for name in ["state_5.sqlite", "logs_2.sqlite", "history.jsonl", "session_index.jsonl"] if (codex_home / name).exists()]
    missing_required = [name for name in required if name not in copied]
    if missing_required:
        raise CleanupError(f"missing required backups: {missing_required}", 4)
    return {"copied": copied, "missing": missing, "path": str(backup_dir)}


def restore_core_files(codex_home: Path, quarantine: Path) -> list[str]:
    restored = []
    backup_dir = quarantine / "db_backups"
    for src in backup_dir.iterdir():
        dest = codex_home / src.name
        shutil.copy2(src, dest)
        restored.append(str(dest))
    return restored


def restore_session_files(result: dict[str, Any]) -> list[dict[str, Any]]:
    restored = []
    for row in result.get("session_files", []):
        qpath = row.get("quarantine_path")
        original = row.get("path")
        if not qpath or not original:
            continue
        src = Path(qpath)
        dest = Path(original)
        if src.exists() and not dest.exists():
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dest)
            restored.append({"path": str(dest), "restored": True})
    return restored


def restore_memory_files(result: dict[str, Any]) -> list[dict[str, Any]]:
    restored = []
    memory = result.get("memory", {})
    rows = list(memory.get("files", []))
    if memory.get("registry"):
        rows.append(memory["registry"])
    for row in rows:
        qpath = row.get("moved_to") or row.get("copied_to")
        original = row.get("path")
        if not qpath or not original:
            continue
        src = Path(qpath)
        dest = Path(original)
        if src.exists() and not dest.exists():
            dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dest)
            restored.append({"path": str(dest), "restored": True})
    return restored


def restore_artifact_files(result: dict[str, Any]) -> list[dict[str, Any]]:
    restored = []
    for row in result.get("artifacts", {}).get("files", []):
        qpath = row.get("quarantine_path")
        original = row.get("path")
        out = {**row, "restored": False}
        if not qpath or not original:
            out["reason"] = "not-quarantined"
            restored.append(out)
            continue
        src = Path(qpath)
        dest = Path(original)
        if not src.exists():
            out["reason"] = "missing-quarantine-path"
            restored.append(out)
            continue
        if dest.exists():
            out["reason"] = "destination-exists"
            restored.append(out)
            continue
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(src), dest)
        out["restored"] = True
        restored.append(out)
    return restored


def render_markdown(manifest: dict[str, Any]) -> str:
    selected = manifest["selected_threads"]
    medium = manifest["medium_not_selected"]
    excluded = manifest["excluded_threads"]
    session_file_count = sum(1 for row in manifest["session_files"] if row["exists"])
    memory_count = len(manifest["memory"]["rollout_summaries"])
    automation = manifest.get("automation", {})
    apply_command = (
        "python3 /home/bjorn/.agents/skills/codex-session-cleanup/scripts/codex_session_cleanup.py apply "
        f"--manifest {manifest.get('manifest_path', '/path/to/manifest.json')} "
        f"--confirm {manifest['manifest_id']} --memory-policy copy --execute"
    )
    if selected_artifact_candidates(manifest):
        apply_command = apply_command.replace(" --execute", " --artifact-policy quarantine --execute")
    restore_command = (
        "python3 /home/bjorn/.agents/skills/codex-session-cleanup/scripts/codex_session_cleanup.py restore "
        f"--quarantine {manifest.get('quarantine_hint', '/path/to/quarantine')} --restore-db --execute"
    )
    lines = [
        f"# Codex Session Cleanup Scan: {manifest['manifest_id']}",
        "",
        f"Generated: {manifest['generated_at']}",
        "",
        "## Summary",
        "",
        f"- Scope: `{manifest['scope']}`",
        f"- Scope roots: {len(manifest.get('scope_roots', []))}",
        f"- Artifact families: `{', '.join(manifest.get('artifact_families', []))}`",
        f"- Selected threads: {len(selected)}",
        f"- Medium-confidence not selected: {len(medium)}",
        f"- Protected/excluded: {len(excluded)}",
        f"- Session files to quarantine: {session_file_count}",
        f"- History rows to remove: {manifest['history'].get('matching_lines', 0)}",
        f"- Session index rows to remove: {manifest['session_index'].get('matching_lines', 0)}",
        f"- State DB rows to remove: {manifest['state_db'].get('matching_rows', 0)}",
        f"- Log rows to remove: {manifest['logs_db'].get('matching_rows', 0)}",
        f"- Linked memory summaries: {memory_count}",
        f"- Automation-eligible threads: {len(automation.get('eligible_thread_ids', []))}",
        "",
        "## Scope Roots",
        "",
        *format_scope_roots(manifest.get("scope_roots", [])),
        "",
        "## Selected Threads",
        "",
        *format_thread_list(selected),
        "",
        "## Medium Confidence Not Selected",
        "",
        *format_thread_list(medium),
        "",
        "## Protected Or Excluded",
        "",
        *format_thread_list(excluded, protected=True),
        "",
        "## Memory Triage",
        "",
        *format_memory_findings(manifest.get("memory", {}).get("triage", [])),
        "",
        "## Artifact Families",
        "",
        *format_artifact_summary(manifest.get("artifacts", {})),
        "",
        "## Automation Policy",
        "",
        f"- Policy: `{automation.get('policy', 'manual-review')}`",
        f"- Memory policy required: `{automation.get('memory_policy_required', 'copy')}`",
        f"- Manual review reasons: `{', '.join(automation.get('manual_review_required_reasons', [])) or 'none'}`",
        "",
        "## Apply",
        "",
        "Apply is quarantine-first. Memory defaults to copy; moving or ignoring memory requires an extra manifest-id confirmation.",
        "",
        f"```bash\n{apply_command}\n```",
        "",
        "## Restore",
        "",
        f"```bash\n{restore_command}\n```",
        "",
    ]
    return "\n".join(lines)


def format_scope_roots(items: list[dict[str, Any]]) -> list[str]:
    if not items:
        return ["- None"]
    return [f"- `{item.get('label')}` {item.get('root_kind')} {item.get('path')}" for item in items]


def format_thread_list(items: list[dict[str, Any]], protected: bool = False) -> list[str]:
    if not items:
        return ["- None"]
    rows = []
    for item in items[:100]:
        reasons = item.get("protected_reasons" if protected else "reasons", [])
        rows.append(
            f"- `{item['id']}` {item.get('confidence') or ''} risk={item.get('risk_score')} "
            f"scope={item.get('scope_label') or '-'} {', '.join(reasons)} - {item['title'][:160]}"
        )
    if len(items) > 100:
        rows.append(f"- ... {len(items) - 100} more")
    return rows


def format_memory_findings(items: list[dict[str, Any]]) -> list[str]:
    if not items:
        return ["- None"]
    rows = []
    for item in items[:100]:
        rows.append(
            f"- `{item.get('recommendation')}` {Path(item.get('path', '')).name}: "
            f"{', '.join(item.get('reasons', []))}"
        )
    if len(items) > 100:
        rows.append(f"- ... {len(items) - 100} more")
    return rows


def format_artifact_summary(items: dict[str, Any]) -> list[str]:
    if not items:
        return ["- None"]
    return [
        f"- `{family}` count={summary.get('count', 0)} bytes={summary.get('bytes', 0)} truncated={summary.get('truncated', 0)}"
        for family, summary in sorted(items.items())
    ]


def candidate_to_dict(item: Candidate) -> dict[str, Any]:
    return {
        "id": item.thread.id,
        "cwd": item.thread.cwd,
        "title": item.thread.title,
        "rollout_path": item.thread.rollout_path,
        "updated_at": item.thread.updated_at,
        "tokens_used": item.thread.tokens_used,
        "confidence": item.confidence,
        "reasons": item.reasons,
        "protected_reasons": item.protected_reasons,
        "scope_label": item.scope_label,
        "risk_score": item.risk_score,
        "age_hours": round(item.age_hours, 3),
        "automation_eligible_by_script": item.confidence == HIGH
        and item.risk_score <= AUTO_RISK_LIMIT
        and item.age_hours >= BROAD_MIN_AGE_HOURS
        and not item.protected_reasons,
    }


def resolve_target_root(cwd: Path) -> Path:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(cwd), "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.DEVNULL,
        ).strip()
        if out:
            return Path(out).resolve()
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass
    return cwd


def cwd_in_scope(cwd: str, target_root: Path) -> bool:
    if not cwd:
        return False
    try:
        path = Path(cwd).resolve()
        return path == target_root or target_root in path.parents
    except OSError:
        return False


def list_session_files(codex_home: Path) -> list[Path]:
    files = []
    for root in [codex_home / "sessions", codex_home / "archived_sessions"]:
        if root.exists():
            files.extend(root.rglob("*.jsonl"))
    return sorted(files)


def read_first_user_text(file: Path) -> str:
    try:
        with file.open("r", encoding="utf8", errors="replace") as handle:
            for line in handle:
                try:
                    row = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if row.get("type") == "message" and row.get("role") == "user":
                    content = row.get("content") or []
                    if content and isinstance(content, list):
                        return " ".join(str(part.get("text", "")) for part in content if isinstance(part, dict))[:500]
    except OSError:
        pass
    return ""


def extract_id_from_filename(name: str) -> str | None:
    match = re.search(r"([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})", name)
    return match.group(1) if match else None


def connect_sqlite(db: Path, readonly: bool) -> sqlite3.Connection:
    if readonly:
        return sqlite3.connect(f"file:{db}?mode=ro", uri=True)
    return sqlite3.connect(db)


def table_names(conn: sqlite3.Connection) -> list[str]:
    return [
        row[0]
        for row in conn.execute("select name from sqlite_master where type='table'")
        if not row[0].startswith("_sqlx")
    ]


def column_names(conn: sqlite3.Connection, table: str) -> list[str]:
    return [row[1] for row in conn.execute(f"pragma table_info({quote_ident(table)})")]


def thread_id_columns(columns: list[str]) -> list[str]:
    allowed = {
        "thread_id",
        "assigned_thread_id",
        "parent_thread_id",
        "child_thread_id",
        "source_thread_id",
        "target_thread_id",
    }
    return [column for column in columns if column in allowed]


def count_where_in(conn: sqlite3.Connection, table: str, column: str, ids: set[str]) -> int:
    if not ids:
        return 0
    placeholders = ",".join("?" for _ in ids)
    sql = f"select count(*) from {quote_ident(table)} where {quote_ident(column)} in ({placeholders})"
    return int(conn.execute(sql, tuple(ids)).fetchone()[0])


def delete_where_in(conn: sqlite3.Connection, table: str, column: str, ids: set[str]) -> int:
    if not ids:
        return 0
    placeholders = ",".join("?" for _ in ids)
    sql = f"delete from {quote_ident(table)} where {quote_ident(column)} in ({placeholders})"
    cur = conn.execute(sql, tuple(ids))
    return int(cur.rowcount if cur.rowcount is not None else 0)


def quote_ident(value: str) -> str:
    return '"' + value.replace('"', '""') + '"'


def require_dir(path: Path, label: str) -> None:
    if not path.is_dir():
        raise CleanupError(f"{label} is not a directory: {path}", 2)


def load_manifest(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise CleanupError(f"manifest not found: {path}", 2)
    return json.loads(path.read_text())


def build_target_label(scope: str, roots: list[ScopeRoot]) -> str:
    if scope in {"all", "codex-home"}:
        return scope
    if len(roots) == 1:
        return roots[0].label
    digest = hashlib.sha1("|".join(str(root.path) for root in roots).encode("utf8")).hexdigest()[:8]
    return f"{scope}-{len(roots)}-{digest}"


def build_manifest_id(scope: str, target_label: str) -> str:
    stamp = dt.datetime.now(dt.UTC).strftime("%Y%m%dT%H%M%SZ")
    slug = re.sub(r"[^a-zA-Z0-9]+", "-", target_label).strip("-").lower() or scope
    return f"codex-session-cleanup-{slug}-{stamp}"


def iso_now() -> str:
    return dt.datetime.now(dt.UTC).isoformat().replace("+00:00", "Z")


def unique_dest(directory: Path, name: str) -> Path:
    candidate = directory / name
    if not candidate.exists():
        return candidate
    stem = candidate.stem
    suffix = candidate.suffix
    for index in range(1, 10000):
        next_candidate = directory / f"{stem}.{index}{suffix}"
        if not next_candidate.exists():
            return next_candidate
    raise CleanupError(f"could not allocate unique destination for {name}", 5)


def write_sha256s(root: Path) -> dict[str, Any]:
    rows = []
    for file in sorted(path for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS"):
        digest = sha256_file(file)
        rel = file.relative_to(root)
        rows.append((digest, str(rel)))
    (root / "SHA256SUMS").write_text("".join(f"{digest}  {rel}\n" for digest, rel in rows))
    return {"files": len(rows), "path": str(root / "SHA256SUMS")}


def verify_sha256s(root: Path) -> None:
    sums = root / "SHA256SUMS"
    if not sums.exists():
        raise CleanupError("SHA256SUMS missing", 4)
    for line in sums.read_text().splitlines():
        digest, rel = line.split("  ", 1)
        file = root / rel
        if not file.exists():
            raise CleanupError(f"checksum target missing: {rel}", 4)
        actual = sha256_file(file)
        if actual != digest:
            raise CleanupError(f"checksum mismatch for {rel}", 4)


def valid_quarantine(path: Path) -> bool:
    return (
        (path / "manifests" / "result.json").exists()
        and (path / "SHA256SUMS").exists()
    )


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


if __name__ == "__main__":
    sys.exit(main())
