#!/usr/bin/env python3
"""Audit source files for common motion-system risks."""

import argparse
import json
import re
from pathlib import Path

SCAN_EXTS = {".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".css", ".scss"}
IGNORE = {
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    ".expo",
    "coverage",
    "target",
    "out",
    ".turbo",
    ".cache",
    "vendor",
    ".venv",
    "__pycache__",
}
DURATION_RE = re.compile(r"(?<![\w-])(?:\d+ms|\d+(?:\.\d+)?s)(?![\w-])")
EASING_RE = re.compile(
    r"cubic-bezier\([^)]+\)|ease-in-out|ease-out|ease-in|linear"
)
R3F_STATE_RE = re.compile(
    r"useFrame\s*\([^)]*\)\s*=>[\s\S]{0,500}?set[A-Z]"
)
REA_STATE_RE = re.compile(
    r"on(Update|Change|Active)\s*\([^)]*\)\s*=>[\s\S]{0,400}?set[A-Z]"
)


def iter_files(root: Path, limit: int):
    """Yield source files under root, skipping ignored directories."""
    seen = 0
    for p in root.rglob("*"):
        if seen >= limit:
            break
        if not p.is_file() or p.suffix not in SCAN_EXTS:
            continue
        rel = p.relative_to(root)
        if set(rel.parts) & IGNORE:
            continue
        seen += 1
        yield p


def audit_file(root: Path, p: Path):
    """Return motion findings for one file, or None when it is clean."""
    rel = str(p.relative_to(root))
    text = p.read_text(encoding="utf-8", errors="ignore")
    findings = []
    durations = DURATION_RE.findall(text)
    easings = EASING_RE.findall(text)
    if len(durations) >= 3:
        findings.append(
            {
                "severity": "medium",
                "type": "hardcoded-durations",
                "detail": sorted(set(durations))[:12],
            }
        )
    if len(easings) >= 3:
        findings.append(
            {
                "severity": "low",
                "type": "hardcoded-easing",
                "detail": sorted(set(easings))[:12],
            }
        )
    if "useFrame" in text and R3F_STATE_RE.search(text):
        findings.append(
            {
                "severity": "high",
                "type": "r3f-setstate-in-useframe",
                "detail": "Possible React state update inside useFrame.",
            }
        )
    idx = text.find("useFrame")
    if idx != -1 and "delta" not in text[idx:idx + 2000]:
        findings.append(
            {
                "severity": "medium",
                "type": "r3f-missing-delta",
                "detail": (
                    "useFrame appears without delta-time normalization nearby."
                ),
            }
        )
    if (
        (
            "react-native-reanimated" in text
            or "useAnimated" in text
            or "Gesture." in text
        )
        and REA_STATE_RE.search(text)
    ):
        findings.append(
            {
                "severity": "medium",
                "type": "reanimated-setstate-in-callback",
                "detail": (
                    "React state update inside a Reanimated/gesture callback "
                    "(use a shared value)."
                ),
            }
        )
    if (
        "react-native-reanimated" in text
        and "useSharedValue" not in text
        and ("withSpring" in text or "withTiming" in text)
    ):
        findings.append(
            {
                "severity": "medium",
                "type": "reanimated-no-shared-value",
                "detail": (
                    "Reanimated animation helper used without obvious shared "
                    "value."
                ),
            }
        )
    if "react-native-gesture-handler" in text and "velocity" not in text:
        findings.append(
            {
                "severity": "medium",
                "type": "gesture-no-velocity",
                "detail": (
                    "Gesture code appears to omit velocity-aware release."
                ),
            }
        )
    if (
        "prefers-reduced-motion" not in text
        and "useReducedMotion" not in text
        and "ReducedMotion" not in text
        and any(
            w in text
            for w in [
                "useFrame",
                "withSpring",
                "withTiming",
                "withRepeat",
                "parallax",
                "ScrollControls",
            ]
        )
    ):
        findings.append(
            {
                "severity": "medium",
                "type": "missing-reduced-motion-signal",
                "detail": "Motion code has no local reduced-motion signal.",
            }
        )
    return {"file": rel, "findings": findings} if findings else None


def main():
    """Run the motion-system audit CLI."""
    ap = argparse.ArgumentParser(
        description="Static audit for motion-system risks."
    )
    ap.add_argument("root", nargs="?", default=".")
    ap.add_argument("--pretty", action="store_true")
    ap.add_argument("--limit", type=int, default=4000)
    args = ap.parse_args()
    root = Path(args.root).resolve()
    if not root.is_dir():
        raise SystemExit(f"error: not a directory: {root}")
    results = []
    for p in iter_files(root, args.limit):
        item = audit_file(root, p)
        if item:
            results.append(item)
    summary = {
        "files_with_findings": len(results),
        "findings": sum(len(r["findings"]) for r in results),
    }
    print(
        json.dumps(
            {"root": str(root), "summary": summary, "results": results},
            indent=2 if args.pretty else None,
        )
    )

if __name__ == "__main__":
    main()
