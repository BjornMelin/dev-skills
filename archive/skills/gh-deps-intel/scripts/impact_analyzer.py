#!/usr/bin/env python3
"""Release note impact extraction and upgrade action suggestions."""

from __future__ import annotations

import re
from typing import Any

from utils import compact_str

BREAKING_PATTERNS = [
    r"\bbreaking\b",
    r"\bremoved\b",
    r"\bdrop(?:ped)? support\b",
    r"\bincompatible\b",
    r"\bmigration\b",
]

DEPRECATION_PATTERNS = [
    r"\bdeprecat(?:e|ed|ion)\b",
    r"\bend[- ]of[- ]life\b",
    r"\bwill be removed\b",
]

FEATURE_PATTERNS = [
    r"\badded\b",
    r"\bnew\b",
    r"\bfeature\b",
    r"\bimprov(?:e|ed|ement)\b",
    r"\bperformance\b",
]


def _collect_matching_lines(text: str, patterns: list[str], limit: int = 12) -> list[str]:
    lines: list[str] = []
    for raw in text.splitlines():
        line = raw.strip()
        if not line:
            continue
        if len(line) < 8:
            continue
        if not re.search(r"[A-Za-z]", line):
            continue
        for pat in patterns:
            if re.search(pat, line, flags=re.IGNORECASE):
                lines.append(compact_str(line, 280))
                break
        if len(lines) >= limit:
            break
    # preserve order and uniqueness
    out: list[str] = []
    seen: set[str] = set()
    for line in lines:
        if line in seen:
            continue
        seen.add(line)
        out.append(line)
    return out


def analyze_dependency_changes(dep_row: dict[str, Any]) -> dict[str, Any]:
    release_notes = dep_row.get("release_notes") or []
    changelog = dep_row.get("changelog_text") or ""

    corpus_parts: list[str] = []
    for rel in release_notes:
        body = rel.get("body")
        if isinstance(body, str):
            corpus_parts.append(body)
    if isinstance(changelog, str) and changelog:
        corpus_parts.append(changelog)

    corpus = "\n\n".join(corpus_parts)
    breaking = _collect_matching_lines(corpus, BREAKING_PATTERNS, limit=15)
    deprecations = _collect_matching_lines(corpus, DEPRECATION_PATTERNS, limit=15)
    features = _collect_matching_lines(corpus, FEATURE_PATTERNS, limit=20)

    risk = "low"
    if breaking:
        risk = "high"
    elif deprecations:
        risk = "medium"

    refactor_actions: list[str] = []
    if breaking:
        refactor_actions.append("Audit breaking changes and removed APIs in the selected release window.")
        refactor_actions.append("Update affected call sites and run full regression tests for touched modules.")
    if deprecations:
        refactor_actions.append("Replace deprecated APIs/flags before upgrading to next major.")
    if dep_row.get("name") == "@types/node":
        refactor_actions.append("Verify TS configuration and Node globals/types remain aligned with runtime major.")

    if not refactor_actions:
        refactor_actions.append("Apply version bump and run focused tests for modules importing this dependency.")

    confidence = "high" if release_notes else "medium" if dep_row.get("fallback_links") else "low"

    return {
        "breaking_changes": breaking,
        "deprecations": deprecations,
        "feature_adoptions": features,
        "refactor_actions": refactor_actions,
        "risk_level": risk,
        "confidence": confidence,
    }
