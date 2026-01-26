#!/usr/bin/env python3
from __future__ import annotations

import argparse
import shlex
import subprocess
import sys
from importlib import metadata
from typing import Iterable


def _installed_version(dist: str) -> str | None:
    try:
        return metadata.version(dist)
    except metadata.PackageNotFoundError:
        return None


def _build_specs(packages: Iterable[str]) -> list[str]:
    specs: list[str] = []
    for pkg in packages:
        if pkg.startswith(("pypi:", "crates:", "npm:", "github:", "http://", "https://")):
            specs.append(pkg)
            continue
        version = _installed_version(pkg)
        if not version:
            print(f"[warn] Could not determine installed version for `{pkg}`; pass an explicit spec like `pypi:{pkg}@<version>`.", file=sys.stderr)
            continue
        specs.append(f"pypi:{pkg}@{version}")
    return specs


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Snapshot dependency source into ./opensrc using the opensrc CLI (read-only; --modify=false)."
    )
    parser.add_argument(
        "--packages",
        nargs="+",
        required=True,
        help="Packages to snapshot. Use distribution names (auto-resolves installed versions) or explicit opensrc specs (e.g. pypi:langgraph@1.0.3).",
    )
    parser.add_argument("--dry-run", action="store_true", help="Print commands without executing.")
    args = parser.parse_args()

    specs = _build_specs(args.packages)
    if not specs:
        print("[error] No valid package specs to snapshot.", file=sys.stderr)
        return 2

    commands: list[list[str]] = []
    for spec in specs:
        commands.append(["npx", "opensrc", spec, "--modify=false"])

    for cmd in commands:
        pretty = " ".join(shlex.quote(x) for x in cmd)
        if args.dry_run:
            print(pretty)
            continue
        print(f"[run] {pretty}")
        subprocess.run(cmd, check=True)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

