#!/usr/bin/env python3
"""Offline eval lab for skill metadata and subagent contracts."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import tempfile
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


SCHEMA = "dev-skills.skill-subagent-eval.v1"
TAIL_CHARS = 4000
CHECK_TIMEOUT_SECONDS = 120


@dataclass(frozen=True)
class EvalCheck:
    id: str
    name: str
    command: tuple[str, ...]

    def to_list_item(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "name": self.name,
            "command": list(self.command),
        }


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def portable_repo_root() -> str:
    return "$REPO"


def default_checks() -> list[EvalCheck]:
    return [
        EvalCheck(
            id="skill-metadata-deep-researcher",
            name="deep-researcher skill metadata validates",
            command=(
                "python3",
                "tools/skill/quick_validate.py",
                "skills/deep-researcher",
            ),
        ),
        EvalCheck(
            id="skill-metadata-subagent-creator",
            name="subagent-creator skill metadata validates",
            command=(
                "python3",
                "tools/skill/quick_validate.py",
                "skills/subagent-creator",
            ),
        ),
        EvalCheck(
            id="skill-metadata-subspawn",
            name="subspawn skill metadata validates",
            command=(
                "python3",
                "tools/skill/quick_validate.py",
                "skills/subspawn",
            ),
        ),
        EvalCheck(
            id="subagent-template-contracts",
            name="Subagent templates validate through subagent-creator",
            command=(
                "python3",
                "skills/subagent-creator/scripts/subagent_creator.py",
                "validate",
                "--json",
                "skills/deep-researcher/templates/agents",
                "skills/subagent-creator/templates/agents",
                "skills/subspawn/templates/agents",
                "subagents/hardened-codex/agents",
            ),
        ),
        EvalCheck(
            id="subspawn-role-contracts",
            name="Subspawn role contracts validate",
            command=(
                "python3",
                "skills/subspawn/scripts/subspawn_plan.py",
                "validate-roles",
                "--json",
            ),
        ),
        EvalCheck(
            id="subspawn-research-plan",
            name="Subspawn research preset plans deterministically",
            command=(
                "python3",
                "skills/subspawn/scripts/subspawn_plan.py",
                "plan",
                "--preset",
                "research",
                "--task",
                "validation smoke",
                "--scope",
                "docs and template metadata",
                "--json",
            ),
        ),
        EvalCheck(
            id="python-helper-compile",
            name="Skill and hardened-subagent helper scripts compile",
            command=(
                "python3",
                "-m",
                "compileall",
                "-q",
                "skills/deep-researcher/scripts",
                "skills/subagent-creator/scripts",
                "skills/subspawn/scripts",
                "subagents/hardened-codex/scripts",
            ),
        ),
    ]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run offline skill/subagent eval checks with JSON evidence output."
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON report")
    parser.add_argument("--list", action="store_true", help="List checks without running them")
    parser.add_argument(
        "--check",
        action="append",
        default=[],
        help="Run one check id; can be repeated. Defaults to all checks.",
    )
    return parser.parse_args(argv)


def selected_checks(checks: list[EvalCheck], ids: list[str]) -> list[EvalCheck]:
    if not ids:
        return checks
    by_id = {check.id: check for check in checks}
    missing = sorted(set(ids) - set(by_id))
    if missing:
        available = ", ".join(sorted(by_id))
        raise SystemExit(f"unknown check id(s): {', '.join(missing)}. Available: {available}")
    return [by_id[check_id] for check_id in ids]


def run_check(check: EvalCheck, root: Path) -> dict[str, Any]:
    started = time.monotonic()
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    with tempfile.TemporaryDirectory(prefix="dev-skills-eval-pycache-") as pycache_dir:
        env["PYTHONPYCACHEPREFIX"] = pycache_dir
        try:
            completed = subprocess.run(
                check.command,
                cwd=root,
                env=env,
                text=True,
                capture_output=True,
                check=False,
                timeout=CHECK_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            duration_ms = round((time.monotonic() - started) * 1000)
            return {
                "id": check.id,
                "name": check.name,
                "command": list(check.command),
                "status": "timed_out",
                "exit_code": None,
                "duration_ms": duration_ms,
                "timeout_seconds": CHECK_TIMEOUT_SECONDS,
                "stdout_tail": tail(output_text(error.stdout), root),
                "stderr_tail": tail(output_text(error.stderr), root),
            }
    duration_ms = round((time.monotonic() - started) * 1000)
    return {
        "id": check.id,
        "name": check.name,
        "command": list(check.command),
        "status": "passed" if completed.returncode == 0 else "failed",
        "exit_code": completed.returncode,
        "duration_ms": duration_ms,
        "timeout_seconds": CHECK_TIMEOUT_SECONDS,
        "stdout_tail": tail(completed.stdout, root),
        "stderr_tail": tail(completed.stderr, root),
    }


def output_text(value: str | bytes | None) -> str:
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode(errors="replace")
    return value


def tail(text: str, root: Path) -> str:
    text = text.replace(str(root), "$REPO").strip()
    if len(text) <= TAIL_CHARS:
        return text
    return "[truncated]\n" + text[-TAIL_CHARS:]


def report(checks: list[EvalCheck], root: Path) -> dict[str, Any]:
    results = [run_check(check, root) for check in checks]
    return {
        "schema": SCHEMA,
        "generated_at": datetime.now(UTC).isoformat().replace("+00:00", "Z"),
        "repo_root": portable_repo_root(),
        "ok": all(result["status"] == "passed" for result in results),
        "checks": results,
    }


def render_human(result: dict[str, Any]) -> str:
    lines = [f"{result['schema']} ok={str(result['ok']).lower()}"]
    for check in result["checks"]:
        lines.append(
            f"- {check['status']}: {check['id']} "
            f"(exit={check['exit_code']}, {check['duration_ms']}ms)"
        )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    root = repo_root()
    checks = selected_checks(default_checks(), args.check)

    if args.list:
        payload = {
            "schema": SCHEMA,
            "repo_root": portable_repo_root(),
            "checks": [check.to_list_item() for check in checks],
        }
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            for check in payload["checks"]:
                print(f"{check['id']}: {' '.join(check['command'])}")
        return 0

    payload = report(checks, root)
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(render_human(payload))
    return 0 if payload["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
