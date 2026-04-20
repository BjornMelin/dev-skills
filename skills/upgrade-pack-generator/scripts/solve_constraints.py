#!/usr/bin/env python3
"""Semver-ish constraint helpers for upgrade-pack-generator."""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Iterable


SEMVER_RE = re.compile(
    r"^\s*v?(?P<major>\d+)(?:\.(?P<minor>\d+|x|\*))?(?:\.(?P<patch>\d+|x|\*))?(?:-(?P<prerelease>[0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?\s*$"
)
TOKEN_RE = re.compile(r"(>=|<=|>|<|=|\^|~)?\s*v?(\d+|x|\*)(?:\.(\d+|x|\*))?(?:\.(\d+|x|\*))?(?:-([0-9A-Za-z.-]+))?")


@dataclass(frozen=True, order=True)
class Semver:
    """Comparable semver tuple."""

    major: int
    minor: int
    patch: int
    prerelease: str = ""

    @property
    def is_prerelease(self) -> bool:
        return bool(self.prerelease)


def parse_semver(raw: str | None) -> Semver | None:
    """Parse a semver-ish string into a comparable tuple."""
    if not raw:
        return None
    match = SEMVER_RE.match(raw.strip())
    if not match:
        return None
    major = int(match.group("major"))
    minor_token = match.group("minor")
    patch_token = match.group("patch")
    if minor_token in {None, "x", "*"}:
        minor = 0
    else:
        minor = int(minor_token)
    if patch_token in {None, "x", "*"}:
        patch = 0
    else:
        patch = int(patch_token)
    return Semver(major=major, minor=minor, patch=patch, prerelease=match.group("prerelease") or "")


def _token_version(major: str, minor: str | None, patch: str | None, prerelease: str | None) -> tuple[Semver, bool, bool]:
    wildcard_minor = minor in {None, "x", "*"}
    wildcard_patch = patch in {None, "x", "*"}
    return (
        Semver(
            major=int(major) if major not in {"x", "*"} else 0,
            minor=0 if wildcard_minor else int(minor or 0),
            patch=0 if wildcard_patch else int(patch or 0),
            prerelease=prerelease or "",
        ),
        wildcard_minor,
        wildcard_patch,
    )


def _bump_major(version: Semver) -> Semver:
    return Semver(version.major + 1, 0, 0)


def _bump_minor(version: Semver) -> Semver:
    return Semver(version.major, version.minor + 1, 0)


def _bump_patch(version: Semver) -> Semver:
    return Semver(version.major, version.minor, version.patch + 1)


def _expand_token(operator: str | None, major: str, minor: str | None, patch: str | None, prerelease: str | None) -> list[tuple[str, Semver]]:
    version, wildcard_minor, wildcard_patch = _token_version(major, minor, patch, prerelease)
    op = operator or "="
    if major in {"x", "*"}:
        return []
    if op == "^":
        upper = _bump_major(version) if version.major > 0 else _bump_minor(version)
        return [(">=", version), ("<", upper)]
    if op == "~":
        return [(">=", version), ("<", _bump_minor(version))]
    if op == "=" and wildcard_minor:
        return [(">=", version), ("<", _bump_major(version))]
    if op == "=" and wildcard_patch:
        return [(">=", version), ("<", _bump_minor(version))]
    return [(op, version)]


def _iter_branch_constraints(branch: str) -> Iterable[list[tuple[str, Semver]]]:
    tokens = TOKEN_RE.findall(branch)
    if not tokens:
        return []
    expanded: list[tuple[str, Semver]] = []
    for operator, major, minor, patch, prerelease in tokens:
        expanded.extend(_expand_token(operator or None, major, minor or None, patch or None, prerelease or None))
    return [expanded]


def compare(actual: Semver, operator: str, expected: Semver) -> bool:
    """Evaluate one comparator."""
    if operator == "=":
        return actual == expected
    if operator == ">":
        return actual > expected
    if operator == ">=":
        return actual >= expected
    if operator == "<":
        return actual < expected
    if operator == "<=":
        return actual <= expected
    return False


def satisfies(actual_version: str | None, range_spec: str | None) -> bool | None:
    """Return whether a version satisfies a semver-ish range."""
    if not range_spec or range_spec.strip() in {"*", "latest"}:
        return True
    actual = parse_semver(actual_version)
    if actual is None:
        return None
    for branch in range_spec.split("||"):
        branch = branch.strip()
        if not branch:
            continue
        groups = list(_iter_branch_constraints(branch))
        if not groups:
            continue
        for constraints in groups:
            if all(compare(actual, operator, expected) for operator, expected in constraints):
                return True
    return False


def select_highest_stable(versions: list[str]) -> str | None:
    """Return the highest stable version from a list."""
    parsed = [parse_semver(version) for version in versions]
    stable = [
        (version, semver)
        for version, semver in zip(versions, parsed, strict=False)
        if semver is not None and not semver.is_prerelease
    ]
    if not stable:
        return None
    stable.sort(key=lambda item: item[1], reverse=True)
    return stable[0][0]
