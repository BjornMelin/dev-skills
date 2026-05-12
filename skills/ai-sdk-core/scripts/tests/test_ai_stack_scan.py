#!/usr/bin/env python3
"""Regression tests for the offline AI stack scanner."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "ai_stack_scan.py"


def run_scan(root: Path, *args: str) -> dict:
    """Run the scanner against a temporary repository.

    Args:
        root: Repository root to scan.
        *args: Extra command-line arguments passed to the scanner.

    Returns:
        Parsed scanner JSON output.

    Raises:
        subprocess.CalledProcessError: Propagated when the scanner exits
            unsuccessfully.
    """
    result = subprocess.run(  # noqa: S603 - test invokes this repo's scanner.
        [sys.executable, str(SCRIPT), "--root", str(root), *args],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


class AiStackScanTests(unittest.TestCase):
    """Focused scanner contract regression tests."""

    def test_default_output_redacts_root_and_source_evidence(self) -> None:
        """Default output redacts scan roots and omits source evidence."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "package.json").write_text(
                '{"dependencies":{"zod":"^3.22.0"}}', encoding="utf-8"
            )

            data = run_scan(root, "--family", "zod-v4")

        self.assertEqual(data["schema"], "ai_stack_scan.v1")
        self.assertEqual(data["root"], "<scan-root>")
        self.assertFalse(data["privacy"]["absolute_root_included"])
        self.assertFalse(data["privacy"]["evidence_included"])
        self.assertTrue(
            all("evidence" not in signal for signal in data["signals"])
        )

    def test_absolute_root_is_opt_in(self) -> None:
        """Absolute root paths are emitted only when explicitly requested."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            (root / "package.json").write_text("{}", encoding="utf-8")

            data = run_scan(root, "--include-absolute-root")

        self.assertEqual(data["root"], str(root))
        self.assertTrue(data["privacy"]["absolute_root_included"])

    def test_symlinked_package_json_outside_root_is_ignored(self) -> None:
        """Symlinked manifests outside the root are ignored."""
        with tempfile.TemporaryDirectory() as tmp:
            work = Path(tmp)
            root = work / "repo"
            outside = work / "outside"
            root.mkdir()
            outside.mkdir()
            (outside / "package.json").write_text(
                '{"dependencies":{"zod":"^3.22.0"}}', encoding="utf-8"
            )
            (root / "package.json").symlink_to(outside / "package.json")

            data = run_scan(root, "--family", "zod-v4")

        self.assertEqual(data["package_manifests"], [])
        self.assertEqual(data["signals"], [])

    def test_oversized_package_json_is_ignored(self) -> None:
        """Manifest parsing respects the per-file byte cap."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            manifest = (
                '{"dependencies":{"zod":"^3.22.0"},"padding":"'
                + ("x" * 128)
                + '"}'
            )
            (root / "package.json").write_text(manifest, encoding="utf-8")

            data = run_scan(root, "--family", "zod-v4", "--max-bytes", "64")

        self.assertEqual(data["package_manifests"], [])
        self.assertEqual(data["packages"], [])
        self.assertEqual(data["signals"], [])

    def test_capped_traversal_is_sorted(self) -> None:
        """File caps preserve deterministic sorted traversal order."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "b").mkdir()
            (root / "a").mkdir()
            source = (
                "import { useChat } from '@ai-sdk/react';\nmessage.content\n"
            )
            (root / "b" / "component.ts").write_text(source, encoding="utf-8")
            (root / "a" / "component.ts").write_text(source, encoding="utf-8")

            data = run_scan(root, "--family", "ai-sdk-ui", "--max-files", "1")

        message_signals = [
            signal
            for signal in data["signals"]
            if signal["id"] == "ai_sdk_ui_message_content"
        ]
        self.assertEqual(message_signals[0]["path"], "a/component.ts")

    def test_repo_level_signals_use_stable_sentinel_path(self) -> None:
        """Repo-level signals use a stable sentinel path."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "component.ts").write_text(
                "import { useChat } from '@ai-sdk/react';\n",
                encoding="utf-8",
            )

            data = run_scan(root, "--family", "ai-sdk-ui")

        missing_package = [
            signal
            for signal in data["signals"]
            if signal["id"] == "ai_sdk_ui_missing_react_package"
        ]
        self.assertEqual(missing_package[0]["path"], "<repo-root>")

    def test_supabase_service_role_public_env_is_redacted(self) -> None:
        """Public service-role environment findings redact secret values."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / ".env.local").write_text(
                "NEXT_PUBLIC_SUPABASE_SERVICE_ROLE_KEY=secret-value\n",
                encoding="utf-8",
            )

            data = run_scan(
                root, "--family", "supabase-ts", "--include-evidence"
            )

        self.assertEqual(
            data["signals"][0]["id"], "supabase_service_role_public_env"
        )
        self.assertNotIn("secret-value", json.dumps(data))


if __name__ == "__main__":
    unittest.main()
