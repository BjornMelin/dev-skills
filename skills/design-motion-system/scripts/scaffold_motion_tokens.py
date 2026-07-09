#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

TS_CONTENT = '''export const motion = {
  duration: {
    instant: 0,
    micro: 120,
    short: 200,
    medium: 360,
    long: 700,
    cinematic: 1200,
  },
  easing: {
    out: [0.16, 1, 0.3, 1],
    inOut: [0.65, 0, 0.35, 1],
    emphasized: [0.2, 0, 0, 1],
    linear: [0, 0, 1, 1],
  },
  spring: {
    snappy: { stiffness: 520, damping: 42, mass: 0.85 },
    soft: { stiffness: 220, damping: 28, mass: 1 },
    bouncy: { stiffness: 340, damping: 20, mass: 0.9 },
    heavy: { stiffness: 180, damping: 34, mass: 1.4 },
    gesture: { stiffness: 420, damping: 36, mass: 1 },
  },
  depth: {
    card: { z: 12, scale: 1.02 },
    sheet: { z: 32, scale: 1.0 },
    hero: { z: 80, scale: 1.06 },
  },
  parallax: {
    subtle: 0.035,
    medium: 0.07,
    hero: 0.12,
  },
  reduced: {
    instant: true,
    fadeOnlyDuration: 120,
    disableCameraTravel: true,
    disableParallax: true,
    disableLoops: true,
    disableBounce: true,
  },
} as const;

export type MotionTokens = typeof motion;
'''

CSS_CONTENT = ''':root {
  --motion-duration-instant: 0ms;
  --motion-duration-micro: 120ms;
  --motion-duration-short: 200ms;
  --motion-duration-medium: 360ms;
  --motion-duration-long: 700ms;
  --motion-duration-cinematic: 1200ms;
  --motion-ease-out: cubic-bezier(0.16, 1, 0.3, 1);
  --motion-ease-in-out: cubic-bezier(0.65, 0, 0.35, 1);
  --motion-ease-emphasized: cubic-bezier(0.2, 0, 0, 1);
  --motion-parallax-subtle: 0.035;
  --motion-parallax-medium: 0.07;
  --motion-parallax-hero: 0.12;
}

@media (prefers-reduced-motion: reduce) {
  :root {
    --motion-duration-micro: 0ms;
    --motion-duration-short: 0ms;
    --motion-duration-medium: 0ms;
    --motion-duration-long: 0ms;
    --motion-duration-cinematic: 0ms;
    --motion-parallax-subtle: 0;
    --motion-parallax-medium: 0;
    --motion-parallax-hero: 0;
  }
}
'''

REA_CONTENT = '''import { Easing } from "react-native-reanimated";
import { motion } from "./motion";

export const reanimatedMotion = {
  duration: motion.duration,
  easing: {
    out: Easing.bezier(...motion.easing.out),
    inOut: Easing.bezier(...motion.easing.inOut),
    emphasized: Easing.bezier(...motion.easing.emphasized),
    linear: Easing.linear,
  },
  spring: motion.spring,
} as const;
'''


def has_native_deps(root: Path):
    path = root / "package.json"
    if not path.exists():
        return False
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return False
    deps = {}
    deps.update(data.get("dependencies", {}) or {})
    deps.update(data.get("devDependencies", {}) or {})
    return "react-native" in deps or "react-native-reanimated" in deps


def main():
    ap = argparse.ArgumentParser(description="Scaffold motion token files. Dry run by default.")
    ap.add_argument("root", nargs="?", default=".")
    ap.add_argument("--dir", default="src/design-system/motion")
    ap.add_argument("--stack", choices=["auto", "web", "native", "both"], default="auto")
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args()
    root = Path(args.root).resolve()
    if not root.is_dir():
        ap.error(f"not a directory: {root}")
    out = (root / args.dir).resolve()
    # Output-path guard: keep writes inside the project root (no absolute --dir, no '..' escape).
    try:
        out.relative_to(root)
    except ValueError:
        ap.error("--dir must stay inside the project root (no absolute paths or '..' escapes)")
    files = {
        out / "motion.ts": TS_CONTENT,
        out / "motion.css": CSS_CONTENT,
    }
    if args.stack in {"native", "both"} or (args.stack == "auto" and has_native_deps(root)):
        files[out / "reanimated-motion.ts"] = REA_CONTENT
    for path, content in files.items():
        print(str(path))
        if args.write:
            path.parent.mkdir(parents=True, exist_ok=True)
            if not path.exists():
                path.write_text(content, encoding="utf-8")
            else:
                print(f"  exists, skipped: {path}")
    if not args.write:
        print("Dry run only. Re-run with --write to create files.")

if __name__ == "__main__":
    main()
