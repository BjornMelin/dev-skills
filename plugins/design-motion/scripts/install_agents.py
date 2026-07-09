#!/usr/bin/env python3
(
    "Install the design-motion Claude Code subagents into a .claude/agents "
    "directory.\n\n"
    "Use this to run the six specialist motion subagents WITHOUT enabling the "
    "full\n"
    "design-motion plugin (e.g. you want the agents but install the skills "
    "separately,\n"
    "or not at all). The plugin's `agents/` directory is the single source of "
    "truth;\n"
    "this script only copies those agent files into your Claude Code agents "
    "directory.\n\n"
    "  python3 plugins/design-motion/scripts/install_agents.py --target "
    "global   # ~/.claude/agents\n"
    "  python3 plugins/design-motion/scripts/install_agents.py --target "
    "project  # ./.claude/agents\n"
    "  python3 plugins/design-motion/scripts/install_agents.py "
    "--dry-run         # preview only\n"
)

from __future__ import annotations

import argparse
import shutil
from pathlib import Path


def target_dir(target: str, project_dir: Path) -> Path:
    """Resolve the destination agent directory for a named target."""
    if target == "global":
        return Path.home() / ".claude" / "agents"
    if target == "project":
        return project_dir / ".claude" / "agents"
    raise ValueError(f"unknown target: {target}")


def main() -> int:
    """Install or preview the bundled design-motion subagents."""
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--target",
        choices=["global", "project"],
        default="project",
    )
    parser.add_argument("--project-dir", type=Path, default=Path.cwd())
    parser.add_argument(
        "--dest",
        type=Path,
        help="explicit destination dir (overrides --target)",
    )
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--overwrite", action="store_true")
    args = parser.parse_args()

    agents_dir = Path(__file__).resolve().parents[1] / "agents"
    dest = args.dest or target_dir(args.target, args.project_dir.resolve())

    installed = []
    for src in sorted(agents_dir.glob("*.md")):
        dst = dest / src.name
        if dst.exists() and not args.overwrite:
            print(f"skip {dst} (exists; pass --overwrite)")
            continue
        installed.append((src, dst))
        if not args.dry_run:
            dest.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)
        action = "would install" if args.dry_run else "installed"
        print(f"{action} {src.name} -> {dst}")

    if not installed:
        print("no agents installed")
    else:
        action = "would install" if args.dry_run else "installed"
        print(f"\n{action} {len(installed)} agent(s) into {dest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
