#!/usr/bin/env python3
"""Tests for the design-motion analysis and scaffold scripts."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPT_DIR))

import audit_motion_system  # noqa: E402
import detect_motion_stack  # noqa: E402
import scaffold_motion_tokens  # noqa: E402
from audit_motion_system import audit_file  # noqa: E402
from detect_motion_stack import scan  # noqa: E402


class _argv:
    """Context manager to patch sys.argv around a main() call."""

    def __init__(self, argv):
        self._argv = argv

    def __enter__(self):
        self._saved = sys.argv
        sys.argv = self._argv
        return self

    def __exit__(self, *exc):
        sys.argv = self._saved


def _write(root: Path, rel: str, text: str) -> Path:
    """Write a fixture file and return its path."""
    p = root / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text, encoding="utf-8")
    return p


class NewBehaviorTests(unittest.TestCase):
    def test_r3f_setstate_parenthesized_callback(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = _write(
                root,
                "Scene.tsx",
                "useFrame((state, delta) => { setX(1); })",
            )
            result = audit_file(root, path)
        self.assertIn(
            "r3f-setstate-in-useframe",
            {f["type"] for f in result["findings"]},
        )

    def test_scan_finds_texture_assets(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "diffuse.png").write_bytes(b"")
            result = scan(root, 100)
        self.assertTrue(result["assets"]["textures"])

    def test_scan_detects_gsap_package_and_files(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(
                root,
                "package.json",
                json.dumps({"dependencies": {"gsap": "^3.0.0"}}),
            )
            _write(root, "anim.ts", "gsap.to('.box', { opacity: 1 })")
            result = scan(root, 100)
        self.assertTrue(result["packages"]["gsap"])
        self.assertEqual(result["matches"]["gsap_files"], ["anim.ts"])

    def test_scaffold_auto_only_writes_native_for_native_deps(self):
        script = SCRIPT_DIR / "scaffold_motion_tokens.py"
        with tempfile.TemporaryDirectory() as tmp:
            web = Path(tmp) / "web"
            native = Path(tmp) / "native"
            web.mkdir()
            native.mkdir()
            _write(
                web,
                "package.json",
                json.dumps({"dependencies": {"react": "^19.0.0"}}),
            )
            _write(
                native,
                "package.json",
                json.dumps({"dependencies": {"react-native": "^0.80.0"}}),
            )
            for r in (web, native):
                subprocess.run(
                    [
                        sys.executable,
                        str(script),
                        str(r),
                        "--stack",
                        "auto",
                        "--write",
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                )
            self.assertFalse(
                (web / "src/design-system/motion/reanimated-motion.ts").exists()
            )
            self.assertTrue(
                (
                    native
                    / "src/design-system/motion/reanimated-motion.ts"
                ).exists()
            )

    def test_scan_prunes_pods_dirs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(
                root,
                "ios/Pods/Scene.ts",
                "import { Canvas } from '@react-three/fiber';",
            )
            result = scan(root, 100)
        self.assertEqual(result["matches"]["r3f_files"], [])

    def test_reanimated_setstate_in_callback_flagged(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = _write(
                root,
                "Sheet.tsx",
                "import 'react-native-reanimated';\n"
                "const g = Gesture.Pan().onUpdate((e) => { setOpen(true); });",
            )
            result = audit_file(root, path)
        self.assertIn(
            "reanimated-setstate-in-callback",
            {f["type"] for f in result["findings"]},
        )


class DetectMotionStackTests(unittest.TestCase):
    def test_detects_r3f_and_package(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(
                root,
                "package.json",
                json.dumps({"dependencies": {"three": "^0.185.0"}}),
            )
            _write(
                root,
                "src/Scene.tsx",
                "import { Canvas } from '@react-three/fiber';\n"
                "<Canvas><mesh /></Canvas>\n"
                "useFrame(() => {});",
            )
            data = scan(root, 100)
            self.assertTrue(data["packages"]["three"])
            self.assertTrue(
                any("Scene.tsx" in f for f in data["matches"]["r3f_files"])
            )

    def test_token_bucket_ignores_json_and_bare_words(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(
                root,
                "meta.json",
                json.dumps({"motion": "x", "duration": "y", "easing": "z"}),
            )
            _write(root, "ok.ts", "const s = useReducedMotion();")
            data = scan(root, 100)
            self.assertFalse(
                any(
                    "meta.json" in f
                    for f in data["matches"]["motion_token_files"]
                )
            )
            self.assertTrue(
                any("ok.ts" in f for f in data["matches"]["motion_token_files"])
            )

    def test_ignores_build_dirs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(root, "target/gen.tsx", "<Canvas />")
            _write(root, "src/real.tsx", "<Canvas />")
            hits = scan(root, 100)["matches"]["r3f_files"]
            self.assertTrue(any("real.tsx" in f for f in hits))
            self.assertFalse(any("target" in f for f in hits))

    def test_main_rejects_nonexistent_dir(self):
        argv = [
            "detect",
            str(Path(tempfile.gettempdir()) / "no-such-dir-xyz-123"),
        ]
        with self.assertRaises(SystemExit) as ctx, _argv(argv):
            detect_motion_stack.main()
        self.assertNotEqual(ctx.exception.code, 0)


class AuditMotionSystemTests(unittest.TestCase):
    def test_flags_hardcoded_durations(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            p = _write(
                root,
                "a.ts",
                "const a=200; x='200ms'; y='300ms'; z='1s';",
            )
            item = audit_file(root, p)
            self.assertIsNotNone(item)
            self.assertIn(
                "hardcoded-durations",
                {f["type"] for f in item["findings"]},
            )

    def test_ignores_build_dirs(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            _write(root, "target/x.ts", "'200ms';'300ms';'400ms';")
            files = list(audit_motion_system.iter_files(root, 100))
            self.assertFalse(any("target" in str(f) for f in files))

    def test_main_rejects_nonexistent_dir(self):
        argv = [
            "audit",
            str(Path(tempfile.gettempdir()) / "no-such-dir-abc-987"),
        ]
        with self.assertRaises(SystemExit) as ctx, _argv(argv):
            audit_motion_system.main()
        self.assertNotEqual(ctx.exception.code, 0)


class ScaffoldMotionTokensTests(unittest.TestCase):
    def test_dry_run_writes_nothing(self):
        with tempfile.TemporaryDirectory() as d:
            with _argv(["scaffold", d]):
                scaffold_motion_tokens.main()
            self.assertFalse((Path(d) / "src").exists())

    def test_write_creates_files(self):
        with tempfile.TemporaryDirectory() as d:
            with _argv(["scaffold", d, "--write"]):
                scaffold_motion_tokens.main()
            self.assertTrue(
                (Path(d) / "src/design-system/motion/motion.ts").exists()
            )

    def test_dir_guard_rejects_escape(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(SystemExit) as ctx, _argv(
                ["scaffold", d, "--dir", "../../etc/evil"]
            ):
                scaffold_motion_tokens.main()
            self.assertNotEqual(ctx.exception.code, 0)


if __name__ == "__main__":
    unittest.main()
