#!/usr/bin/env python3
"""Detect whether a repo has a Next.js/React web surface and suggest a path."""

from __future__ import annotations

import argparse
import json
from pathlib import Path


def read_text(path: Path) -> str:
    return path.read_text(errors="ignore") if path.exists() else ""


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--json", action="store_true")
    parser.add_argument("path", nargs="?", default=".")
    args = parser.parse_args()

    root = Path(args.path).resolve()
    package_json = read_text(root / "package.json")
    apps_web_pkg = read_text(root / "apps/web/package.json")
    text = f"{package_json}\n{apps_web_pkg}"

    has_next = '"next"' in text
    has_react = '"react"' in text
    has_app_router = (root / "apps/web/app").exists() or (root / "app").exists()
    has_components_json = (root / "apps/web/components.json").exists() or (
        root / "components.json"
    ).exists()

    recommended = "next-route" if has_next and has_app_router else "react-surface"
    if has_components_json:
        recommended = f"{recommended}+shadcn"

    result = {
        "root": str(root),
        "has_next": has_next,
        "has_react": has_react,
        "has_app_router": has_app_router,
        "has_components_json": has_components_json,
        "recommended_path": recommended,
    }

    if args.json:
        print(json.dumps(result, indent=2))
        return 0

    print(f"Root: {root}")
    print(f"Has Next: {has_next}")
    print(f"Has React: {has_react}")
    print(f"Has App Router: {has_app_router}")
    print(f"Has components.json: {has_components_json}")
    print(f"Recommended path: {recommended}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
