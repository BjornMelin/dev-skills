#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import unittest


SCRIPT_PATH = Path(__file__).resolve().parents[1] / "scan_hotspots.py"
SPEC = importlib.util.spec_from_file_location("scan_hotspots", SCRIPT_PATH)
assert SPEC is not None
scan_hotspots = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
sys.modules[SPEC.name] = scan_hotspots
SPEC.loader.exec_module(scan_hotspots)


class ScanTextLoopStackTests(unittest.TestCase):
    def scan(self, text: str):
        root = Path("/repo")
        return scan_hotspots.scan_text(root / "example.ts", root, text)

    def test_top_level_code_after_callback_is_not_inside_loop(self) -> None:
        findings = self.scan(
            """
items.map((item) => {
  return item.id;
});

fetch("/api/users");
"""
        )

        self.assertNotIn("io-or-query-in-loop", {finding.kind for finding in findings})

    def test_indented_callback_body_is_still_inside_loop(self) -> None:
        findings = self.scan(
            """
items.map((item) => {
  fetch(`/api/users/${item.id}`);
});
"""
        )

        self.assertIn("io-or-query-in-loop", {finding.kind for finding in findings})


if __name__ == "__main__":
    unittest.main()
