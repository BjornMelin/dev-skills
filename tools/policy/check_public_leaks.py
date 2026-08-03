#!/usr/bin/env python3
"""Repo-wide guard: no private-project references in the public catalog.

This repository is PUBLIC and feeds the Agent Skills Lab. Private-project
skills and flavored variants belong in their private repos (for example a
project's own private skills repo), never here.

The check scans every tracked file (full mode) or every staged file
(``--staged``, pre-commit mode) for private-marker patterns, in both file
PATHS and file CONTENTS, including members of packaged ``.skill``/``.zip``
archives. Staged mode reads blob content from the git index, not the working
tree, so a partially staged file is judged by what would actually be
committed. A file that cannot be read fails the check (fail closed). Findings
are reported as ``path:line`` only; the matching text is never echoed, so the
public CI log cannot itself become a leak.

Run: python3 tools/policy/check_public_leaks.py [--staged]
"""

from __future__ import annotations

import argparse
import io
import re
import subprocess
import sys
import zipfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# Private markers. Project names match as whole tokens so ordinary
# identifiers (assignRole, designReview) never trip the guard; the home
# path matches as a literal prefix.
PRIVATE_MARKERS = re.compile(
    r"\bsignr\b|\bcareer-os\b|\btripsage\b|/home/bjorn",
    re.IGNORECASE,
)

ARCHIVE_SUFFIXES = {".skill", ".zip"}

# Only these files may legitimately contain a marker: the guards must name
# the patterns they reject. Nothing else is eligible; sanitize the file
# instead of extending this list.
ALLOWLIST = {
    "tools/policy/check_public_leaks.py",
    "tools/policy/test_check_public_leaks.py",
    "tools/skill/check_expo_motion_public_contract.py",
}


def tracked_files(*, staged: bool) -> list[str]:
    """List the repo-relative paths in scope for the scan.

    Args:
        staged: When true, list files staged for commit (pre-commit
            mode); otherwise list every tracked file.

    Returns:
        Repo-relative POSIX paths.
    """
    if staged:
        cmd = [
            "git",
            "diff",
            "--cached",
            "--name-only",
            "--diff-filter=ACMR",
        ]
    else:
        cmd = ["git", "ls-files"]
    out = subprocess.run(
        cmd, cwd=REPO, capture_output=True, text=True, check=True
    )
    return [line for line in out.stdout.splitlines() if line]


def read_bytes(rel: str, *, staged: bool) -> bytes | None:
    """Read a file's bytes from the index (staged) or working tree.

    Args:
        rel: Repo-relative path.
        staged: Read the staged blob via ``git show :<path>`` when true.

    Returns:
        The file content, or ``None`` when it cannot be read.
    """
    if staged:
        proc = subprocess.run(
            ["git", "show", f":{rel}"],
            cwd=REPO,
            capture_output=True,
        )
        return proc.stdout if proc.returncode == 0 else None
    try:
        return (REPO / rel).read_bytes()
    except OSError:
        return None


def looks_binary(data: bytes) -> bool:
    """Heuristically classify content as binary (NUL in the first 8KiB)."""
    return b"\x00" in data[:8192]


def scan_text(rel: str, data: bytes, failures: list[str]) -> None:
    """Append a ``path:line`` failure for each marker hit in the content."""
    text = data.decode("utf-8", errors="replace")
    for lineno, line in enumerate(text.splitlines(), 1):
        if PRIVATE_MARKERS.search(line):
            failures.append(f"{rel}:{lineno}")


def scan_archive(rel: str, data: bytes, failures: list[str]) -> None:
    """Scan a packaged archive's member names and text members."""
    try:
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            for member in archive.namelist():
                if PRIVATE_MARKERS.search(member):
                    failures.append(f"{rel}!{member}:0 (member name)")
                member_data = archive.read(member)
                if not looks_binary(member_data):
                    scan_text(f"{rel}!{member}", member_data, failures)
    except (zipfile.BadZipFile, OSError):
        failures.append(f"{rel}:0 (unreadable archive; fail closed)")


def main() -> int:
    """Run the scan and return a process exit code."""
    parser = argparse.ArgumentParser()
    parser.add_argument("--staged", action="store_true")
    args = parser.parse_args()

    failures: list[str] = []
    for rel in tracked_files(staged=args.staged):
        if rel in ALLOWLIST:
            continue
        if PRIVATE_MARKERS.search(rel):
            failures.append(f"{rel}:0 (path name)")
        data = read_bytes(rel, staged=args.staged)
        if data is None:
            failures.append(f"{rel}:0 (unreadable; fail closed)")
            continue
        if Path(rel).suffix.lower() in ARCHIVE_SUFFIXES:
            scan_archive(rel, data, failures)
        elif not looks_binary(data):
            scan_text(rel, data, failures)

    if failures:
        print(
            "PUBLIC LEAK CHECK FAILED: private-project references at "
            "(content redacted; open the file locally):",
            file=sys.stderr,
        )
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        print(
            "\nPrivate-project content belongs in its private skills repo. "
            "Sanitize the file; the allowlist is reserved for the guard "
            "scripts themselves.",
            file=sys.stderr,
        )
        return 1
    print("public leak check: clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
