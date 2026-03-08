#!/usr/bin/env python3
"""Render Markdown and JSON upgrade intelligence reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from utils import ensure_dir, markdown_escape, now_iso, write_json


def _dep_sort_key(dep: dict[str, Any]) -> tuple[str, str]:
    return str(dep.get("ecosystem") or ""), str(dep.get("name") or "")


def _summarize_counts(deps: list[dict[str, Any]]) -> dict[str, int]:
    totals = {
        "total": len(deps),
        "high_risk": 0,
        "medium_risk": 0,
        "low_risk": 0,
        "with_breaking": 0,
        "with_deprecations": 0,
    }
    for dep in deps:
        risk = dep.get("risk_level")
        if risk == "high":
            totals["high_risk"] += 1
        elif risk == "medium":
            totals["medium_risk"] += 1
        else:
            totals["low_risk"] += 1
        if dep.get("breaking_changes"):
            totals["with_breaking"] += 1
        if dep.get("deprecations"):
            totals["with_deprecations"] += 1
    return totals


def render_markdown(report: dict[str, Any]) -> str:
    deps = sorted(report.get("dependencies", []), key=_dep_sort_key)
    counts = _summarize_counts(deps)

    lines: list[str] = []
    lines.append(f"# Dependency Upgrade Intelligence Report")
    lines.append("")
    lines.append(f"- Generated at: `{report.get('generated_at')}`")
    lines.append(f"- Repository: `{report.get('repo_root')}`")
    lines.append(f"- Mode: `{report.get('mode')}`")
    lines.append(f"- Runtime policy: `{report.get('compatibility_policy')}`")
    targeted = report.get("targeted_dependencies") or []
    if targeted:
        lines.append(f"- Targeted dependency selectors: `{', '.join(str(x) for x in targeted)}`")
    lines.append("")

    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"Analyzed **{counts['total']}** dependencies. High-risk upgrades: **{counts['high_risk']}**, "
        f"medium-risk: **{counts['medium_risk']}**, low-risk: **{counts['low_risk']}**."
    )
    lines.append(
        f"Breaking-change signals found for **{counts['with_breaking']}** dependencies and deprecation signals for **{counts['with_deprecations']}**."
    )
    lines.append("")

    lines.append("## Runtime Context")
    lines.append("")
    node_runtime = (report.get("repo_context") or {}).get("node_runtime") or {}
    py_runtime = (report.get("repo_context") or {}).get("python_runtime") or {}
    lines.append(f"- Node runtime hint: `{node_runtime.get('detected') or 'not detected'}`")
    lines.append(f"- Node major used for compatibility: `{node_runtime.get('major') or 'n/a'}`")
    lines.append(f"- Python runtime hint: `{py_runtime.get('detected') or 'not detected'}`")
    lines.append("")

    if targeted:
        lines.append("## Targeted Dependency Scope")
        lines.append("")
        lines.append("This report is scoped to explicitly requested dependency selector(s).")
        lines.append("")

    lines.append("## Upgrade Matrix")
    lines.append("")
    lines.append("| Ecosystem | Dependency | Current | Target | Latest | Risk | Reason |")
    lines.append("|---|---|---:|---:|---:|---|---|")
    for dep in deps:
        lines.append(
            "| "
            + " | ".join(
                [
                    markdown_escape(str(dep.get("ecosystem") or "")),
                    markdown_escape(str(dep.get("name") or "")),
                    markdown_escape(str(dep.get("current_version") or "unknown")),
                    markdown_escape(str(dep.get("target_version") or "unknown")),
                    markdown_escape(str(dep.get("latest_available") or "unknown")),
                    markdown_escape(str(dep.get("risk_level") or "low")),
                    markdown_escape(str(dep.get("target_reason") or "")),
                ]
            )
            + " |"
        )
    lines.append("")

    lines.append("## Required Refactors")
    lines.append("")
    for dep in deps:
        actions = dep.get("refactor_actions") or []
        if not actions:
            continue
        lines.append(f"### {dep.get('name')}")
        lines.append(f"- Current -> target: `{dep.get('current_version') or 'unknown'}` -> `{dep.get('target_version') or 'unknown'}`")
        for action in actions[:8]:
            lines.append(f"- {action}")
        lines.append("")

    lines.append("## Breaking Changes and Deprecations")
    lines.append("")
    for dep in deps:
        breaking = dep.get("breaking_changes") or []
        deprecations = dep.get("deprecations") or []
        if not breaking and not deprecations:
            continue
        lines.append(f"### {dep.get('name')}")
        for line in breaking[:8]:
            lines.append(f"- BREAKING: {line}")
        for line in deprecations[:8]:
            lines.append(f"- DEPRECATION: {line}")
        lines.append("")

    lines.append("## New Features and Improvements to Consider")
    lines.append("")
    for dep in deps:
        features = dep.get("feature_adoptions") or []
        if not features:
            continue
        lines.append(f"### {dep.get('name')}")
        for line in features[:6]:
            lines.append(f"- {line}")
    lines.append("")

    lines.append("## Repository Impact Map")
    lines.append("")
    for dep in deps:
        usage = dep.get("repo_usage")
        if not isinstance(usage, dict):
            continue
        lines.append(f"### {dep.get('name')}")
        lines.append(f"- {usage.get('summary')}")
        files = usage.get("files") or []
        if files:
            lines.append("- Affected files:")
            for fp in files[:40]:
                lines.append(f"- `{fp}`")
        hits = usage.get("hits") or []
        if hits:
            lines.append("- Representative matches:")
            for h in hits[:20]:
                p = h.get("path") or ""
                ln = h.get("line")
                txt = h.get("text") or ""
                loc = f"{p}:{ln}" if ln else p
                lines.append(f"- `{loc}` -> `{markdown_escape(str(txt))}`")
        lines.append("")

    lines.append("## Ordered Implementation Checklist")
    lines.append("")
    lines.append("1. Create a branch and pin upgrade order by risk (high -> medium -> low).")
    lines.append("2. Upgrade one dependency (or one tightly-coupled group) at a time.")
    lines.append("3. Apply listed refactors, then run tests/lint/type checks for impacted modules.")
    lines.append("4. Validate runtime compatibility constraints (Node/Python) after each upgrade batch.")
    lines.append("5. Re-run this skill and confirm no unresolved breaking/deprecation items remain.")
    lines.append("")

    lines.append("## Source Links")
    lines.append("")
    for dep in deps:
        links = dep.get("source_links") or []
        if not links:
            continue
        lines.append(f"### {dep.get('name')}")
        for item in links:
            label = item.get("label") or "source"
            url = item.get("url") or ""
            lines.append(f"- {label}: {url}")
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def write_reports(
    out_dir: Path,
    repo_root: str,
    repo_context: dict[str, Any],
    dependencies: list[dict[str, Any]],
    mode: str,
    compatibility_policy: str,
    command_traces: list[dict[str, Any]],
    warnings: list[str],
    targeted_dependencies: list[str] | None = None,
    deep_repo_map: bool = False,
) -> dict[str, str]:
    ensure_dir(out_dir)

    report = {
        "generated_at": now_iso(),
        "repo_root": repo_root,
        "mode": mode,
        "compatibility_policy": compatibility_policy,
        "repo_context": repo_context,
        "summary": _summarize_counts(dependencies),
        "targeted_dependencies": targeted_dependencies or [],
        "deep_repo_map": deep_repo_map,
        "dependencies": dependencies,
        "warnings": warnings,
        "command_traces": command_traces,
    }

    md_path = out_dir / "dependency-upgrade-report.md"
    json_path = out_dir / "dependency-upgrade-report.json"

    md_path.write_text(render_markdown(report), encoding="utf-8")
    write_json(json_path, report)

    return {
        "markdown": str(md_path),
        "json": str(json_path),
    }
