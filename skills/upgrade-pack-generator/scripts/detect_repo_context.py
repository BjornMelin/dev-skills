#!/usr/bin/env python3
"""Detect repo package-manager and framework context for upgrade-pack generation."""

from __future__ import annotations

import argparse
import json

from common import detect_repo_context, repo_path


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True, help="Path to the target repo root.")
    parser.add_argument("--json", action="store_true", help="Emit JSON instead of text.")
    return parser


def main() -> None:
    args = build_parser().parse_args()
    context = detect_repo_context(repo_path(args.repo_root))

    if args.json:
        print(json.dumps(context, indent=2, sort_keys=False))
        return

    print(f"repo_root: {context['repo_root']}")
    print(f"package_manager: {context['package_manager']}")
    print(f"detected_by: {context['detected_by']}")
    print(f"root_lockfiles: {', '.join(context['root_lockfiles']) or '(none)'}")
    frameworks = ", ".join(context["frameworks_detected"]) or "(none)"
    print(f"frameworks_detected: {frameworks}")


if __name__ == "__main__":
    main()
