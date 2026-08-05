#!/usr/bin/env python3
"""Regression tests for the interface-suite consistency gate.

The suite spreads one contract across prose, JSON schemas, and agent
definitions. This gate checks they agree. Each case below copies the real
repository files into a temporary root, injects one specific disagreement, and
asserts the gate rejects it.

Three cases reproduce defects the gate itself once had:

* `cap_row_from_principle_matrix` -- the cap pattern scanned every table, so
  Principle 6's tool-access matrix, whose rows also begin with a mode name,
  overwrote the real `build` cap. Drift then passed silently.
* `cap_drift_matching_next_mode` -- the per-mode fallback could match a *later*
  mode's number, so changing one cap still found a match and reported success.
* `wcag_criterion_removed` -- a whole WCAG guideline went missing from a skill
  that advertises conformance audits, with nothing to catch it.

A gate that has never been run against the failure it exists to catch is an
assertion, not a test. These are committed so that stays true.

Run: python3 tools/skill/test_check_interface_suite.py
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
GATE = REPO / "tools" / "skill" / "check_interface_suite.py"

ROUTER = "skills/better-interface/SKILL.md"
CONSOLIDATOR = "subagents/claude/agents/global/interface-consolidator.md"
VERIFIED_SCHEMA = "skills/better-interface/references/verified-schema.json"
CANDIDATE_SCHEMA = "skills/better-interface/references/candidate-schema.json"
WCAG_SKILL = "skills/wcag-audit-patterns/SKILL.md"

DOMAINS = [
    "better-accessibility",
    "better-layout",
    "better-writing",
    "better-typography",
    "better-colors",
    "better-ui",
]

TRACKED = [
    ROUTER,
    CONSOLIDATOR,
    VERIFIED_SCHEMA,
    CANDIDATE_SCHEMA,
    WCAG_SKILL,
    *[f"skills/{d}/SKILL.md" for d in DOMAINS],
]


def make_root(tmp: str) -> Path:
    """Copy the real suite files into a throwaway root."""
    root = Path(tmp) / "fixture"
    for rel in TRACKED:
        dst = root / rel
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(REPO / rel, dst)
    # The gate resolves reference links relative to the skill directory.
    src_refs = REPO / "skills" / "better-interface" / "references"
    dst_refs = root / "skills" / "better-interface" / "references"
    for path in src_refs.iterdir():
        if path.is_file():
            shutil.copy2(path, dst_refs / path.name)
    return root


def run_gate(root: Path) -> tuple[int, str]:
    """Run the gate against a fixture root; return (exit code, output)."""
    proc = subprocess.run(
        [sys.executable, str(GATE), str(root)],
        capture_output=True,
        text=True,
    )
    return proc.returncode, proc.stdout + proc.stderr


def patch(root: Path, rel: str, old: str, new: str, every: bool) -> bool:
    """Replace `old` with `new` in a fixture file; False if `old` is absent.

    `every` matters for vocabulary checks: the router states most verdicts
    twice, so replacing one occurrence leaves the other satisfying the gate and
    the case passes for the wrong reason.
    """
    path = root / rel
    text = path.read_text(encoding="utf-8")
    if old not in text:
        return False
    path.write_text(
        text.replace(old, new) if every else text.replace(old, new, 1),
        encoding="utf-8",
    )
    return True


def expect(cond: bool, label: str, failures: list[str]) -> None:
    """Record a labelled assertion result."""
    print(f"  {'ok' if cond else 'FAIL'}  {label}")
    if not cond:
        failures.append(label)


def case(
    label: str,
    rel: str,
    old: str,
    new: str,
    failures: list[str],
    every: bool = False,
) -> None:
    """Inject one disagreement and require the gate to reject it."""
    with tempfile.TemporaryDirectory() as tmp:
        root = make_root(tmp)
        if not patch(root, rel, old, new, every):
            # The anchor text moved. Fail loudly: a silently skipped case is
            # exactly the failure mode these tests exist to prevent.
            expect(False, f"{label} [ANCHOR NOT FOUND in {rel}]", failures)
            return
        code, out = run_gate(root)
        expect(code == 1 and "INCONSISTENT" in out, label, failures)


def main() -> int:
    """Run every regression case; return a process exit code."""
    failures: list[str] = []

    print("baseline:")
    with tempfile.TemporaryDirectory() as tmp:
        root = make_root(tmp)
        code, out = run_gate(root)
        expect(
            code == 0 and "interface-suite-ok" in out,
            "unmodified suite passes",
            failures,
        )

    print("\nrejects injected drift:")

    # The bug that made this gate report success on real drift: Principle 6's
    # tool-access matrix rows also start with a mode name.
    # `build` specifically: the router contains a *second* table whose rows
    # also begin with "| `build` |" (Principle 6's tool-access matrix). The
    # gate must read the cap from the mode table and ignore that one.
    case(
        "mode cap drift (build)",
        ROUTER,
        "15, scoped to the diff |",
        "99, scoped to the diff |",
        failures,
    )
    case(
        "mode cap drift (core)",
        ROUTER,
        "| `core` |",
        "| `core-renamed` |",
        failures,
    )

    # Coverage honesty: all six domains must be enumerated in the schema.
    case(
        "domain dropped from verified-schema coverage",
        VERIFIED_SCHEMA,
        '"better-ui"',
        '"better-ui-typo"',
        failures,
    )
    case(
        "domain enum drift in candidate-schema",
        CANDIDATE_SCHEMA,
        '"better-colors"',
        '"better-colours"',
        failures,
    )

    # Every domain skill must declare the same three-level ladder, so a lane's
    # severity means the same thing whichever skill produced it.
    case(
        "domain skill renames a severity level",
        "skills/better-ui/SKILL.md",
        "- `HIGH`",
        "- `CRITICAL`",
        failures,
    )
    case(
        "domain skill loses its ## Severity section",
        "skills/better-colors/SKILL.md",
        "## Severity",
        "## Severity notes (prose)",
        failures,
    )

    # Verdict vocabulary is checked in both directions against the schema enum.
    case(
        "verdict wording drifts from the schema enum",
        ROUTER,
        "`Needs changes`",
        "`Needs-changes`",
        failures,
        every=True,
    )
    case(
        "consolidator invents a verdict the schema does not define",
        CONSOLIDATOR,
        "`Approve`",
        "`Ship it`",
        failures,
        every=True,
    )

    # WCAG conformance coverage.
    case(
        "WCAG criterion removed (whole guideline regression)",
        WCAG_SKILL,
        "### 2.5.8 Target Size (Minimum) (Level AA) - WCAG 2.2",
        "### Target size notes",
        failures,
    )
    case(
        "WCAG 2.2 addition removed (3.3.8)",
        WCAG_SKILL,
        "### 3.3.8 Accessible Authentication (Minimum) (Level AA) - WCAG 2.2",
        "### Accessible authentication",
        failures,
    )

    if failures:
        print(f"\n{len(failures)} case(s) failed", file=sys.stderr)
        return 1
    print("\nall interface-suite gate regression cases pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
