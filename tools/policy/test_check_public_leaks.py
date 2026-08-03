#!/usr/bin/env python3
"""Regression tests for the public-leak guard's security contract.

Covers: token-boundary matching (no substring false positives), path-name
leaks, archive member scanning, staged index-vs-worktree divergence, and
fail-closed behavior for unreadable selections. Runs against throwaway git
repos; no network, no fixtures left behind.

Run: python3 tools/policy/test_check_public_leaks.py
"""

from __future__ import annotations

import io
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

GUARD = Path(__file__).resolve().parent / "check_public_leaks.py"


def run_guard(repo: Path, *, staged: bool = False) -> tuple[int, str]:
    """Run the guard against a repo; return (exit code, stderr)."""
    guard_copy = repo / "tools" / "policy" / "check_public_leaks.py"
    guard_copy.parent.mkdir(parents=True, exist_ok=True)
    guard_copy.write_text(
        GUARD.read_text(encoding="utf-8"), encoding="utf-8"
    )
    args = [sys.executable, str(guard_copy)]
    if staged:
        args.append("--staged")
    proc = subprocess.run(args, cwd=repo, capture_output=True, text=True)
    return proc.returncode, proc.stderr


def git(repo: Path, *args: str) -> None:
    """Run a git command in the fixture repo."""
    subprocess.run(
        ["git", *args],
        cwd=repo,
        check=True,
        capture_output=True,
        env={
            "GIT_AUTHOR_NAME": "t",
            "GIT_AUTHOR_EMAIL": "t@t",
            "GIT_COMMITTER_NAME": "t",
            "GIT_COMMITTER_EMAIL": "t@t",
            "HOME": str(repo),
            "PATH": "/usr/bin:/bin",
        },
    )


def make_repo(tmp: str) -> Path:
    """Create a minimal committed repo fixture."""
    repo = Path(tmp) / "fixture"
    (repo / ".git").parent.mkdir(parents=True)
    git(repo, "init", "-q")
    (repo / "README.md").write_text("clean\n", encoding="utf-8")
    git(repo, "add", "README.md")
    git(repo, "commit", "-qm", "init")
    return repo


def expect(cond: bool, label: str, failures: list[str]) -> None:
    """Record a labelled assertion result."""
    print(f"  {'ok' if cond else 'FAIL'}  {label}")
    if not cond:
        failures.append(label)


def main() -> int:
    """Run every regression case; return a process exit code."""
    failures: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        repo = make_repo(tmp)

        # 1. Token boundary: camel-case identifiers must NOT trip.
        (repo / "ok.ts").write_text(
            "const assignRole = designReview;\n", encoding="utf-8"
        )
        git(repo, "add", "ok.ts")
        git(repo, "commit", "-qm", "ok")
        code, _ = run_guard(repo)
        expect(code == 0, "camel-case identifiers pass", failures)

        # 2. Content marker in a root file (no prefix blind spot).
        (repo / "README.md").write_text(
            "see the signr project\n", encoding="utf-8"
        )
        git(repo, "add", "README.md")
        git(repo, "commit", "-qm", "leak")
        code, err = run_guard(repo)
        expect(code == 1, "root README content leak fails", failures)
        expect("signr" not in err, "leak text is redacted", failures)
        git(repo, "revert", "-n", "HEAD")
        git(repo, "commit", "-qm", "clean")

        # 3. Path-name leak with generic content.
        leak_dir = repo / "skills" / "x" / "references"
        leak_dir.mkdir(parents=True)
        (leak_dir / "signr.md").write_text("generic\n", encoding="utf-8")
        git(repo, "add", "-A", "skills")
        git(repo, "commit", "-qm", "path")
        code, _ = run_guard(repo)
        expect(code == 1, "path-name leak fails", failures)
        git(repo, "rm", "-qr", "skills")
        git(repo, "commit", "-qm", "rm")

        # 4. Archive member leak inside a .skill zip.
        buf = io.BytesIO()
        with zipfile.ZipFile(buf, "w") as zf:
            zf.writestr("inner/notes.md", "for the signr backend\n")
        (repo / "pack.skill").write_bytes(buf.getvalue())
        git(repo, "add", "pack.skill")
        git(repo, "commit", "-qm", "zip")
        code, _ = run_guard(repo)
        expect(code == 1, "archive member content leak fails", failures)
        git(repo, "rm", "-q", "pack.skill")
        git(repo, "commit", "-qm", "rmzip")

        # 5. Staged mode reads the INDEX, not the working tree.
        bad = repo / "staged.md"
        bad.write_text("mentions signr here\n", encoding="utf-8")
        git(repo, "add", "staged.md")
        bad.write_text("scrubbed in worktree only\n", encoding="utf-8")
        code, _ = run_guard(repo, staged=True)
        expect(code == 1, "staged blob leak caught despite clean "
               "worktree", failures)
        git(repo, "reset", "-q", "staged.md")
        bad.unlink()

        # 6. Fail closed: a tracked-but-missing file is a failure.
        (repo / "ghost.md").write_text("clean\n", encoding="utf-8")
        git(repo, "add", "ghost.md")
        git(repo, "commit", "-qm", "ghost")
        (repo / "ghost.md").unlink()
        code, _ = run_guard(repo)
        expect(code == 1, "unreadable selection fails closed", failures)

    if failures:
        print(f"\n{len(failures)} case(s) failed", file=sys.stderr)
        return 1
    print("\nall public-leak guard regression cases pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
