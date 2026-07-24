#!/usr/bin/env python3
"""Model Routing drift gate (v4, calibrated 2026-07-24, CursorBench 3.2).

The routing doctrine lives in ~/.claude/MODELS.md but is mirrored across the
estate: this repo's routing skills, the installed skill copies, the codex
subagent TOMLs (repo source + installed), and ~/.codex/MODELS.md. Mirrors
drift silently when the doctrine recalibrates; this gate makes drift loud.

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

# Patterns that must not appear in any v4 routing surface. "Fable" as a role
# name is retired (the role is "Root"); opus-4.8 lane pins are retired.
BANNED = [
    (re.compile(r"\bFable\b"), "retired role name 'Fable' (v4 role is 'Root')"),
    (re.compile(r"opus-4[.-]8"), "retired opus-4.8 lane reference"),
    (re.compile(r'"medium"\s*\(Sol worker\) is the (standard|default)'),
     "Sol medium described as default tier (v4 default is high)"),
]

failures: list[str] = []


def fail(msg: str) -> None:
    failures.append(msg)


def read(path: str) -> str | None:
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except OSError:
        return None


def sha(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()[:12]


def check_banned(path: str, label: str) -> None:
    text = read(path)
    if text is None:
        return
    for pattern, why in BANNED:
        for match in pattern.finditer(text):
            line = text.count("\n", 0, match.start()) + 1
            fail(f"{label}:{line}: {why}")


def main() -> int:
    # 1. Repo routing skills carry no retired doctrine.
    for skill in ROUTING_SKILLS:
        check_banned(os.path.join(REPO, "skills", skill, "SKILL.md"),
                     f"skills/{skill}/SKILL.md")

    # 2. Installed copies match the repo byte-for-byte (skip if not installed).
    for skill in ROUTING_SKILLS:
        repo_path = os.path.join(REPO, "skills", skill, "SKILL.md")
        installed_path = os.path.join(HOME, ".agents", "skills", skill, "SKILL.md")
        repo_text, installed_text = read(repo_path), read(installed_path)
        if repo_text is None or installed_text is None:
            continue
        if sha(repo_text) != sha(installed_text):
            fail(f"~/.agents/skills/{skill}: installed copy diverges from repo "
                 f"(repo {sha(repo_text)} vs installed {sha(installed_text)}) - re-sync")
        check_banned(installed_path, f"~/.agents/skills/{skill}/SKILL.md")

    # 3. Sol judgment roles pinned high in repo TOMLs and installed TOMLs.
    for role in SOL_HIGH_ROLES:
        for base, label in [
            (os.path.join(REPO, "subagents", "codex", "agents", "global"),
             "subagents/codex/agents/global"),
            (os.path.join(HOME, ".codex", "agents"), "~/.codex/agents"),
        ]:
            text = read(os.path.join(base, f"{role}.toml"))
            if text is None:
                continue
            if 'model_reasoning_effort = "high"' not in text:
                fail(f"{label}/{role}.toml: judgment role not pinned high")

    # 4. Live doctrine anchors present (skip silently on machines without them).
    claude_models = read(os.path.join(HOME, ".claude", "MODELS.md"))
    if claude_models is not None:
        for anchor in ["## Workflow Gate", "| root (default) | opus-5 | xhigh |"]:
            if anchor not in claude_models:
                fail(f"~/.claude/MODELS.md: missing v4 anchor: {anchor!r}")
    codex_models = read(os.path.join(HOME, ".codex", "MODELS.md"))
    if codex_models is not None:
        if "| worker (default) | gpt-5.6-sol | high |" not in codex_models:
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
