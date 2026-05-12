#!/usr/bin/env python3
"""Regression tests for Streamlit ui_audit.v1 output."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]

SCRIPT = Path(__file__).resolve().parents[1] / "audit_streamlit_project.py"


def run_audit(root: Path) -> dict:
    """Run the Streamlit audit in ui_audit.v1 mode.

    Args:
        root: Temporary project root to scan.

    Returns:
        Decoded audit JSON.

    Raises:
        subprocess.CalledProcessError: If the audit process fails.
    """
    output = subprocess.check_output(
        [
            sys.executable,
            str(SCRIPT),
            "--root",
            str(root),
            "--format",
            "ui-audit-json",
            "--no-check-latest",
        ],
        text=True,
    )
    return json.loads(output)


class StreamlitUiAuditTests(unittest.TestCase):
    """Streamlit ui_audit.v1 contract tests."""

    def test_deprecated_api_maps_to_ui_audit_error(self) -> None:
        """Deprecated Streamlit calls become redacted error findings."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "app.py").write_text(
                "import streamlit as st\nst.cache(lambda: 1)()\n",
                encoding="utf-8",
            )

            data = run_audit(root)

        self.assertEqual(data["schema"], "ui_audit.v1")
        self.assertEqual(data["target"]["root"], "<scan-root>")
        self.assertEqual(data["summary"]["status"], "fail")
        self.assertEqual(data["findings"][0]["id"], "streamlit.deprecated_api")
        self.assertEqual(data["findings"][0]["severity"], "error")
        self.assertEqual(data["findings"][0]["locations"][0]["path"], "app.py")
        self.assertNotIn(str(root), json.dumps(data))

    def test_clean_project_passes_with_observations(self) -> None:
        """Clean Streamlit projects still emit inventory observations."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "app.py").write_text(
                "import streamlit as st\nst.write('ok')\n",
                encoding="utf-8",
            )

            data = run_audit(root)

        self.assertEqual(data["summary"]["status"], "pass")
        self.assertEqual(data["findings"], [])
        self.assertGreaterEqual(len(data["observations"]), 1)

    def test_beta_api_uses_file_line_location(self) -> None:
        """Legacy beta APIs point to source locations, not API-name paths."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "app.py").write_text(
                "import streamlit as st\nst.beta_columns(2)\n",
                encoding="utf-8",
            )

            data = run_audit(root)

        finding = data["findings"][0]
        self.assertEqual(finding["id"], "streamlit.deprecated_beta_api")
        self.assertEqual(finding["locations"][0]["path"], "app.py")
        self.assertEqual(finding["locations"][0]["line"], 2)
        self.assertNotEqual(finding["locations"][0]["path"], "st.beta_columns")

    def test_windows_locations_under_scan_root_are_relative(self) -> None:
        """Windows paths under a Windows scan root stay root-relative."""
        import importlib.util

        spec = importlib.util.spec_from_file_location(
            "audit_streamlit_project",
            SCRIPT,
        )
        self.assertIsNotNone(spec)
        self.assertIsNotNone(spec.loader)
        module = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = module
        spec.loader.exec_module(module)

        location = module._ui_location(
            Path(r"C:\repo\app"),
            r"C:\repo\app\pages\Home.py:7",
        )

        self.assertEqual(location, {"path": "pages/Home.py", "line": 7})

    @unittest.skipIf(tomllib is None, "pyproject TOML parsing requires tomllib")
    def test_dependency_specs_redact_direct_urls(self) -> None:
        """UI audit observations do not expose direct dependency URL specs."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "pyproject.toml").write_text(
                "[project]\n"
                "dependencies = [\n"
                '  "streamlit @ https://token@example.com/streamlit.whl",\n'
                "]\n",
                encoding="utf-8",
            )
            (root / "app.py").write_text(
                "import streamlit as st\nst.write('ok')\n",
                encoding="utf-8",
            )

            data = run_audit(root)

        payload_text = json.dumps(data)
        self.assertNotIn("token@example.com", payload_text)
        dependency_observation = next(
            obs
            for obs in data["observations"]
            if obs["id"] == "streamlit.dependency_specs"
        )
        self.assertEqual(
            dependency_observation["data"][0]["spec"],
            "streamlit <redacted-spec>",
        )


if __name__ == "__main__":
    unittest.main()
