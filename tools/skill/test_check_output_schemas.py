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
    """Run the gate against a fixture root; return (exit code, stderr)."""
    proc = subprocess.run(
        [sys.executable, str(GATE), str(root)],
        capture_output=True,
        text=True,
    )
    return proc.returncode, proc.stdout + proc.stderr


def write_schema(root: Path, schema: object) -> None:
    """Place one schema where the gate's discovery glob will find it."""
    path = root / "skills" / "probe" / "references" / "probe-schema.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(schema, indent=2), encoding="utf-8")


def expect(cond: bool, label: str, failures: list[str]) -> None:
    """Record a labelled assertion result."""
    print(f"  {'ok' if cond else 'FAIL'}  {label}")
    if not cond:
        failures.append(label)


REJECT_CASES: list[tuple[str, object]] = [
    (
        "nullable object missing additionalProperties",
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
        {"type": "object"},
    ),
    (
        # The reviewer's exact repro. Distinct from the nullable case above:
        # that one carries `properties`, so it is reachable through the
        # properties branch even when the type check is broken. With no
        # `properties` anywhere, a broken type check makes the whole file
        # invisible and the gate reports "0 schema(s) checked" -- success.
        "bare nullable object, no properties anywhere",
        {"type": ["object", "null"]},
    ),
    (
        "nullable object nested under a non-object root",
        {
            "anyOf": [
                {"type": "string"},
                {"type": ["object", "null"]},
            ]
        },
    ),
    (
        "optional-by-omission (property absent from required)",
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
        {
            "type": "object",
            "additionalProperties": False,
            "required": ["a", "ghost"],
            "properties": {"a": {"type": "string"}},
        },
    ),
    (
        "violation nested inside array items",
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
    """Run every regression case; return a process exit code."""
    failures: list[str] = []

    print("rejects strict-mode violations:")
    for label, schema in REJECT_CASES:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            write_schema(root, schema)
            code, out = run_gate(root)
            # Non-zero is necessary but not sufficient: a gate that skips the
            # file reports success, and one that crashes also exits non-zero.
            # Require the specific diagnosis.
            expect(
                code == 1 and "INVALID" in out,
                label,
                failures,
            )

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
