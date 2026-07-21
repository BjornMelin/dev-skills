#!/usr/bin/env python3
"""Retire the `grilling` / `grill-with-docs` / `batch-grill-me` skills and
repoint their dependents onto the scored `grill-me` / `batch-grill-with-docs`
skills.

This is a *local maintenance* utility, not an Agent Skill. It edits the
installed skill tree under ~/.agents/skills (override with --skills-dir).
Re-run it after re-installing the mattpocock skill pack, since that pack
overwrites the dependent skills back to their upstream `/grilling` wording.

It is idempotent: token swaps that are already applied are no-ops, and a
directory that is already gone is skipped. Use --dry-run to preview.

What it does:
  1. Repoint skill references inside the 5 dependent skills:
       /grilling        -> /grill-me            (superset: one-at-a-time + scoring)
       /grill-with-docs -> /batch-grill-with-docs
     Only leading-slash command tokens are touched, so `grilling` ticket-type
     *labels* (no slash) are left alone.
  2. Remove the retired skill directories if present:
       grilling, grill-with-docs, batch-grill-me
"""

from __future__ import annotations

import argparse
import re
import shutil
import sys
from pathlib import Path

# Dependent skills that invoke the retired skills by name.
DEPENDENTS = (
    "improve-codebase-architecture",
    "loop-me",
    "triage",
    "wayfinder",
    "setup-matt-pocock-skills",
)

# Retired skill directories to remove.
RETIRED = (
    "grilling",
    "grill-with-docs",
    "batch-grill-me",
)

# (leading-slash command token) -> replacement. A trailing (?![a-z0-9-])
# guard stops us from matching a longer skill name that merely starts the
# same way (and makes the swaps idempotent).
REPOINTS = (
    (re.compile(r"/grilling(?![a-z0-9-])"), "/grill-me"),
    (re.compile(r"/grill-with-docs(?![a-z0-9-])"), "/batch-grill-with-docs"),
)

TEXT_SUFFIXES = {".md", ".markdown", ".yaml", ".yml", ".txt", ".json"}


def repoint_file(path: Path, dry_run: bool) -> int:
    """Apply the repoint swaps to one file. Returns the number of substitutions."""
    original = path.read_text(encoding="utf-8")
    updated = original
    total = 0
    for pattern, replacement in REPOINTS:
        updated, count = pattern.subn(replacement, updated)
        total += count
    if total and not dry_run:
        path.write_text(updated, encoding="utf-8")
    return total


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--skills-dir",
        default=str(Path.home() / ".agents" / "skills"),
        help="Installed skills directory (default: ~/.agents/skills)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would change without writing.",
    )
    args = parser.parse_args()

    skills_dir = Path(args.skills_dir).expanduser()
    if not skills_dir.is_dir():
        print(f"error: skills dir not found: {skills_dir}", file=sys.stderr)
        return 1

    tag = "[dry-run] " if args.dry_run else ""
    repoint_total = 0
    files_changed = 0

    print(f"{tag}Repointing dependents under {skills_dir}")
    for dep in DEPENDENTS:
        dep_dir = skills_dir / dep
        if not dep_dir.is_dir():
            print(f"  - {dep}: not installed, skipping")
            continue
        dep_changes = 0
        for path in sorted(dep_dir.rglob("*")):
            if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
                continue
            count = repoint_file(path, args.dry_run)
            if count:
                files_changed += 1
                dep_changes += count
                rel = path.relative_to(skills_dir)
                print(f"    {rel}: {count} reference(s) repointed")
        repoint_total += dep_changes
        if dep_changes == 0:
            print(f"  - {dep}: already repointed (no changes)")

    print(f"{tag}Removing retired skill directories")
    removed = 0
    for name in RETIRED:
        target = skills_dir / name
        if target.is_dir():
            print(f"    removing {name}/")
            if not args.dry_run:
                shutil.rmtree(target)
            removed += 1
        else:
            print(f"  - {name}: already gone, skipping")

    print(
        f"{tag}Done: {repoint_total} reference(s) across {files_changed} file(s) "
        f"repointed; {removed} directory(ies) removed."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
