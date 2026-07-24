#!/usr/bin/env python3
"""Model Routing drift gate (v4, calibrated 2026-07-24, CursorBench 3.2).

The routing doctrine lives in ~/.claude/MODELS.md but is mirrored across the
estate: this repo's routing skills, the installed skill copies, the codex
subagent TOMLs (repo source + installed), and ~/.codex/MODELS.md. Mirrors
drift silently when the doctrine recalibrates; this gate makes drift loud.

Repository files are the source of truth and MUST exist (missing = failure);
installed copies are optional per machine and are skipped when absent.

Run from anywhere:  python3 tools/routing/check_model_routing_drift.py
Exit 0 = converged. Exit 1 = drift (each line says where and what).

When the doctrine recalibrates (v5+), update the invariants here IN THE SAME
CHANGE - this file failing after an intentional recalibration is the gate
working, not a bug.
"""

from __future__ import annotations

import hashlib
import os
import re
import sys

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
HOME = os.path.expanduser("~")

ROUTING_SKILLS = ["codex-delegate", "codex-review", "multi-model-review"]

# Sol judgment roles pinned "high" since v4; the four deliberately-cheap
# mechanical roles (ci_triager, shallow_bug_reviewer, test_runner,
# pr_shepherd) stay "medium" by design and are not listed here.
SOL_HIGH_ROLES = [
    "bun_ts_reviewer", "clerk_reviewer", "convex_reviewer", "expo_reviewer",
    "nextjs_reviewer", "openai_api_reviewer", "python_uv_reviewer",
    "react_reviewer", "vercel_reviewer", "reviewer", "runtime_bug_reviewer",
    "performance_reviewer", "history_reviewer", "citation_auditor",
    "docs_auditor", "dependency_researcher", "implementation_worker",
    "docs_aligner", "ui_debugger", "release_validator",
]

# Patterns that must not appear in any v4 routing surface. The standalone
# role name "Fable" is retired (the v4 role is "Root"); the negative
# lookahead keeps legitimate "Fable-5" model references matchable text
# out of scope. opus-4.8 lane pins are retired.
BANNED = [
    (re.compile(r"\bFable\b(?!-\d)"),
     "retired role name 'Fable' (v4 role is 'Root')"),
    (re.compile(r"opus-4[.-]8"), "retired opus-4.8 lane reference"),
    (re.compile(r'"medium"\s*\(Sol worker\) is the (standard|default)'),
     "Sol medium described as default tier (v4 default is high)"),
]

failures: list[str] = []


def fail(msg: str) -> None:
    """Record one drift finding for the final report.

    Args:
        msg: Human-readable description of where and what drifted.
    """
    failures.append(msg)


def read(path: str) -> str | None:
    """Read a text file, tolerating absence.

    Args:
        path: Absolute path to read.

    Returns:
        File contents, or None when the file is missing or unreadable.
    """
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except OSError:
        return None


def sha(text: str) -> str:
    """Return a short content fingerprint for mismatch reporting.

    Args:
        text: Content to fingerprint.

    Returns:
        First 12 hex chars of the SHA-256 digest.
    """
    return hashlib.sha256(text.encode()).hexdigest()[:12]


def check_banned(path: str, label: str, required: bool = False) -> None:
    """Scan one file for retired-doctrine patterns.

    Args:
        path: Absolute path of the file to scan.
        label: Display label used in drift findings.
        required: When True, a missing file is itself a failure.
    """
    text = read(path)
    if text is None:
        if required:
            fail(f"{label}: required source file missing or unreadable")
        return
    for pattern, why in BANNED:
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            fail(f"{label}:{line}: {why}")


def main() -> int:
    """Run every convergence check and report drift.

    Returns:
        Process exit code: 0 when converged, 1 when any drift was found.
    """
    # 1. Repo routing skills exist and carry no retired doctrine.
    for skill in ROUTING_SKILLS:
        check_banned(
            os.path.join(REPO, "skills", skill, "SKILL.md"),
            f"skills/{skill}/SKILL.md",
            required=True,
        )

    # 2. Installed copies match the repo byte-for-byte. The repo side is
    #    required; the installed side is optional per machine.
    for skill in ROUTING_SKILLS:
        repo_path = os.path.join(REPO, "skills", skill, "SKILL.md")
        installed_path = os.path.join(
            HOME, ".agents", "skills", skill, "SKILL.md"
        )
        repo_text = read(repo_path)
        if repo_text is None:
            continue  # already failed as required in check 1
        installed_text = read(installed_path)
        if installed_text is None:
            continue  # not installed on this machine
        if sha(repo_text) != sha(installed_text):
            fail(
                f"~/.agents/skills/{skill}: installed copy diverges from "
                f"repo (repo {sha(repo_text)} vs installed "
                f"{sha(installed_text)}) - re-sync"
            )
        check_banned(
            installed_path, f"~/.agents/skills/{skill}/SKILL.md"
        )

    # 3. Sol judgment roles pinned high: repo TOMLs required, installed
    #    TOMLs optional.
    repo_toml_base = os.path.join(
        REPO, "subagents", "codex", "agents", "global"
    )
    installed_toml_base = os.path.join(HOME, ".codex", "agents")
    for role in SOL_HIGH_ROLES:
        repo_toml = read(os.path.join(repo_toml_base, f"{role}.toml"))
        if repo_toml is None:
            fail(
                f"subagents/codex/agents/global/{role}.toml: required "
                f"role TOML missing"
            )
        elif 'model_reasoning_effort = "high"' not in repo_toml:
            fail(
                f"subagents/codex/agents/global/{role}.toml: judgment "
                f"role not pinned high"
            )
        installed_toml = read(
            os.path.join(installed_toml_base, f"{role}.toml")
        )
        if (installed_toml is not None
                and 'model_reasoning_effort = "high"' not in installed_toml):
            fail(f"~/.codex/agents/{role}.toml: judgment role not pinned high")

    # 4. Live doctrine anchors present (skipped on machines without the
    #    doctrine files - they are per-user config, not repo sources).
    claude_models = read(os.path.join(HOME, ".claude", "MODELS.md"))
    if claude_models is not None:
        anchors = [
            "## Workflow Gate",
            "| root (default) | opus-5 | xhigh |",
        ]
        for anchor in anchors:
            if anchor not in claude_models:
                fail(f"~/.claude/MODELS.md: missing v4 anchor: {anchor!r}")
    codex_models = read(os.path.join(HOME, ".codex", "MODELS.md"))
    if (codex_models is not None
            and "| worker (default) | gpt-5.6-sol | high |"
            not in codex_models):
        fail("~/.codex/MODELS.md: Sol worker default is not high")

    if failures:
        print(f"MODEL ROUTING DRIFT ({len(failures)}):")
        for item in failures:
            print(f"  - {item}")
        return 1
    print("model routing converged (v4)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
