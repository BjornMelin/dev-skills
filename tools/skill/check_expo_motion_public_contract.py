#!/usr/bin/env python3
"""Keep the public expo-motion skill and audit contract generic and aligned."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

FORBIDDEN = (
    re.compile(r"\bnpx\b|\bnpm\b", re.IGNORECASE),
    re.compile(
        r"\bSDK\s*\d+(?:\.\d+)*\b|\bRN\s*\d+(?:\.\d+)*\b",
        re.IGNORECASE,
    ),
    re.compile(r"signr|career-os|tripsage|/home/bjorn", re.IGNORECASE),
)
REQUIRED = (
    "newArchEnabled",
    "packageManager",
    "scheduleOnRN",
    "useReducedMotion",
)
SUPPORTED_DOCTOR_FORMATS = frozenset({"markdown", "json"})
DOCTOR_COMMAND_RE = re.compile(
    r"(?m)^\s*expo-motion-audit\s+doctor"
    r"(?:\s+--format\s+(?P<format>[a-z]+))?\s*$"
)
MARKDOWN_LINK_RE = re.compile(
    r"\]\((?![a-z][a-z0-9+.-]*:|//|#)([^)#\s]+)(?:#[^)]+)?\)"
)


def _public_contract_files(root: Path, skill: Path) -> list[Path]:
    """Return the skill and its canonical public audit surfaces."""
    files = [path for path in skill.rglob("*") if path.is_file()]
    for path in (
        root / "docs" / "reference" / "expo-motion-audit.md",
        root / "crates" / "expo-motion-audit-core",
        root / "crates" / "expo-motion-audit",
    ):
        if path.is_file():
            files.append(path)
        elif path.is_dir():
            files.extend(child for child in path.rglob("*") if child.is_file())
    return files


def _frontmatter_description(skill_path: Path) -> str:
    text = skill_path.read_text(encoding="utf-8")
    match = re.match(r"^---\n(.*?)\n---\n", text, re.DOTALL)
    if not match:
        raise ValueError("SKILL.md is missing YAML frontmatter")
    description = re.search(
        r"^description:\s*(.+)$", match.group(1), re.MULTILINE
    )
    if not description:
        raise ValueError("SKILL.md frontmatter is missing description")
    value = description.group(1).strip()
    if value.startswith('"') and value.endswith('"'):
        return json.loads(value)
    if value.startswith("'") and value.endswith("'"):
        return value[1:-1].replace("''", "'")
    return value


def _catalog_skill(value: Any) -> dict[str, Any] | None:
    if isinstance(value, dict):
        if value.get("name") == "expo-motion":
            return value
        for child in value.values():
            found = _catalog_skill(child)
            if found:
                return found
    elif isinstance(value, list):
        for child in value:
            found = _catalog_skill(child)
            if found:
                return found
    return None


def check(root: Path) -> list[str]:
    """Validate the public expo-motion skill and audit contracts.

    Args:
        root: Repository root containing the skill, catalog, and audit docs.

    Returns:
        A list of validation errors, empty when all public contracts pass.
    """
    skill = root / "skills" / "expo-motion"
    skill_entrypoint = skill / "SKILL.md"
    catalog = root / "catalog" / "agent-skills-lab.json"
    errors: list[str] = []

    if not skill_entrypoint.is_file():
        return [f"missing {skill_entrypoint}"]

    skill_files = [path for path in skill.rglob("*") if path.is_file()]
    for path in _public_contract_files(root, skill):
        if path.suffix.lower() not in {".md", ".json", ".yaml", ".yml", ".rs"}:
            continue
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(root)
        for pattern in FORBIDDEN:
            if pattern.search(text):
                errors.append(
                    "forbidden public-contract text in "
                    f"{relative}: {pattern.pattern}"
                )

    body = "\n".join(
        path.read_text(encoding="utf-8")
        for path in skill_files
        if path.suffix == ".md"
    )
    for required in REQUIRED:
        if required not in body:
            errors.append(f"missing manifest/API contract marker: {required}")

    for path in skill.rglob("*.md"):
        text = path.read_text(encoding="utf-8")
        for target in MARKDOWN_LINK_RE.findall(text):
            if not (path.parent / target).is_file():
                errors.append(
                    "broken local reference in "
                    f"{path.relative_to(root)}: {target}"
                )

    audit_reference = root / "docs" / "reference" / "expo-motion-audit.md"
    if audit_reference.is_file():
        doctor_formats = {
            match.group("format") or "markdown"
            for match in DOCTOR_COMMAND_RE.finditer(
                audit_reference.read_text(encoding="utf-8")
            )
        }
        for unsupported in sorted(doctor_formats - SUPPORTED_DOCTOR_FORMATS):
            errors.append(
                f"unsupported expo-motion-audit doctor format in "
                f"{audit_reference.relative_to(root)}: {unsupported}"
            )
        for missing in sorted(SUPPORTED_DOCTOR_FORMATS - doctor_formats):
            errors.append(
                f"missing expo-motion-audit doctor format in "
                f"{audit_reference.relative_to(root)}: {missing}"
            )

    if catalog.is_file():
        try:
            catalog_value = json.loads(catalog.read_text(encoding="utf-8"))
            catalog_entry = _catalog_skill(catalog_value)
            description = _frontmatter_description(skill_entrypoint)
            if not catalog_entry:
                errors.append("catalog is missing expo-motion")
            elif catalog_entry.get("description") != description:
                errors.append(
                    "catalog description does not match SKILL.md frontmatter"
                )
        except (OSError, ValueError, json.JSONDecodeError) as error:
            errors.append(f"catalog/frontmatter parse failed: {error}")
    else:
        errors.append(f"missing {catalog}")

    return errors


def main() -> int:
    """Run the public-contract checker from the command line.

    Returns:
        Zero when the contract passes; otherwise one.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root", type=Path, default=Path(__file__).resolve().parents[2]
    )
    args = parser.parse_args()
    errors = check(args.root.resolve())
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print("expo-motion-public-contract-ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
