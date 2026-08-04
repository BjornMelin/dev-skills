#!/usr/bin/env python3
"""Install the Claude Code subagent catalog with backups.

Mirrors ``subagents/codex/scripts/sync_agents.py`` for the Claude side: Codex roles are TOML
under ``~/.codex/agents``; Claude agents are Markdown with YAML frontmatter under
``~/.claude/agents`` (global) or ``<project>/.claude/agents`` (project).

Safety properties this installer guarantees:

* **No symlink traversal.** The target directory and every destination file are checked with
  ``lstat``. A symlinked target would write outside the intended tree; a symlinked destination
  would back up and then overwrite its referent. Both abort before any write.
* **Atomic replacement.** Files are staged in the target directory and ``os.replace``\\d into
  position, so an interrupted run never leaves a partially written agent. ``shutil.copy2``
  truncates the live destination and has no rollback.
* **Fail before the first write.** Validation and safety checks run over the whole catalog up
  front; a single failure aborts the run rather than leaving a half-installed set.

Home is always resolved at runtime via ``Path.home()``. This repository is public and
``tools/policy/check_public_leaks.py`` rejects a literal home path in any tracked file.
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
import tempfile
from dataclasses import dataclass
from datetime import datetime
from datetime import timezone
from pathlib import Path

try:
    import yaml
except ImportError:  # pragma: no cover - dependency is declared in requirements-ci.txt
    print("PyYAML is required: pip install pyyaml", file=sys.stderr)
    raise SystemExit(1)

ROOT = Path(__file__).resolve().parents[1]
AGENTS_ROOT = ROOT / "agents" / "global"

# Claude Code agent frontmatter. `model` and `effort` are required and pinned explicitly on
# every role: MODELS.md forbids inheriting a worker model or effort. This is the subset this
# pack uses, not the full set Claude Code accepts -- unknown keys are rejected so a typo
# cannot silently produce an agent that behaves differently than the file reads.
REQUIRED_KEYS = ("name", "description", "model", "effort")
ALLOWED_KEYS = {
    "name",
    "description",
    "model",
    "effort",
    "tools",
    "disallowedTools",
    "skills",
    "permissionMode",
    "maxTurns",
    "memory",
    "isolation",
    "background",
    "color",
}
STRING_KEYS = {"name", "description", "model", "effort", "permissionMode", "memory", "color"}
LIST_OR_STRING_KEYS = {"tools", "disallowedTools", "skills"}
VALID_EFFORTS = {"low", "medium", "high", "xhigh", "max"}
VALID_PERMISSION_MODES = {"default", "plan", "acceptEdits", "bypassPermissions"}
NAME_CHARS = set("abcdefghijklmnopqrstuvwxyz0123456789-")


def stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


@dataclass(frozen=True)
class Agent:
    path: Path
    name: str
    meta: dict


def parse(path: Path) -> tuple[Agent | None, list[str]]:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None, ["no YAML frontmatter"]
    end = text.find("\n---\n", 3)
    if end == -1:
        return None, ["unterminated YAML frontmatter"]
    try:
        meta = yaml.safe_load(text[4:end])
    except yaml.YAMLError as exc:
        return None, [f"malformed YAML: {str(exc).splitlines()[0]}"]
    if not isinstance(meta, dict):
        return None, ["frontmatter is not a mapping"]
    return Agent(path=path, name=str(meta.get("name", "")), meta=meta), []


def validate(agent: Agent) -> list[str]:
    errors = []
    meta = agent.meta

    for key in REQUIRED_KEYS:
        if not meta.get(key):
            errors.append(f"missing '{key}'")

    unknown = set(meta) - ALLOWED_KEYS
    if unknown:
        errors.append(f"unexpected key(s): {', '.join(sorted(unknown))}")

    for key in STRING_KEYS & set(meta):
        if not isinstance(meta[key], str):
            errors.append(f"'{key}' must be a string, got {type(meta[key]).__name__}")
    for key in LIST_OR_STRING_KEYS & set(meta):
        if not isinstance(meta[key], (str, list)):
            errors.append(f"'{key}' must be a string or list, got {type(meta[key]).__name__}")
    if "maxTurns" in meta and not isinstance(meta["maxTurns"], int):
        errors.append("'maxTurns' must be an integer")

    name = meta.get("name")
    if isinstance(name, str) and name:
        if set(name) - NAME_CHARS:
            errors.append(f"name '{name}' must be lowercase letters, digits, and hyphens only")
        if name != agent.path.stem:
            errors.append(f"name '{name}' must match filename '{agent.path.stem}'")

    if meta.get("model") == "inherit":
        errors.append("model must be pinned explicitly, never 'inherit'")

    effort = meta.get("effort")
    if effort is not None and effort not in VALID_EFFORTS:
        errors.append(f"invalid effort '{effort}'; expected one of {sorted(VALID_EFFORTS)}")

    mode = meta.get("permissionMode")
    if mode is not None and mode not in VALID_PERMISSION_MODES:
        errors.append(f"invalid permissionMode '{mode}'")

    return errors


def discover() -> tuple[list[Agent], list[str]]:
    agents, errors = [], []
    for path in sorted(AGENTS_ROOT.glob("*.md")):
        agent, parse_errors = parse(path)
        if agent is None:
            errors += [f"{path.name}: {e}" for e in parse_errors]
            continue
        agents.append(agent)
        errors += [f"{path.name}: {e}" for e in validate(agent)]
    names = [a.name for a in agents]
    for dupe in {n for n in names if names.count(n) > 1}:
        errors.append(f"duplicate agent name '{dupe}'; the loader silently discards one")
    return agents, errors


def check_target_safe(target: Path) -> list[str]:
    """Refuse to install through a symlink, at the directory or the file level."""
    errors = []
    if target.is_symlink():
        errors.append(f"target directory is a symlink: {target} -> {os.readlink(target)}")
    elif target.exists() and not target.is_dir():
        errors.append(f"target exists and is not a directory: {target}")
    for parent in list(target.parents)[:2]:
        if parent.is_symlink():
            errors.append(f"target parent is a symlink: {parent}")
    return errors


def install(agents: list[Agent], target: Path, dry_run: bool) -> int:
    unsafe = check_target_safe(target)
    for dst in (target / a.path.name for a in agents):
        if dst.is_symlink():
            unsafe.append(f"destination is a symlink: {dst} -> {os.readlink(dst)}")
    if unsafe:
        for error in unsafe:
            print(f"UNSAFE {error}", file=sys.stderr)
        print("aborting before any write", file=sys.stderr)
        return 1

    if not dry_run:
        target.mkdir(parents=True, exist_ok=True)

    backup_dir = target.parent / "agent-backups" / f"claude-{stamp()}"
    wrote = False
    for agent in agents:
        dst = target / agent.path.name
        payload = agent.path.read_bytes()
        if dst.exists() and dst.read_bytes() == payload:
            print(f"  unchanged  {agent.name}")
            continue
        if dry_run:
            print(f"  would sync {agent.name} -> {dst}")
            continue
        if dst.exists():
            backup_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(dst, backup_dir / dst.name)
        # stage beside the target so os.replace is atomic (same filesystem), then swap
        fd, tmp = tempfile.mkstemp(dir=target, prefix=f".{dst.name}.", suffix=".tmp")
        try:
            with os.fdopen(fd, "wb") as handle:
                handle.write(payload)
                handle.flush()
                os.fsync(handle.fileno())
            shutil.copystat(agent.path, tmp)
            os.replace(tmp, dst)
        except BaseException:
            Path(tmp).unlink(missing_ok=True)
            raise
        wrote = True
        print(f"  synced     {agent.name} -> {dst}")

    if wrote and backup_dir.exists():
        print(f"backups: {backup_dir}")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--target", choices=("global", "project"), default="global")
    parser.add_argument("--project-dir", type=Path, default=Path.cwd())
    parser.add_argument("--list", action="store_true", help="list the catalog and exit")
    parser.add_argument("--validate", action="store_true", help="validate sources and exit")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)

    if not AGENTS_ROOT.is_dir():
        print(f"no agent catalog at {AGENTS_ROOT}", file=sys.stderr)
        return 1

    agents, errors = discover()
    if errors:
        for error in errors:
            print(f"INVALID {error}", file=sys.stderr)
        return 1
    if not agents:
        print(f"no agents found in {AGENTS_ROOT}", file=sys.stderr)
        return 1

    if args.list or args.validate:
        for agent in agents:
            meta = agent.meta
            tools = meta.get("tools", "all")
            if isinstance(tools, list):
                tools = ", ".join(tools)
            print(
                f"  {agent.name:28s} {meta.get('model', '?'):8s} "
                f"{meta.get('effort', '?'):6s} tools={tools}"
            )
        if args.validate:
            print(f"{len(agents)} agent(s) valid")
        return 0

    target = (
        Path.home() / ".claude" / "agents"
        if args.target == "global"
        else args.project_dir / ".claude" / "agents"
    )
    print(f"installing {len(agents)} agent(s) -> {target}")
    return install(agents, target, args.dry_run)


if __name__ == "__main__":
    raise SystemExit(main())
