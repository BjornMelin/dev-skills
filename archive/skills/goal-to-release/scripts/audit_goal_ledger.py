#!/usr/bin/env python3
"""Audit a goal-to-release ledger for implementation and closeout readiness."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUIRED_HEADINGS = [
    "Goal Charter",
    "Preflight",
    "Issue Plan",
    "Branch and PR Graph",
    "Lane Implementation Loop",
    "Closeout Audit",
    "Retrospective",
]

PLACEHOLDER_RE = re.compile(r"\b(?:TBD|UNVERIFIED)\b")
CHECKBOX_RE = re.compile(r"(?m)^\s*-\s+\[\s\]\s+")
HEADING_RE = re.compile(r"(?m)^##\s+(.+?)\s*$")
ACHIEVED_RE = re.compile(r"(?im)^Status:\s*(achieved|complete|completed|done|shipped)\s*$")


def resolve_ledger(path: Path) -> Path:
    expanded = path.expanduser()
    if expanded.is_dir():
        return expanded / "ledger.md"
    return expanded


def split_sections(text: str) -> dict[str, str]:
    matches = list(HEADING_RE.finditer(text))
    sections: dict[str, str] = {}
    for index, match in enumerate(matches):
        start = match.end()
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        sections[match.group(1).strip()] = text[start:end].strip()
    return sections


def status_is_achieved(text: str) -> bool:
    return bool(ACHIEVED_RE.search(text))


def missing_headings(text: str) -> list[str]:
    return [
        heading
        for heading in REQUIRED_HEADINGS
        if not re.search(rf"(?m)^##\s+{re.escape(heading)}\s*$", text)
    ]


def count_unchecked_checkboxes(text: str) -> int:
    return len(CHECKBOX_RE.findall(text))


def count_placeholders(text: str) -> int:
    return len(PLACEHOLDER_RE.findall(text))


def has_meaningful_content(value: str) -> bool:
    stripped = "\n".join(
        line.strip(" -\t")
        for line in value.splitlines()
        if line.strip() and not line.strip().startswith("| ---")
    ).strip()
    return bool(stripped) and not PLACEHOLDER_RE.search(stripped)


def subfield_content(section: str, label: str) -> str:
    pattern = re.compile(
        rf"(?ms)^{re.escape(label)}:\s*(.*?)(?=^[A-Z][A-Za-z0-9 /.-]+:\s*$|\Z)"
    )
    match = pattern.search(section)
    return match.group(1).strip() if match else ""


def issue_table_warnings(text: str) -> list[str]:
    warnings: list[str] = []
    rows = [
        line.strip()
        for line in text.splitlines()
        if line.strip().startswith("|") and line.count("|") >= 8
    ]
    data_rows = [
        row
        for row in rows
        if not row.startswith("| Issue ")
        and not re.fullmatch(r"\|\s*-+\s*(\|\s*-+\s*)+\|", row)
    ]
    for index, row in enumerate(data_rows, start=1):
        cells = [cell.strip() for cell in row.strip("|").split("|")]
        if len(cells) < 8:
            continue
        issue, _lane, branch, pr, scope, acceptance, docs, status = cells[:8]
        if pr not in {"", "TBD", "UNVERIFIED"} and issue in {"", "TBD", "UNVERIFIED"}:
            warnings.append(f"issue row {index}: PR is set but issue is missing")
        if issue not in {"", "TBD", "UNVERIFIED"} and pr in {"", "TBD", "UNVERIFIED"}:
            warnings.append(f"issue row {index}: issue is set but PR is missing")
        if branch in {"", "TBD", "UNVERIFIED"} and pr not in {"", "TBD", "UNVERIFIED"}:
            warnings.append(f"issue row {index}: PR is set but branch is missing")
        if status.lower() in {"done", "merged", "closed", "shipped"}:
            for label, value in (
                ("scope", scope),
                ("acceptance checks", acceptance),
                ("docs/deploy impact", docs),
            ):
                if value in {"", "TBD", "UNVERIFIED"}:
                    warnings.append(f"issue row {index}: completed row is missing {label}")
    return warnings


def summary_artifact_warnings(ledger_path: Path, text: str) -> tuple[list[str], list[str]]:
    blockers: list[str] = []
    warnings: list[str] = []
    if not status_is_achieved(text):
        return blockers, warnings

    goal_dir = ledger_path.parent
    for name in ("summary.md", "summary.json"):
        path = goal_dir / name
        if not path.exists():
            blockers.append(f"achieved goal is missing {name}")
        elif path.stat().st_size == 0:
            blockers.append(f"achieved goal has empty {name}")

    summary_json = goal_dir / "summary.json"
    if summary_json.exists() and summary_json.stat().st_size:
        try:
            payload = json.loads(summary_json.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            blockers.append(f"summary.json is invalid JSON: {exc}")
        else:
            for field in ("title", "status", "outcomes", "validation", "follow_ups"):
                if field not in payload:
                    warnings.append(f"summary.json is missing field: {field}")
    return blockers, warnings


def closeout_warnings(sections: dict[str, str]) -> list[str]:
    warnings: list[str] = []
    implementation = sections.get("Lane Implementation Loop", "")
    for label in ("Validation", "Hosted checks/review", "Deploy/monitoring", "Residual risk"):
        value = subfield_content(implementation, label)
        if not has_meaningful_content(value):
            warnings.append(f"lane implementation loop is missing meaningful {label.lower()} evidence")

    retrospective = sections.get("Retrospective", "")
    for label in ("What shipped", "Verification and deploy evidence", "What to do differently next time"):
        value = subfield_content(retrospective, label)
        if not has_meaningful_content(value):
            warnings.append(f"retrospective is missing meaningful {label.lower()}")
    return warnings


def audit(path: Path, *, strict: bool = False) -> dict[str, Any]:
    ledger_path = resolve_ledger(path)
    if not ledger_path.is_file():
        return {
            "ledger": str(ledger_path),
            "blockers": [f"ledger not found: {ledger_path}"],
            "warnings": [],
            "strict": strict,
        }

    text = ledger_path.read_text(encoding="utf-8")
    sections = split_sections(text)
    blockers: list[str] = []
    warnings: list[str] = []

    blockers.extend(f"missing heading: ## {heading}" for heading in missing_headings(text))

    placeholder_count = count_placeholders(text)
    if placeholder_count:
        blockers.append(f"{placeholder_count} unresolved TBD/UNVERIFIED marker(s)")

    unchecked = count_unchecked_checkboxes(text)
    if unchecked:
        if strict:
            blockers.append(f"{unchecked} unchecked closeout checkbox(es)")
        else:
            warnings.append(f"{unchecked} unchecked closeout checkbox(es)")

    warnings.extend(issue_table_warnings(text))
    warnings.extend(closeout_warnings(sections))

    summary_blockers, summary_warnings = summary_artifact_warnings(ledger_path, text)
    blockers.extend(summary_blockers)
    warnings.extend(summary_warnings)

    return {
        "ledger": str(ledger_path),
        "blockers": blockers,
        "warnings": warnings,
        "strict": strict,
        "status": "blocked" if blockers else "warning" if warnings else "pass",
    }


def render_markdown(report: dict[str, Any]) -> str:
    lines = [f"# Goal Ledger Audit: {report['ledger']}", ""]
    blockers = report["blockers"]
    warnings = report["warnings"]

    if not blockers and not warnings:
        lines.append("PASS: ledger has no obvious implementation or closeout gaps.")
    else:
        if blockers:
            lines.append("## Blockers")
            lines.extend(f"- {item}" for item in blockers)
            lines.append("")
        if warnings:
            lines.append("## Warnings")
            lines.extend(f"- {item}" for item in warnings)
    return "\n".join(lines).rstrip() + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit a goal-to-release ledger for implementation and closeout readiness.",
    )
    parser.add_argument("ledger", help="Path to a ledger.md file or goal workbench directory.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat unchecked closeout checkboxes as blockers.",
    )
    parser.add_argument(
        "--format",
        choices=("markdown", "json"),
        default="markdown",
        help="Output format.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = audit(Path(args.ledger), strict=args.strict)
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_markdown(report), end="")
    return 1 if report["blockers"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
