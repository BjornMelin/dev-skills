#!/usr/bin/env python3
"""Sync and validate the bundled package source map."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from common import bundled_source_map_path


REQUIRED_KEYS = (
    "packageName",
    "officialDocs",
    "officialApiReference",
    "officialGithubRepo",
    "sourceConfidence",
    "verifiedAt",
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--source",
        help="Optional external package_source_map.json to copy into the bundled skill location.",
    )
    parser.add_argument(
        "--skill-script",
        default=__file__,
        help="Any script path inside the upgrade-pack-generator skill; used to resolve the bundled destination.",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Validate only; do not overwrite the bundled source map.",
    )
    return parser


def validate_payload(payload: Any) -> list[str]:
    """Return validation errors for a source-map payload."""
    errors: list[str] = []
    if not isinstance(payload, list) or not payload:
        return ["source map must be a non-empty JSON list"]
    for index, entry in enumerate(payload):
        if not isinstance(entry, dict):
            errors.append(f"entry {index} must be an object")
            continue
        for key in REQUIRED_KEYS:
            value = entry.get(key)
            if key in {"officialDocs", "officialApiReference"} and value is None:
                errors.append(f"entry {index}.{key} must be a non-empty string")
                continue
            if not isinstance(value, str) or not value.strip():
                errors.append(f"entry {index}.{key} must be a non-empty string")
    return errors


def main() -> None:
    args = build_parser().parse_args()
    bundled_path = bundled_source_map_path(args.skill_script)
    source_path = Path(args.source).expanduser().resolve() if args.source else bundled_path
    payload = json.loads(source_path.read_text(encoding="utf-8"))
    errors = validate_payload(payload)
    if errors:
        print("Source map validation failed:")
        for error in errors:
            print(f"- {error}")
        raise SystemExit(1)
    if not args.check and source_path != bundled_path:
        bundled_path.parent.mkdir(parents=True, exist_ok=True)
        bundled_path.write_text(
            json.dumps(payload, indent=2, sort_keys=False) + "\n",
            encoding="utf-8",
        )
    print(bundled_path)


if __name__ == "__main__":
    main()
