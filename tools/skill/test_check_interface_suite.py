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
    """Copy the real suite files into a throwaway root.

    Cases mutate the real repository's files rather than synthetic stand-ins,
    so a case cannot drift from the contract it claims to guard.

    Args:
        tmp: Temporary directory to build the fixture inside.

    Returns:
        The fixture root to hand the gate.

    Raises:
        OSError: If a source file is missing or the copy fails.
    """
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
    """Run the gate against a fixture root.

    Args:
        root: Fixture repository root the gate should inspect.

    Returns:
        The gate's exit code and its combined stdout and stderr. Both streams
        are merged because the gate prints diagnoses to stderr and its success
        line to stdout, and cases assert against either.

    Raises:
        OSError: If the interpreter cannot be executed.
    """
    proc = subprocess.run(
        [sys.executable, str(GATE), str(root)],
        capture_output=True,
        text=True,
    )
    return proc.returncode, proc.stdout + proc.stderr


def patch(root: Path, rel: str, old: str, new: str, every: bool) -> bool:
    """Replace `old` with `new` in a fixture file.

    Args:
        root: Fixture root containing the file.
        rel: Repository-relative path to mutate.
        old: Anchor text to replace.
        new: Replacement text.
        every: Replace all occurrences rather than only the first. This matters
            for vocabulary checks: the router states most verdicts twice, so
            replacing one leaves the other satisfying the gate and the case
            passes for the wrong reason.

    Returns:
        True when the anchor was found and replaced, False when it is absent
        so the caller can fail the case loudly instead of skipping it.

    Raises:
        OSError: If the fixture file cannot be read or written.
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
    """Record a labelled assertion result.

    Args:
        cond: Whether the case passed.
        label: Human-readable case name for the result line.
        failures: Accumulates failed case labels in place.

    Returns:
        None. Results are printed and recorded in ``failures``.
    """
    print(f"  {'ok' if cond else 'FAIL'}  {label}")
    if not cond:
        failures.append(label)


def case(
    label: str,
    rel: str,
    old: str,
    new: str,
    diagnostic: str,
    failures: list[str],
    every: bool = False,
) -> None:
    """Inject one disagreement and require the gate's specific diagnosis.

    Args:
        label: Human-readable case name for the result line.
        rel: Repository-relative file to mutate.
        old: Anchor text to replace.
        new: Replacement text carrying the injected defect.
        diagnostic: Fragment the gate must emit. Asserting only a non-zero exit
            and the generic marker would let an unrelated validation failure
            satisfy the case -- a test passing for the wrong reason, which is
            the defect class these tests exist to prevent.
        failures: Accumulates failed case labels in place.
        every: Replace all occurrences rather than the first.

    Returns:
        None. Results are printed and recorded in ``failures``.

    Raises:
        OSError: If a fixture file cannot be read or written.
    """
    with tempfile.TemporaryDirectory() as tmp:
        root = make_root(tmp)
        if not patch(root, rel, old, new, every):
            # The anchor text moved. Fail loudly: a silently skipped case is
            # exactly the failure mode these tests exist to prevent.
            expect(False, f"{label} [ANCHOR NOT FOUND in {rel}]", failures)
            return
        code, out = run_gate(root)
        if code != 1:
            expect(False, f"{label} [expected exit 1, got {code}]", failures)
            return
        if diagnostic not in out:
            expect(
                False,
                f"{label} [wrong diagnosis; wanted {diagnostic!r}]",
                failures,
            )
            return
        expect(True, label, failures)


def main() -> int:
    """Run every regression case.

    Returns:
        0 when every case behaves as specified, 1 otherwise.

    Raises:
        OSError: If a fixture cannot be created or the gate cannot be run.
    """
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

    # `build` specifically: the router contains a *second* table whose rows
    # also begin with "| `build` |" (Principle 6's tool-access matrix). The
    # gate must read the cap from the mode table and ignore that one. Scanning
    # every table is the bug that let real drift pass.
    case(
        "mode cap drift (build)",
        ROUTER,
        "15, scoped to the diff |",
        "99, scoped to the diff |",
        "cap for `build` is 15, router says 99",
        failures,
    )
    # Numeric drift on a non-build mode. The original bug was a fallback that
    # could match a *later* mode's number, so a changed cap still found a match
    # and the gate reported success. Renaming a mode (below) only proves the
    # gate notices a missing row -- it does not exercise cap comparison at all.
    case(
        "mode cap drift (core, numeric)",
        ROUTER,
        "reported `Detected only` | 8 |",
        "reported `Detected only` | 88 |",
        "cap for `core` is 8, router says 88",
        failures,
    )
    case(
        "mode cap drift (quick, numeric)",
        ROUTER,
        "`HIGH` and `MEDIUM` | 5 |",
        "`HIGH` and `MEDIUM` | 55 |",
        "cap for `quick` is 5, router says 55",
        failures,
    )
    case(
        "mode row missing entirely",
        ROUTER,
        "| `core` |",
        "| `core-renamed` |",
        "mode table has no row for `core`",
        failures,
    )

    # Coverage honesty: all six domains must be enumerated in the schema.
    case(
        "domain dropped from verified-schema coverage",
        VERIFIED_SCHEMA,
        '"better-ui"',
        '"better-ui-typo"',
        "coverage must require all six domain keys",
        failures,
    )
    case(
        "domain enum drift in candidate-schema",
        CANDIDATE_SCHEMA,
        '"better-colors"',
        '"better-colours"',
        "domain enum",
        failures,
    )

    # Every domain skill must declare the same three-level ladder, so a lane's
    # severity means the same thing whichever skill produced it.
    case(
        "domain skill renames a severity level",
        "skills/better-ui/SKILL.md",
        "- `HIGH`",
        "- `CRITICAL`",
        "severity ladder is ['CRITICAL', 'MEDIUM', 'LOW']",
        failures,
    )
    case(
        "domain skill loses its ## Severity section",
        "skills/better-colors/SKILL.md",
        "## Severity",
        "## Severity notes (prose)",
        "no '## Severity' section",
        failures,
    )

    # Verdict vocabulary is checked in both directions against the schema enum.
    case(
        "verdict wording drifts from the schema enum",
        ROUTER,
        "`Needs changes`",
        "`Needs-changes`",
        "schema verdict(s) ['Needs changes'] never mentioned",
        failures,
        every=True,
    )
    case(
        "consolidator drops a required verdict",
        CONSOLIDATOR,
        "`Approve`",
        "`Ship it`",
        "schema verdict(s) ['Approve'] never mentioned",
        failures,
        every=True,
    )
    # The opposite direction, and the only way to reach it. `VERDICT_RE` matches
    # exactly the five known literals, so an invented verdict is never *found*
    # in the prose and can never populate `unknown`. That branch fires only when
    # the schema stops defining a verdict the prose still uses -- so the schema
    # is what has to be mutated here, not the prose.
    case(
        "schema drops a verdict the router still uses",
        VERIFIED_SCHEMA,
        '"No verdict"',
        '"Verdict withheld"',
        "verdict(s) ['No verdict'] absent from the schema enum",
        failures,
    )

    # WCAG conformance coverage.
    case(
        "WCAG criterion removed (whole guideline regression)",
        WCAG_SKILL,
        "### 2.5.8 Target Size (Minimum) (Level AA) - WCAG 2.2",
        "### Target size notes",
        "missing WCAG 2.2 A/AA criteria ['2.5.8']",
        failures,
    )
    case(
        "WCAG 2.2 addition removed (3.3.8)",
        WCAG_SKILL,
        "### 3.3.8 Accessible Authentication (Minimum) (Level AA) - WCAG 2.2",
        "### Accessible authentication",
        "missing WCAG 2.2 A/AA criteria ['3.3.8']",
        failures,
    )

    if failures:
        print(f"\n{len(failures)} case(s) failed", file=sys.stderr)
        return 1
    print("\nall interface-suite gate regression cases pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
