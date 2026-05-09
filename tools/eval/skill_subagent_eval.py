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
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


SCHEMA = "dev-skills.skill-subagent-eval.v1"
TAIL_CHARS = 4000
CHECK_TIMEOUT_SECONDS = 120


@dataclass(frozen=True)
class EvalCheck:
    """Static definition for one offline eval check.

    Attributes:
        id: Stable machine-readable check identifier.
        name: Human-readable check label.
        command: Repo-relative command argv to execute.
    """

    id: str
    name: str
    command: tuple[str, ...]

    def to_list_item(self) -> dict[str, Any]:
        """Render this check for the list-mode JSON contract.

        Returns:
            Dictionary containing the check id, label, and command argv.
        """
        return {
            "id": self.id,
            "name": self.name,
            "command": list(self.command),
        }


def repo_root() -> Path:
    """Resolve the repository root from this script location.

    Returns:
        Absolute repository root path.
    """
    return Path(__file__).resolve().parents[2]


def portable_repo_root() -> str:
    """Return the portable repository-root marker used in reports.

    Returns:
        The literal `$REPO` marker.
    """
    return "$REPO"


def default_checks() -> list[EvalCheck]:
    """Build the default offline eval check set.

    Returns:
        Ordered list of deterministic skill and subagent checks.
    """
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
    """Parse eval-lab command-line arguments.

    Args:
        argv: Optional argument vector. Defaults to process arguments.

    Returns:
        Parsed argparse namespace.

    Raises:
        SystemExit: If argparse rejects the provided arguments.
    """
    parser = argparse.ArgumentParser(
        description=(
            "Run offline skill/subagent eval checks with JSON evidence output."
        )
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON report")
    parser.add_argument(
        "--list",
        action="store_true",
        help="List checks without running them",
    )
    parser.add_argument(
        "--check",
        action="append",
        default=[],
        help="Run one check id; can be repeated. Defaults to all checks.",
    )
    return parser.parse_args(argv)


def selected_checks(checks: list[EvalCheck], ids: list[str]) -> list[EvalCheck]:
    """Filter checks to requested ids while preserving request order.

    Args:
        checks: Available checks.
        ids: Requested check ids. Empty means all checks.

    Returns:
        Selected check definitions.

    Raises:
        SystemExit: If any requested id is unknown.
    """
    if not ids:
        return checks
    by_id = {check.id: check for check in checks}
    missing = sorted(set(ids) - set(by_id))
    if missing:
        available = ", ".join(sorted(by_id))
        missing_ids = ", ".join(missing)
        message = f"unknown check id(s): {missing_ids}. Available: {available}"
        raise SystemExit(message)
    return [by_id[check_id] for check_id in ids]


def run_check(check: EvalCheck, root: Path) -> dict[str, Any]:
    """Run one eval check and return a bounded evidence record.

    Args:
        check: Eval check definition to execute.
        root: Repository root used as the subprocess working directory.

    Returns:
        JSON-serializable check result.
    """
    started = time.monotonic()
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    with tempfile.TemporaryDirectory(
        prefix="dev-skills-eval-pycache-",
    ) as pycache_dir:
        env["PYTHONPYCACHEPREFIX"] = pycache_dir
        try:
            # Check commands are static repo-owned definitions, not user input.
            completed = subprocess.run(  # noqa: S603
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
        except OSError as error:
            duration_ms = round((time.monotonic() - started) * 1000)
            return {
                "id": check.id,
                "name": check.name,
                "command": list(check.command),
                "status": "failed",
                "exit_code": None,
                "duration_ms": duration_ms,
                "timeout_seconds": CHECK_TIMEOUT_SECONDS,
                "stdout_tail": "",
                "stderr_tail": tail(str(error), root),
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
    """Normalize subprocess exception output to text.

    Args:
        value: Captured subprocess output as text, bytes, or None.

    Returns:
        Decoded text, or an empty string when no output exists.
    """
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode(errors="replace")
    return value


def tail(text: str, root: Path) -> str:
    """Sanitize and bound command output for report embedding.

    Args:
        text: Raw command output.
        root: Absolute repository root to redact.

    Returns:
        Bounded text with repository paths replaced by `$REPO`.
    """
    text = text.replace(str(root), "$REPO").strip()
    if len(text) <= TAIL_CHARS:
        return text
    return "[truncated]\n" + text[-TAIL_CHARS:]


def report(checks: list[EvalCheck], root: Path) -> dict[str, Any]:
    """Run checks and assemble the eval-lab report.

    Args:
        checks: Checks to execute.
        root: Repository root used as subprocess working directory.

    Returns:
        JSON-serializable report payload.
    """
    results = [run_check(check, root) for check in checks]
    return {
        "schema": SCHEMA,
        "generated_at": (
            datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
        ),
        "repo_root": portable_repo_root(),
        "ok": all(result["status"] == "passed" for result in results),
        "checks": results,
    }


def render_human(result: dict[str, Any]) -> str:
    """Render a compact human-readable report summary.

    Args:
        result: Eval-lab report payload.

    Returns:
        Multi-line summary for terminal output.
    """
    lines = [f"{result['schema']} ok={str(result['ok']).lower()}"]
    lines.extend(
        (
            f"- {check['status']}: {check['id']} "
            f"(exit={check['exit_code']}, {check['duration_ms']}ms)"
        )
        for check in result["checks"]
    )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    """Run the eval-lab CLI.

    Args:
        argv: Optional argument vector. Defaults to process arguments.

    Returns:
        Process exit code.
    """
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
