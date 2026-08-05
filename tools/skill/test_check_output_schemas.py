#!/usr/bin/env python3
"""Regression tests for the strict-structured-output schema gate.

Every case here is a schema shape that OpenAI strict mode rejects at request
time with a 400, after the model has already been dispatched. The gate exists to
turn that into a CI error instead.

Two of these cases are shapes the gate itself once let through:

* a root object with no `properties` -- skipped entirely before it was walked
* a nullable object, `{"type": ["object", "null"]}` -- the type check compared
  against the string "object" and never matched an array

The gate was written to catch exactly this class and shipped passing on it. That
is why these are committed rather than run by hand: a gate with no test is an
assertion, and the whole point of this one is that assertions are what fail.

Run: python3 tools/skill/test_check_output_schemas.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

GATE = Path(__file__).resolve().parent / "check_output_schemas.py"


def run_gate(root: Path) -> tuple[int, str]:
    """Run the gate against a fixture root.

    Args:
        root: Directory the gate scans for schemas.

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


def write_schema(root: Path, schema: object) -> None:
    """Place one schema where the gate's discovery glob will find it.

    The gate only considers files named `*schema*.json` inside a `references`
    or `schemas` directory under `skills/` or `plugins/`, so the path matters
    as much as the content.

    Args:
        root: Fixture root to write beneath.
        schema: JSON-serializable schema body.

    Returns:
        None.

    Raises:
        OSError: If the fixture directory or file cannot be written.
        TypeError: If `schema` is not JSON-serializable.
    """
    path = root / "skills" / "probe" / "references" / "probe-schema.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(schema, indent=2), encoding="utf-8")


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


REJECT_CASES: list[tuple[str, str, object]] = [
    (
        "nullable object missing additionalProperties",
        "$.meta: objects must set \"additionalProperties\": false",
        {
            "type": "object",
            "additionalProperties": False,
            "required": ["meta"],
            "properties": {
                "meta": {
                    "type": ["object", "null"],
                    "required": ["x"],
                    "properties": {"x": {"type": "string"}},
                }
            },
        },
    ),
    (
        "implicit object (properties, no declared type)",
        "$.m: objects must set \"additionalProperties\": false",
        {
            "type": "object",
            "additionalProperties": False,
            "required": ["m"],
            "properties": {
                "m": {
                    "required": ["x"],
                    "properties": {"x": {"type": "string"}},
                }
            },
        },
    ),
    (
        "anyOf branch missing additionalProperties",
        "$: objects must set \"additionalProperties\": false",
        {
            "anyOf": [
                {
                    "type": "object",
                    "required": ["a"],
                    "properties": {"a": {"type": "string"}},
                    "additionalProperties": False,
                },
                {
                    "type": "object",
                    "required": ["b"],
                    "properties": {"b": {"type": "string"}},
                },
            ]
        },
    ),
    (
        "bare root object with no properties",
        "$: objects must set \"additionalProperties\": false",
        {"type": "object"},
    ),
    (
        # The reviewer's exact repro. Distinct from the nullable case above:
        # that one carries `properties`, so it is reachable through the
        # properties branch even when the type check is broken. With no
        # `properties` anywhere, a broken type check makes the whole file
        # invisible and the gate reports "0 schema(s) checked" -- success.
        "bare nullable object, no properties anywhere",
        "$: objects must set \"additionalProperties\": false",
        {"type": ["object", "null"]},
    ),
    (
        "nullable object nested under a non-object root",
        "$: objects must set \"additionalProperties\": false",
        {
            "anyOf": [
                {"type": "string"},
                {"type": ["object", "null"]},
            ]
        },
    ),
    (
        "optional-by-omission (property absent from required)",
        "$: 'required' must include every key in 'properties'; missing ['b']",
        {
            "type": "object",
            "additionalProperties": False,
            "required": ["a"],
            "properties": {
                "a": {"type": "string"},
                "b": {"type": "string"},
            },
        },
    ),
    (
        "required names a key absent from properties",
        "$: 'required' names keys absent from 'properties': ['ghost']",
        {
            "type": "object",
            "additionalProperties": False,
            "required": ["a", "ghost"],
            "properties": {"a": {"type": "string"}},
        },
    ),
    (
        "violation nested inside array items",
        "$.rows.items: objects must set \"additionalProperties\": false",
        {
            "type": "object",
            "additionalProperties": False,
            "required": ["rows"],
            "properties": {
                "rows": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "required": ["a"],
                        "properties": {"a": {"type": "string"}},
                    },
                }
            },
        },
    ),
]

VALID_SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "required": ["rows", "meta"],
    "properties": {
        "rows": {
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": False,
                "required": ["a"],
                "properties": {"a": {"type": "string"}},
            },
        },
        "meta": {
            "type": ["object", "null"],
            "additionalProperties": False,
            "required": ["x"],
            "properties": {"x": {"type": "string"}},
        },
    },
}


def main() -> int:
    """Run every regression case.

    Returns:
        0 when every case behaves as specified, 1 otherwise.

    Raises:
        OSError: If a fixture cannot be created or the gate cannot be run.
    """
    failures: list[str] = []

    print("rejects strict-mode violations:")
    for label, diagnostic, schema in REJECT_CASES:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_schema(root, schema)
            code, out = run_gate(root)
            # Non-zero is necessary but not sufficient: a gate that crashes
            # also exits non-zero, and an unrelated violation would satisfy a
            # bare "INVALID" check. Require this case's own diagnosis, path
            # included, so a case cannot pass for the wrong reason.
            if code != 1:
                expect(False, f"{label} [expected exit 1, got {code}]",
                       failures)
            elif diagnostic not in out:
                expect(False, f"{label} [wrong diagnosis; wanted "
                       f"{diagnostic!r}]", failures)
            else:
                expect(True, label, failures)

    print("\naccepts a valid schema:")
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        write_schema(root, VALID_SCHEMA)
        code, out = run_gate(root)
        expect(code == 0, "strict-mode-valid schema passes", failures)
        # Guards the original bug directly: the gate used to report success
        # having checked nothing at all.
        expect(
            "1 schema(s) checked" in out,
            "reports the schema as actually checked, not skipped",
            failures,
        )

    if failures:
        print(f"\n{len(failures)} case(s) failed", file=sys.stderr)
        return 1
    print("\nall output-schema gate regression cases pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
