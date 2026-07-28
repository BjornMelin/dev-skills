#!/usr/bin/env python3
"""Create an isolated goal-to-release ledger workbench."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
import uuid
from pathlib import Path

MODES = ("plan-only", "preflight", "implement", "monitor", "closeout", "retrospective")


def slugify(value: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "-", value.lower()).strip("-")
    slug = re.sub(r"-{2,}", "-", slug)
    return slug[:72].strip("-") or "goal"


def run_git(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def repo_root(start: Path) -> Path | None:
    result = run_git(["rev-parse", "--show-toplevel"], start)
    if result.returncode != 0:
        return None
    return Path(result.stdout.strip()).resolve()


def is_git_ignored(path: Path, root: Path) -> bool:
    relative = os.path.relpath(path, root)
    result = run_git(["check-ignore", "-q", "--", relative], root)
    return result.returncode == 0


def repo_slug(root: Path | None, explicit_repo: str) -> str:
    if explicit_repo:
        return slugify(explicit_repo)
    if root:
        return slugify(root.name)
    return "repo"


def build_ledger(title: str, repo: str, mode: str, date: str, goal_id: str) -> str:
    return f"""# Goal Ledger: {title}

Date: {date}
Repo: {repo or "TBD"}
Goal id: {goal_id}
Mode: {mode}
Status: planned

## Goal Charter

Objective:

Non-goals:

User steering and constraints:
- 

Definition of done:
- 

Release/deploy boundary:
- 

## Preflight

Repo:
Branch:
Dirty worktree:
Discovered surfaces:
- 

Required gates:
- 

Docs/deploy evidence needed:
- 

Suggested subagent reviews:
- 

Blockers / warnings:
- 

## Issue Plan

| Issue | Lane | Branch | PR | Scope | Acceptance checks | Docs/deploy impact | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| TBD | TBD | TBD | TBD | TBD | TBD | TBD | Planned |

## Branch and PR Graph

Base:

Graph:
1. TBD

Stack contract:
- Base branch:
- Dependency:
- Retarget plan:
- Validation scope:
- Merge order:

Merge plan:
- 

## Lane Implementation Loop

Lane:
Issue:
Branch:
PR:

Evidence gathered:
- 

Decision:
- 

Files changed:
- 

Validation:
- 

Hosted checks/review:
- Review decision:
- Unresolved threads:
- 

Deploy/monitoring:
- Applicability:
- 

Residual risk:
- 

## Closeout Audit

Issues:
- [ ] All implemented issues link to PRs.
- [ ] Closed issues match shipped behavior.
- [ ] Non-shipped items are dropped or moved to follow-up issues.

PRs:
- [ ] All PRs merged into intended targets.
- [ ] Stacked PRs retargeted or closed.
- [ ] Review threads resolved from live hosted state.

Docs:
- [ ] README/current-state docs updated.
- [ ] ADR/SPEC/requirements updated when contracts changed.
- [ ] Runbooks/deploy docs updated when operations changed.

Release/deploy:
- [ ] Required dev deploy completed or explicitly deferred.
- [ ] Required prod deploy completed or explicitly deferred.
- [ ] Monitoring/diagnostics checked.

Durable summary:
- [ ] summary.md exists for achieved durable goals.
- [ ] summary.json exists for achieved durable goals.

## Retrospective

What shipped:
- 

Major decisions:
- 

User steering that changed the outcome:
- 

Hard cuts / entropy reductions:
- 

Verification and deploy evidence:
- 

Docs and issue hygiene:
- 

What to do differently next time:
- 

Reusable workflow update:
- 
"""


def default_goal_dir(args: argparse.Namespace, goal_id: str, slug: str) -> tuple[Path, str]:
    start = Path(args.repo_root or ".").expanduser().resolve()
    root = repo_root(start) or start
    directory_name = f"{args.date}-{slug}-{goal_id}"

    if args.prefer_repo_local:
        repo_goal_root = root / ".agents" / "goals"
        if is_git_ignored(repo_goal_root, root):
            return repo_goal_root / directory_name, "repo-local"
        if args.no_fallback:
            raise SystemExit(
                f"error: {repo_goal_root} is not git-ignored; refusing to create tracked goal artifacts"
            )

    home_root = Path(args.home_fallback).expanduser()
    return home_root / repo_slug(root, args.repo) / directory_name, "home-fallback"


def resolve_output(args: argparse.Namespace, goal_id: str, slug: str) -> tuple[Path, str]:
    if not args.out:
        return default_goal_dir(args, goal_id, slug)

    out = Path(args.out).expanduser()
    if out.suffix:
        return out, "explicit-file"
    return out / f"{args.date}-{slug}-{goal_id}", "explicit-dir"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Create a goal-to-release ledger workbench.",
    )
    parser.add_argument("--title", required=True, help="Goal title.")
    parser.add_argument("--repo", default="", help="Repository or workspace name.")
    parser.add_argument("--repo-root", default=".", help="Repo path used for git ignore checks.")
    parser.add_argument("--mode", default="implement", choices=MODES, help="Active workflow mode.")
    parser.add_argument("--date", default=dt.date.today().isoformat(), help="Ledger date.")
    parser.add_argument("--out", help="Output file or directory. Defaults to an untracked goal workbench.")
    parser.add_argument(
        "--prefer-repo-local",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Prefer an ignored .agents/goals directory in the repo.",
    )
    parser.add_argument(
        "--home-fallback",
        default="~/.codex/goals",
        help="Fallback root when repo-local storage would be tracked.",
    )
    parser.add_argument(
        "--no-fallback",
        action="store_true",
        help="Fail instead of using the home fallback when repo-local storage is not ignored.",
    )
    parser.add_argument("--print-json", action="store_true", help="Print machine-readable result.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    goal_id = uuid.uuid4().hex[:8]
    slug = slugify(args.title)
    target, storage = resolve_output(args, goal_id, slug)
    ledger = build_ledger(args.title, args.repo, args.mode, args.date, goal_id)

    if target.suffix:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(ledger, encoding="utf-8")
        ledger_path = target
        goal_dir = target.parent
    else:
        target.mkdir(parents=True, exist_ok=True)
        for child in ("evidence", "preflight", "archive"):
            (target / child).mkdir(exist_ok=True)
        ledger_path = target / "ledger.md"
        ledger_path.write_text(ledger, encoding="utf-8")
        goal_dir = target

    result = {
        "goal_dir": str(goal_dir),
        "ledger": str(ledger_path),
        "storage": storage,
        "goal_id": goal_id,
    }
    if args.print_json:
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(ledger_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
