#!/usr/bin/env python3
"""Install the Claude Code subagent catalog with backups.

Mirrors ``subagents/codex/scripts/sync_agents.py`` for the Claude side: Codex roles are TOML
under ``~/.codex/agents``; Claude agents are Markdown with YAML frontmatter under
``~/.claude/agents`` (global) or ``<project>/.claude/agents`` (project).

Home is always resolved at runtime via ``Path.home()``. This repository is public and
``tools/policy/check_public_leaks.py`` rejects a literal home path in any tracked file.
"""

from __future__ import annotations

import argparse
import re
import shutil
import sys
from dataclasses import dataclass
from datetime import datetime
from datetime import timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AGENTS_ROOT = ROOT / "agents" / "global"

# Claude Code agent frontmatter keys. `model` and `effort` are pinned explicitly on every
# agent: MODELS.md forbids inheriting a worker model or effort.
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
VALID_EFFORTS = {"low", "medium", "high", "xhigh", "max"}
FRONTMATTER_RE = re.compile(r"\A---\n(.*?)\n---\n", re.DOTALL)


def stamp() -> str:
    return datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")


@dataclass(frozen=True)
class Agent:
    path: Path
    name: str
    meta: dict[str, str]


def parse(path: Path) -> Agent:
    text = path.read_text(encoding="utf-8")
    match = FRONTMATTER_RE.match(text)
    if not match:
        raise ValueError(f"{path.name}: no YAML frontmatter")
    meta: dict[str, str] = {}
    for line in match.group(1).splitlines():
        if not line.strip() or line.startswith("#") or line.startswith(" "):
            continue
        key, _, value = line.partition(":")
        meta[key.strip()] = value.strip().strip('"')
    return Agent(path=path, name=meta.get("name", ""), meta=meta)


def validate(agent: Agent) -> list[str]:
    errors = []
    for key in REQUIRED_KEYS:
        if not agent.meta.get(key):
            errors.append(f"missing '{key}'")
    unknown = set(agent.meta) - ALLOWED_KEYS
    if unknown:
        errors.append(f"unexpected key(s): {', '.join(sorted(unknown))}")
    if agent.name and agent.name != agent.path.stem:
        errors.append(f"name '{agent.name}' must match filename '{agent.path.stem}'")
    if agent.meta.get("model") == "inherit":
        errors.append("model must be pinned explicitly, never 'inherit'")
    effort = agent.meta.get("effort")
    if effort and effort not in VALID_EFFORTS:
        errors.append(f"invalid effort '{effort}'; expected one of {sorted(VALID_EFFORTS)}")
    return errors


def discover() -> list[Agent]:
    return [parse(p) for p in sorted(AGENTS_ROOT.glob("*.md"))]


def install(agents: list[Agent], target: Path, dry_run: bool) -> int:
    backup_dir = target.parent / "agent-backups" / f"claude-{stamp()}"
    target.mkdir(parents=True, exist_ok=True)
    for agent in agents:
        dst = target / agent.path.name
        if dst.exists() and dst.read_bytes() == agent.path.read_bytes():
            print(f"  unchanged  {agent.name}")
            continue
        if dry_run:
            print(f"  would sync {agent.name} -> {dst}")
            continue
        if dst.exists():
            backup_dir.mkdir(parents=True, exist_ok=True)
            shutil.copy2(dst, backup_dir / dst.name)
        shutil.copy2(agent.path, dst)
        print(f"  synced     {agent.name} -> {dst}")
    if backup_dir.exists():
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

    agents = discover()
    if not agents:
        print(f"no agents found in {AGENTS_ROOT}", file=sys.stderr)
        return 1

    failed = False
    for agent in agents:
        errors = validate(agent)
        if errors:
            failed = True
            for error in errors:
                print(f"INVALID {agent.path.name}: {error}", file=sys.stderr)
    if failed:
        return 1

    if args.list or args.validate:
        for agent in agents:
            meta = agent.meta
            print(
                f"  {agent.name:28s} {meta.get('model', '?'):8s} "
                f"{meta.get('effort', '?'):6s} tools={meta.get('tools', 'all')}"
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
