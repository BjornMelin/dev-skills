#!/usr/bin/env python3
"""Regression tests for the Dash ui_audit.v1 adapter."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "dash_ui_audit_adapter.py"


def run_adapter(payload: dict) -> dict:
    """Run the Dash adapter against a temporary preflight payload.

    Args:
        payload: JSON-serializable preflight object.

    Returns:
        Decoded adapter output.

    Raises:
        subprocess.CalledProcessError: If the adapter process fails.
    """
    with tempfile.TemporaryDirectory() as tmp:
        input_path = Path(tmp) / "preflight.json"
        input_path.write_text(json.dumps(payload), encoding="utf-8")
        output = subprocess.check_output(
            [sys.executable, str(SCRIPT), "--input", str(input_path)],
            text=True,
        )
    return json.loads(output)


class DashUiAuditAdapterTests(unittest.TestCase):
    """Dash adapter contract tests."""

    def test_callback_map_rows_become_observations(self) -> None:
        """Callback preflight rows are preserved as observations."""
        data = run_adapter(
            {
                "repo_root": "/tmp/example",
                "callbacks": [
                    {
                        "file": "app.py",
                        "callback_decorators": 1,
                        "output_calls": 1,
                        "input_calls": 1,
                        "state_calls": 0,
                    }
                ],
            }
        )
        self.assertEqual(data["schema"], "ui_audit.v1")
        self.assertEqual(data["target"]["root"], "<scan-root>")
        self.assertEqual(data["summary"]["status"], "pass")
        self.assertEqual(data["findings"], [])
        self.assertEqual(
            data["observations"][0]["locations"][0]["path"], "app.py"
        )

    def test_callback_without_output_becomes_warning(self) -> None:
        """Suspicious callback rows become actionable warning findings."""
        data = run_adapter(
            {
                "repo_root": "/tmp/example",
                "callbacks": [
                    {
                        "file": "callbacks.py",
                        "callback_decorators": 2,
                        "output_calls": 0,
                        "input_calls": 2,
                        "state_calls": 0,
                    }
                ],
            }
        )
        self.assertEqual(data["summary"]["status"], "warning")
        self.assertEqual(data["summary"]["counts"]["warning"], 1)
        self.assertEqual(
            data["findings"][0]["id"],
            "dash.callback_without_output",
        )

    def test_absolute_paths_are_redacted_against_repo_root(self) -> None:
        """Absolute callback paths are emitted as repo-relative locations."""
        data = run_adapter(
            {
                "repo_root": "/tmp/example",
                "callbacks": [
                    {
                        "file": "/tmp/example/pkg/callbacks.py",
                        "callback_decorators": 1,
                        "output_calls": 0,
                        "input_calls": 1,
                        "state_calls": 0,
                    }
                ],
            }
        )
        payload_text = json.dumps(data)
        self.assertNotIn("/tmp/example", payload_text)
        self.assertEqual(
            data["observations"][0]["locations"][0]["path"],
            "pkg/callbacks.py",
        )
        self.assertEqual(
            data["findings"][0]["locations"][0]["path"],
            "pkg/callbacks.py",
        )

    def test_malformed_callback_payload_warns(self) -> None:
        """Malformed callback maps do not silently produce a pass result."""
        data = run_adapter({"repo_root": "/tmp/example", "callbacks": {}})
        self.assertEqual(data["summary"]["status"], "warning")
        self.assertEqual(
            data["findings"][0]["id"],
            "dash.invalid_preflight_payload",
        )

    def test_malformed_callback_rows_warn(self) -> None:
        """Non-object callback rows do not silently produce a pass result."""
        data = run_adapter(
            {"repo_root": "/tmp/example", "callbacks": ["not-a-row"]}
        )
        self.assertEqual(data["summary"]["status"], "warning")
        self.assertEqual(
            data["findings"][0]["id"],
            "dash.invalid_preflight_payload",
        )

    def test_windows_paths_are_repo_relative_when_possible(self) -> None:
        """Windows callback paths are root-relative before redaction fallback."""
        data = run_adapter(
            {
                "repo_root": r"C:\repo\app",
                "callbacks": [
                    {
                        "file": r"C:\repo\app\pkg\callbacks.py",
                        "callback_decorators": 1,
                        "output_calls": 0,
                        "input_calls": 1,
                        "state_calls": 0,
                    }
                ],
            }
        )
        payload_text = json.dumps(data)
        self.assertNotIn(r"C:\repo\app", payload_text)
        self.assertEqual(
            data["observations"][0]["locations"][0]["path"],
            "pkg/callbacks.py",
        )

    def test_outside_root_absolute_paths_redact_to_basename(self) -> None:
        """Outside-root absolute paths fall back to basename redaction."""
        data = run_adapter(
            {
                "repo_root": "/tmp/example",
                "callbacks": [
                    {
                        "file": "/home/alice/private_callbacks.py",
                        "callback_decorators": 1,
                        "output_calls": 0,
                        "input_calls": 1,
                        "state_calls": 0,
                    }
                ],
            }
        )
        payload_text = json.dumps(data)
        self.assertNotIn("/home/alice", payload_text)
        self.assertEqual(
            data["observations"][0]["locations"][0]["path"],
            "private_callbacks.py",
        )

    def test_outside_root_windows_paths_redact_to_basename(self) -> None:
        """Outside-root Windows paths fall back to basename redaction."""
        data = run_adapter(
            {
                "repo_root": r"C:\repo\app",
                "callbacks": [
                    {
                        "file": r"C:\Users\alice\private_callbacks.py",
                        "callback_decorators": 1,
                        "output_calls": 0,
                        "input_calls": 1,
                        "state_calls": 0,
                    }
                ],
            }
        )
        payload_text = json.dumps(data)
        self.assertNotIn(r"C:\Users\alice", payload_text)
        self.assertEqual(
            data["observations"][0]["locations"][0]["path"],
            "private_callbacks.py",
        )

    def test_malformed_count_fields_do_not_crash(self) -> None:
        """Malformed callback counts are coerced to zero deterministically."""
        data = run_adapter(
            {
                "repo_root": "/tmp/example",
                "callbacks": [
                    {
                        "file": "callbacks.py",
                        "callback_decorators": "many",
                        "output_calls": None,
                        "input_calls": {},
                        "state_calls": [],
                    }
                ],
            }
        )
        self.assertEqual(data["summary"]["status"], "pass")
        self.assertEqual(
            data["observations"][0]["data"],
            {
                "callback_decorators": 0,
                "output_calls": 0,
                "input_calls": 0,
                "state_calls": 0,
            },
        )


if __name__ == "__main__":
    unittest.main()
