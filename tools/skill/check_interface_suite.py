#!/usr/bin/env python3
"""Verify the better-* interface suite agrees with itself across files.

The suite spreads one contract over three kinds of artifact: the router's prose, three JSON
schemas, and three subagent definitions. Nothing else in the repo checks that they still say
the same thing, and three separate review rounds each found a place where they had drifted --
a schema requiring a field its agent was forbidden to produce, a verdict named in prose that
no enum contained, a mode cap stated in two places with two values.

Those are all mechanically checkable. This closes that class:

* every domain skill declares a `## Severity` ladder with exactly HIGH, MEDIUM, LOW
* the verdict vocabulary is identical in the router, the consolidator, and the schema enum
* the mode caps in the router table match the consolidator's instructions
* every `references/` path the router names exists
* the six domain names are spelled identically everywhere they are enumerated

Usage: python3 tools/skill/check_interface_suite.py [repo-root]
"""

from __future__ import annotations

import json
import pathlib
import re
import sys

DOMAINS = [
    "better-accessibility",
    "better-layout",
    "better-writing",
    "better-typography",
    "better-colors",
    "better-ui",
]
LEVELS = ["HIGH", "MEDIUM", "LOW"]
ROUTER = "skills/better-interface/SKILL.md"
CONSOLIDATOR = "subagents/claude/agents/global/interface-consolidator.md"
VERIFIED_SCHEMA = "skills/better-interface/references/verified-schema.json"
CANDIDATE_SCHEMA = "skills/better-interface/references/candidate-schema.json"
FINDINGS_SCHEMA = "skills/better-interface/references/findings-schema.json"

VERDICT_RE = re.compile(r"`(Block|Needs changes|Approve|Inconclusive|No verdict)`")
CAP_ROW_RE = re.compile(r"^\|\s*`(quick|core|full|build)`\s*\|.*\|\s*([^|]*?)\s*\|\s*$", re.M)


def read(root: pathlib.Path, rel: str) -> str:
    return (root / rel).read_text(encoding="utf-8")


def check_severity_ladders(root: pathlib.Path, errors: list[str]) -> None:
    for domain in DOMAINS:
        rel = f"skills/{domain}/SKILL.md"
        text = read(root, rel)
        match = re.search(r"^## Severity\n(.*?)(?=^## )", text, re.S | re.M)
        if not match:
            errors.append(f"{rel}: no '## Severity' section")
            continue
        found = re.findall(r"^- `([A-Z]+)`", match.group(1), re.M)
        if found != LEVELS:
            errors.append(f"{rel}: severity ladder is {found}, expected {LEVELS}")


def check_verdicts(root: pathlib.Path, errors: list[str]) -> None:
    schema = json.loads(read(root, VERIFIED_SCHEMA))
    enum = set(schema["properties"]["verdict"]["enum"])
    for rel in (ROUTER, CONSOLIDATOR):
        used = set(VERDICT_RE.findall(read(root, rel)))
        unknown = used - enum
        if unknown:
            errors.append(f"{rel}: verdict(s) {sorted(unknown)} absent from the schema enum")
        missing = enum - used
        if missing:
            errors.append(f"{rel}: schema verdict(s) {sorted(missing)} never mentioned")


def check_mode_caps(root: pathlib.Path, errors: list[str]) -> None:
    router = read(root, ROUTER)
    caps: dict[str, str] = {}
    for mode, cap in CAP_ROW_RE.findall(router):
        digits = re.findall(r"\d+", cap)
        caps[mode] = digits[0] if digits else "none"
    for mode in ("quick", "core", "full", "build"):
        if mode not in caps:
            errors.append(f"{ROUTER}: mode table has no row for `{mode}`")

    consolidator = read(root, CONSOLIDATOR)
    for mode, cap in caps.items():
        if cap == "none":
            continue
        if not re.search(rf"{cap} for `{mode}`|`{mode}`[^.\n]*{cap}", consolidator):
            errors.append(
                f"{CONSOLIDATOR}: cap for `{mode}` does not match the router's {cap}"
            )


def check_reference_paths(root: pathlib.Path, errors: list[str]) -> None:
    router_dir = (root / ROUTER).parent
    for rel_link in re.findall(r"\]\((references/[^)]+)\)", read(root, ROUTER)):
        if not (router_dir / rel_link).exists():
            errors.append(f"{ROUTER}: broken reference link -> {rel_link}")


def check_domain_enums(root: pathlib.Path, errors: list[str]) -> None:
    for rel in (CANDIDATE_SCHEMA, FINDINGS_SCHEMA):
        schema = json.loads(read(root, rel))
        enum = schema["properties"]["domain"]["enum"]
        if sorted(enum) != sorted(DOMAINS):
            errors.append(f"{rel}: domain enum {sorted(enum)} != {sorted(DOMAINS)}")
    coverage = json.loads(read(root, VERIFIED_SCHEMA))["properties"]["coverage"]
    # Keyed by domain rather than an array: six required keys make "exactly six, none
    # duplicated" structural. An array of six enum-bearing items admits six duplicates.
    if coverage.get("type") != "object":
        errors.append(f"{VERIFIED_SCHEMA}: coverage must be an object keyed by domain")
        return
    if sorted(coverage.get("required", [])) != sorted(DOMAINS):
        errors.append(f"{VERIFIED_SCHEMA}: coverage must require all six domain keys")
    if sorted(coverage.get("properties", {})) != sorted(DOMAINS):
        errors.append(f"{VERIFIED_SCHEMA}: coverage keys != the six domains")
    for domain, entry in coverage.get("properties", {}).items():
        states = entry.get("properties", {}).get("state", {}).get("enum", [])
        if "not-in-scope" not in states:
            errors.append(f"{VERIFIED_SCHEMA}: coverage.{domain} state enum lacks not-in-scope")


def main(argv: list[str]) -> int:
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path.cwd()
    if not (root / ROUTER).exists():
        print(f"interface suite not found at {root / ROUTER}", file=sys.stderr)
        return 1

    errors: list[str] = []
    for check in (
        check_severity_ladders,
        check_verdicts,
        check_mode_caps,
        check_reference_paths,
        check_domain_enums,
    ):
        try:
            check(root, errors)
        except (KeyError, FileNotFoundError, json.JSONDecodeError) as exc:
            errors.append(f"{check.__name__}: {type(exc).__name__}: {exc}")

    if errors:
        for error in errors:
            print(f"INCONSISTENT {error}", file=sys.stderr)
        print(f"interface-suite check: {len(errors)} inconsistency(ies)", file=sys.stderr)
        return 1
    print("interface-suite-ok (severity, verdicts, caps, links, domain enums)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
