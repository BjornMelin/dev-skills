#!/usr/bin/env python3
"""Verify the better-* interface suite agrees with itself across files.

The suite spreads one contract over three kinds of artifact: the router's
prose, three JSON schemas, and three subagent definitions. Nothing else in the
repo checks that they still say the same thing, and three separate review
rounds each found a place where they had drifted -- a schema requiring a field
its agent was forbidden to produce, a verdict named in prose that no enum
contained, a mode cap stated in two places with two values.

Those are all mechanically checkable. This closes that class:

* every domain skill declares a `## Severity` ladder with exactly HIGH,
  MEDIUM, LOW
* the verdict vocabulary is identical in the router, the consolidator, and
  the schema enum
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
WCAG_SKILL = "skills/wcag-audit-patterns/SKILL.md"

# Every WCAG 2.2 Level A and AA success criterion, per
# https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/ and the WCAG 2.2
# recommendation. 4.1.1 Parsing is deliberately absent: it is obsolete in 2.2.
WCAG_A_AA = [
    "1.1.1", "1.2.1", "1.2.2", "1.2.3", "1.2.4", "1.2.5",
    "1.3.1", "1.3.2", "1.3.3", "1.3.4", "1.3.5",
    "1.4.1", "1.4.2", "1.4.3", "1.4.4", "1.4.5",
    "1.4.10", "1.4.11", "1.4.12", "1.4.13",
    "2.1.1", "2.1.2", "2.1.4", "2.2.1", "2.2.2", "2.3.1",
    "2.4.1", "2.4.2", "2.4.3", "2.4.4", "2.4.5", "2.4.6", "2.4.7", "2.4.11",
    "2.5.1", "2.5.2", "2.5.3", "2.5.4", "2.5.7", "2.5.8",
    "3.1.1", "3.1.2", "3.2.1", "3.2.2", "3.2.3", "3.2.4", "3.2.6",
    "3.3.1", "3.3.2", "3.3.3", "3.3.4", "3.3.7", "3.3.8",
    "4.1.2", "4.1.3",
]

VERDICT_RE = re.compile(
    r"`(Block|Needs changes|Approve|Inconclusive|No verdict)`"
)
# Anchored to the mode table specifically. Scanning every table let a later
# row that also begins with a mode name (Principle 6's tool-access matrix)
# overwrite the real cap, after which this check silently passed regardless of
# what the consolidator said.
MODE_TABLE_RE = re.compile(
    r"^\| Mode \| Coverage \| Finding cap \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
    re.M,
)
CAP_ROW_RE = re.compile(
    r"^\|\s*`(quick|core|full|build)`\s*\|.*\|\s*([^|]*?)\s*\|\s*$",
    re.M,
)


def read(root: pathlib.Path, rel: str) -> str:
    """Read a tracked file relative to the repository root.

    Args:
        root: Repository root directory.
        rel: File path relative to the root.

    Returns:
        The file contents as text.

    Raises:
        OSError: When the file cannot be read.
    """
    return (root / rel).read_text(encoding="utf-8")


def check_severity_ladders(root: pathlib.Path, errors: list[str]) -> None:
    """Require each domain skill to declare the shared severity ladder.

    Args:
        root: Repository root directory.
        errors: Accumulates human-readable violations in place.
    """
    for domain in DOMAINS:
        rel = f"skills/{domain}/SKILL.md"
        text = read(root, rel)
        match = re.search(r"^## Severity\n(.*?)(?=^## )", text, re.S | re.M)
        if not match:
            errors.append(f"{rel}: no '## Severity' section")
            continue
        found = re.findall(r"^- `([A-Z]+)`", match.group(1), re.M)
        if found != LEVELS:
            errors.append(
                f"{rel}: severity ladder is {found}, expected {LEVELS}"
            )


def check_verdicts(root: pathlib.Path, errors: list[str]) -> None:
    """Require the verdict vocabulary to match the verified-schema enum.

    Args:
        root: Repository root directory.
        errors: Accumulates human-readable violations in place.
    """
    schema = json.loads(read(root, VERIFIED_SCHEMA))
    enum = set(schema["properties"]["verdict"]["enum"])
    for rel in (ROUTER, CONSOLIDATOR):
        used = set(VERDICT_RE.findall(read(root, rel)))
        unknown = used - enum
        if unknown:
            errors.append(
                f"{rel}: verdict(s) {sorted(unknown)} absent from the "
                f"schema enum"
            )
        missing = enum - used
        if missing:
            errors.append(
                f"{rel}: schema verdict(s) {sorted(missing)} never mentioned"
            )


def check_mode_caps(root: pathlib.Path, errors: list[str]) -> None:
    """Require router mode caps to match the consolidator's stated caps.

    Args:
        root: Repository root directory.
        errors: Accumulates human-readable violations in place.
    """
    router = read(root, ROUTER)
    table = MODE_TABLE_RE.search(router)
    if not table:
        errors.append(
            f"{ROUTER}: could not find the Mode/Coverage/Finding cap table"
        )
        return
    caps: dict[str, str] = {}
    for mode, cap in CAP_ROW_RE.findall(table.group(1)):
        if mode in caps:
            errors.append(
                f"{ROUTER}: duplicate mode row for `{mode}` in the mode table"
            )
        digits = re.findall(r"\d+", cap)
        caps[mode] = digits[0] if digits else "none"
    for mode in ("quick", "core", "full", "build"):
        if mode not in caps:
            errors.append(f"{ROUTER}: mode table has no row for `{mode}`")

    # Parse the consolidator's own "N for `mode`" pairs and compare dicts.
    # A loose regex per mode is not enough: `` `full`[^.\n]*15 `` happily
    # matches the *next* mode's cap in "99 for `full`, 15 for `build`", so a
    # real drift passed.
    consolidator = read(root, CONSOLIDATOR)
    cap_pairs = re.findall(
        r"(\d+) for `(quick|core|full|build)`", consolidator
    )
    stated = {mode: cap for cap, mode in cap_pairs}
    for mode, cap in caps.items():
        if cap == "none":
            continue
        if mode not in stated:
            errors.append(
                f"{CONSOLIDATOR}: states no cap for `{mode}` "
                f"(router says {cap})"
            )
        elif stated[mode] != cap:
            errors.append(
                f"{CONSOLIDATOR}: cap for `{mode}` is {stated[mode]}, "
                f"router says {cap}"
            )


def check_reference_paths(root: pathlib.Path, errors: list[str]) -> None:
    """Require every reference link the router names to resolve.

    Args:
        root: Repository root directory.
        errors: Accumulates human-readable violations in place.
    """
    router_dir = (root / ROUTER).parent
    for rel_link in re.findall(r"\]\((references/[^)]+)\)", read(root, ROUTER)):
        if not (router_dir / rel_link).exists():
            errors.append(f"{ROUTER}: broken reference link -> {rel_link}")


def check_domain_enums(root: pathlib.Path, errors: list[str]) -> None:
    """Require every schema to enumerate the six domains identically.

    Args:
        root: Repository root directory.
        errors: Accumulates human-readable violations in place.
    """
    for rel in (CANDIDATE_SCHEMA, FINDINGS_SCHEMA):
        schema = json.loads(read(root, rel))
        enum = schema["properties"]["domain"]["enum"]
        if sorted(enum) != sorted(DOMAINS):
            errors.append(
                f"{rel}: domain enum {sorted(enum)} != {sorted(DOMAINS)}"
            )
    coverage = json.loads(read(root, VERIFIED_SCHEMA))["properties"]["coverage"]
    # Keyed by domain rather than an array: six required keys make "exactly
    # six, none duplicated" structural. An array of six enum-bearing items
    # admits six duplicates.
    if coverage.get("type") != "object":
        errors.append(
            f"{VERIFIED_SCHEMA}: coverage must be an object keyed by domain"
        )
        return
    if sorted(coverage.get("required", [])) != sorted(DOMAINS):
        errors.append(
            f"{VERIFIED_SCHEMA}: coverage must require all six domain keys"
        )
    if sorted(coverage.get("properties", {})) != sorted(DOMAINS):
        errors.append(f"{VERIFIED_SCHEMA}: coverage keys != the six domains")
    for domain, entry in coverage.get("properties", {}).items():
        states = entry.get("properties", {}).get("state", {}).get("enum", [])
        if "not-in-scope" not in states:
            errors.append(
                f"{VERIFIED_SCHEMA}: coverage.{domain} state enum "
                f"lacks not-in-scope"
            )


def check_wcag_coverage(root: pathlib.Path, errors: list[str]) -> None:
    """Require the WCAG audit checklist to carry every Level A and AA criterion.

    The skill advertises WCAG 2.2 audits and names VPAT and legal conformance
    among its uses, so a silently missing criterion is worse than an obviously
    partial checklist -- a reviewer following it reports a clean pass over an
    unexamined requirement. A whole guideline (2.5 Input Modalities) went absent
    that way. AAA is out of scope by the skill's own scope statement.

    Args:
        root: Repository root directory.
        errors: Accumulates human-readable violations in place.
    """
    path = root / WCAG_SKILL
    if not path.exists():
        errors.append(f"{WCAG_SKILL}: missing")
        return
    text = path.read_text(encoding="utf-8")
    present = set(re.findall(r"^###\s+(\d+\.\d+\.\d+)\s", text, re.M))
    missing = [sc for sc in WCAG_A_AA if sc not in present]
    if missing:
        errors.append(f"{WCAG_SKILL}: missing WCAG 2.2 A/AA criteria {missing}")


def main(argv: list[str]) -> int:
    """Run every interface-suite consistency check.

    Args:
        argv: Command-line arguments; an optional repository root.

    Returns:
        0 when the suite is consistent, 1 otherwise.
    """
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
        check_wcag_coverage,
    ):
        try:
            check(root, errors)
        except (KeyError, FileNotFoundError, json.JSONDecodeError) as exc:
            errors.append(f"{check.__name__}: {type(exc).__name__}: {exc}")

    if errors:
        for error in errors:
            print(f"INCONSISTENT {error}", file=sys.stderr)
        print(
            f"interface-suite check: {len(errors)} inconsistency(ies)",
            file=sys.stderr,
        )
        return 1
    print(
        "interface-suite-ok (severity, verdicts, caps, links, domain enums, "
        "wcag A/AA coverage)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
