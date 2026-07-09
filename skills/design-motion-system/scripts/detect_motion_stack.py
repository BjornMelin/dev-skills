#!/usr/bin/env python3
"""Detect motion, 3D, native animation, and asset stack signals."""

import argparse
import json
import os
from pathlib import Path

PACKAGE_KEYS = {
    "three": ["three"],
    "react_three_fiber": ["@react-three/fiber"],
    "drei": ["@react-three/drei"],
    "postprocessing": ["postprocessing", "@react-three/postprocessing"],
    "expo": ["expo"],
    "react_native": ["react-native"],
    "reanimated": ["react-native-reanimated"],
    "gesture_handler": ["react-native-gesture-handler"],
    "skia": ["@shopify/react-native-skia"],
    "framer_motion": ["framer-motion", "motion"],
    "gsap": ["gsap", "@gsap/react"],
}

EXTS = {
    ".ts",
    ".tsx",
    ".js",
    ".jsx",
    ".mjs",
    ".cjs",
    ".css",
    ".scss",
    ".glsl",
    ".wgsl",
}
KEYWORDS = {
    "r3f_files": ["@react-three/fiber", "<Canvas", "useFrame", "useThree"],
    "three_files": ["from 'three'", 'from "three"', "THREE."],
    "drei_files": ["@react-three/drei"],
    "shader_files": [
        "ShaderMaterial",
        "gl_FragColor",
        "fragmentShader",
        "vertexShader",
        "uniform",
    ],
    "reanimated_files": [
        "react-native-reanimated",
        "useSharedValue",
        "useAnimatedStyle",
        "withSpring",
        "withTiming",
        "withDecay",
    ],
    "gesture_files": [
        "react-native-gesture-handler",
        "GestureDetector",
        "Gesture.Pan",
        "Gesture.Tap",
    ],
    "motion_token_files": [
        "reducedMotion",
        "prefers-reduced-motion",
        "useReducedMotion",
        "--motion-duration",
        "--motion-ease",
        "motion.duration",
        "motion.easing",
        "motion.spring",
    ],
    "gsap_files": [
        "gsap.to",
        "gsap.from",
        "ScrollTrigger",
        "useGSAP",
        "from 'gsap'",
        'from "gsap"',
    ],
}

IGNORE_DIRS = {
    "node_modules", ".git", "dist", "build", ".next", ".expo", "Pods",
    ".gradle", "coverage", "target", "out", ".turbo", ".cache",
    "vendor", ".venv", "__pycache__",
}


def read_package(root: Path):
    """Read package dependencies from package.json when present."""
    path = root / "package.json"
    if not path.exists():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return {"_error": str(exc)}
    deps = {}
    for key in [
        "dependencies",
        "devDependencies",
        "peerDependencies",
        "optionalDependencies",
    ]:
        deps.update(data.get(key, {}) or {})
    return deps


def iter_files(root: Path, limit: int):
    """Yield files that can contribute stack or asset signals."""
    count = 0
    allow = EXTS | {
        ".json",
        ".glb",
        ".gltf",
        ".hdr",
        ".ktx2",
        ".png",
        ".jpg",
        ".jpeg",
        ".webp",
        ".avif",
    }
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [name for name in dirnames if name not in IGNORE_DIRS]
        for name in filenames:
            if count >= limit:
                return
            p = Path(dirpath) / name
            if p.suffix not in allow:
                continue
            count += 1
            yield p


def scan(root: Path, limit: int):
    """Scan a repository root for package, source, and asset signals."""
    deps = read_package(root)
    packages = {
        name: any(pkg in deps for pkg in candidates)
        for name, candidates in PACKAGE_KEYS.items()
    }
    matches = {name: [] for name in KEYWORDS}
    assets = {"glb": [], "gltf": [], "hdr": [], "ktx2": [], "textures": []}
    for p in iter_files(root, limit):
        rel = str(p.relative_to(root))
        low = p.suffix.lower().lstrip(".")
        if low in assets:
            assets[low].append(rel)
        if low in {"png", "jpg", "jpeg", "webp", "avif"}:
            assets["textures"].append(rel)
        # .json is used for asset/dep detection above, never for keyword
        # scanning (bare "motion"/"duration" in package.json / fixtures caused
        # false positives).
        if p.suffix not in EXTS:
            continue
        try:
            text = p.read_text(encoding="utf-8", errors="ignore")[:25000]
        except OSError:
            continue
        for bucket, words in KEYWORDS.items():
            if any(w in text for w in words):
                matches[bucket].append(rel)
    return {
        "root": str(root),
        "packages": packages,
        "matches": matches,
        "assets": assets,
    }


def main():
    """Run the stack detection CLI."""
    ap = argparse.ArgumentParser(
        description=(
            "Detect 3D, motion, and native animation stack signals in a repo."
        )
    )
    ap.add_argument("root", nargs="?", default=".")
    ap.add_argument("--pretty", action="store_true")
    ap.add_argument("--limit", type=int, default=4000)
    args = ap.parse_args()
    root = Path(args.root).resolve()
    if not root.is_dir():
        raise SystemExit(f"error: not a directory: {root}")
    data = scan(root, args.limit)
    print(
        json.dumps(data, indent=2 if args.pretty else None, sort_keys=True)
    )

if __name__ == "__main__":
    main()
