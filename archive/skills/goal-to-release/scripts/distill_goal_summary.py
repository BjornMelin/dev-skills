#!/usr/bin/env python3
"""Distill a completed goal workbench into compact durable summary artifacts."""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

HEADING_RE = re.compile(r"(?m)^##\s+(.+?)\s*$")
META_RE = re.compile(r"(?m)^([A-Za-z][A-Za-z0-9 _-]+):\s*(.*?)\s*$")
SECRET_PATTERNS = [
    re.compile(r"sk-[A-Za-z0-9_-]{20,}"),
    re.compile(r"gh[pousr]_[A-Za-z0-9_]{20,}"),
    re.compile(r"xox[baprs]-[A-Za-z0-9-]{20,}"),
    re.compile(r"(?i)(api[_-]?key|token|secret|password)=\S+"),
]


def resolve_goal_dir(path: Path) -> Path:
    expanded = path.expanduser()
    if expanded.is_file():
        return expanded.parent
    return expanded


def ledger_path(goal_dir: Path) -> Path:
    direct = goal_dir / "ledger.md"
    if direct.exists():
        return direct
    if goal_dir.is_file():
        return goal_dir
    raise FileNotFoundError(f"ledger.md not found in {goal_dir}")


def redact(value: str) -> str:
    redacted = value
    for pattern in SECRET_PATTERNS:
        redacted = pattern.sub("[REDACTED]", redacted)
    return redacted


def split_sections(text: str) -> dict[str, str]:
    matches = list(HEADING_RE.finditer(text))
    sections: dict[str, str] = {}
    for index, match in enumerate(matches):
        start = match.end()
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        sections[match.group(1).strip()] = redact(text[start:end].strip())
    return sections


def metadata(text: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for key, value in META_RE.findall(text):
        normalized = key.strip().lower().replace(" ", "_")
        if normalized in {"date", "repo", "goal_id", "mode", "status"}:
            values[normalized] = redact(value.strip())
    return values


def title_from_ledger(text: str, fallback: str) -> str:
    first_line = text.splitlines()[0].strip() if text.splitlines() else ""
    if first_line.startswith("# Goal Ledger:"):
        return redact(first_line.split(":", 1)[1].strip())
    if first_line.startswith("#"):
        return redact(first_line.lstrip("#").strip())
    return fallback


def bullets(section: str, limit: int = 20) -> list[str]:
    values: list[str] = []
    for line in section.splitlines():
        stripped = line.strip()
        if stripped.startswith("- "):
            item = stripped[2:].strip()
            if item and item not in {"TBD", "UNVERIFIED"}:
                values.append(item)
    return values[:limit]


def subfield(section: str, label: str) -> str:
    pattern = re.compile(
        rf"(?ms)^{re.escape(label)}:\s*(.*?)(?=^[A-Z][A-Za-z0-9 /.-]+:\s*$|\Z)"
    )
    match = pattern.search(section)
    if not match:
        return ""
    return redact(match.group(1).strip())


def list_artifacts(goal_dir: Path) -> list[str]:
    artifacts: list[str] = []
    for folder_name in ("preflight", "evidence", "archive"):
        folder = goal_dir / folder_name
        if not folder.exists():
            continue
        for path in sorted(folder.rglob("*")):
            if path.is_file():
                artifacts.append(str(path.relative_to(goal_dir)))
    return artifacts[:250]


def table_rows(section: str) -> list[dict[str, str]]:
    rows: list[dict[str, str]] = []
    for line in section.splitlines():
        stripped = line.strip()
        if not stripped.startswith("|") or stripped.startswith("| ---") or stripped.startswith("| Issue "):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if len(cells) < 8:
            continue
        rows.append(
            {
                "issue": cells[0],
                "lane": cells[1],
                "branch": cells[2],
                "pr": cells[3],
                "scope": cells[4],
                "acceptance_checks": cells[5],
                "docs_deploy_impact": cells[6],
                "status": cells[7],
            }
        )
    return rows


def build_summary(goal_dir: Path, *, title_override: str | None = None) -> dict[str, Any]:
    ledger = ledger_path(goal_dir)
    text = redact(ledger.read_text(encoding="utf-8"))
    sections = split_sections(text)
    meta = metadata(text)
    title = title_override or title_from_ledger(text, goal_dir.name)
    implementation = sections.get("Lane Implementation Loop", "")
    retrospective = sections.get("Retrospective", "")

    return {
        "title": title,
        "repo": meta.get("repo", ""),
        "goal_id": meta.get("goal_id", ""),
        "date": meta.get("date", ""),
        "mode": meta.get("mode", ""),
        "status": meta.get("status", ""),
        "distilled_at": datetime.now(timezone.utc).isoformat(),
        "goal_dir": str(goal_dir),
        "ledger": str(ledger),
        "outcomes": bullets(subfield(retrospective, "What shipped")) or bullets(retrospective),
        "major_decisions": bullets(subfield(retrospective, "Major decisions")),
        "hard_cuts": bullets(subfield(retrospective, "Hard cuts / entropy reductions")),
        "validation": bullets(subfield(implementation, "Validation"))
        or bullets(subfield(retrospective, "Verification and deploy evidence")),
        "deploy_monitoring": bullets(subfield(implementation, "Deploy/monitoring")),
        "residual_risks": bullets(subfield(implementation, "Residual risk")),
        "follow_ups": bullets(subfield(retrospective, "What to do differently next time")),
        "issues": table_rows(sections.get("Issue Plan", "")),
        "artifacts": list_artifacts(goal_dir),
    }


def render_markdown(summary: dict[str, Any]) -> str:
    lines = [f"# Goal Summary: {summary['title']}", ""]
    for label, field in (
        ("Repo", "repo"),
        ("Goal id", "goal_id"),
        ("Date", "date"),
        ("Status", "status"),
        ("Ledger", "ledger"),
    ):
        value = summary.get(field)
        if value:
            lines.append(f"- {label}: `{value}`")
    lines.append("")

    section_map = [
        ("Outcomes", "outcomes"),
        ("Major Decisions", "major_decisions"),
        ("Hard Cuts", "hard_cuts"),
        ("Validation", "validation"),
        ("Deploy / Monitoring", "deploy_monitoring"),
        ("Residual Risks", "residual_risks"),
        ("Follow-ups", "follow_ups"),
    ]
    for heading, field in section_map:
        values = summary.get(field) or []
        if not values:
            continue
        lines.append(f"## {heading}")
        lines.extend(f"- {item}" for item in values)
        lines.append("")

    if summary.get("issues"):
        lines.append("## Issues and PRs")
        lines.append("| Issue | Lane | Branch | PR | Status |")
        lines.append("| --- | --- | --- | --- | --- |")
        for row in summary["issues"]:
            lines.append(
                f"| {row['issue']} | {row['lane']} | {row['branch']} | {row['pr']} | {row['status']} |"
            )
        lines.append("")

    if summary.get("artifacts"):
        lines.append("## Artifacts")
        lines.extend(f"- `{item}`" for item in summary["artifacts"])
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Distill a goal workbench into summary.md and summary.json.")
    parser.add_argument("goal_dir", help="Path to a goal workbench directory or ledger.md.")
    parser.add_argument("--title", help="Override summary title.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Fail if the ledger status is not achieved/complete/completed/done/shipped.",
    )
    parser.add_argument("--print-json", action="store_true", help="Print the summary JSON payload.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        goal_dir = resolve_goal_dir(Path(args.goal_dir))
        summary = build_summary(goal_dir, title_override=args.title)
    except (OSError, FileNotFoundError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    if args.strict and summary.get("status", "").lower() not in {
        "achieved",
        "complete",
        "completed",
        "done",
        "shipped",
    }:
        print("error: strict mode requires achieved/complete/completed/done/shipped status", file=sys.stderr)
        return 1

    (goal_dir / "summary.md").write_text(render_markdown(summary), encoding="utf-8")
    (goal_dir / "summary.json").write_text(json.dumps(summary, indent=2, sort_keys=True), encoding="utf-8")

    if args.print_json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(goal_dir / "summary.md")
        print(goal_dir / "summary.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
