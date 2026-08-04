#!/usr/bin/env python3
"""Verify every skill output schema is valid for strict structured outputs.

Schemas under `skills/*/references/*schema*.json` are handed to `codex exec --output-schema`,
which compiles to OpenAI structured outputs. Strict mode is stricter than JSON Schema:

* every key in `properties` must also appear in `required` -- optional-by-omission is illegal
* `additionalProperties` must be `false` on every object

A schema that violates either is still valid JSON and still valid JSON Schema, so
`jq empty` and a `json.load` both pass. It fails at request time with a 400, after the model
has been dispatched. This check closes that gap: the failure becomes a CI error instead of a
runtime surprise.

Optionality is expressed by making the key required and its type nullable, e.g.
`{"type": ["string", "null"]}`.

Usage: python3 tools/skill/check_output_schemas.py [root]
"""

from __future__ import annotations

import json
import pathlib
import sys

SKIP_DIR_PARTS = {"node_modules", "target", ".git", "archive"}


def walk(node: object, path: str, errors: list[str]) -> None:
    if isinstance(node, dict):
        props = node.get("properties")
        if isinstance(props, dict):
            required = node.get("required")
            if not isinstance(required, list):
                errors.append(f"{path}: has 'properties' but no 'required' array")
                required = []
            missing = [k for k in props if k not in required]
            if missing:
                errors.append(
                    f"{path}: 'required' must include every key in 'properties'; "
                    f"missing {missing}"
                )
            extra = [k for k in required if k not in props]
            if extra:
                errors.append(f"{path}: 'required' names keys absent from 'properties': {extra}")
            if node.get("additionalProperties") is not False:
                errors.append(f"{path}: objects must set \"additionalProperties\": false")
            for key, sub in props.items():
                walk(sub, f"{path}.{key}", errors)
        for key, sub in node.items():
            if key != "properties":
                walk(sub, f"{path}.{key}" if key == "items" else path, errors)
    elif isinstance(node, list):
        for item in node:
            walk(item, path, errors)


def schema_files(root: pathlib.Path) -> list[pathlib.Path]:
    found = []
    for base in ("skills", "plugins"):
        for path in (root / base).rglob("*.json"):
            if SKIP_DIR_PARTS & set(path.parts):
                continue
            if "schema" not in path.name:
                continue
            if path.parent.name not in {"references", "schemas"}:
                continue
            found.append(path)
    return sorted(found)


def main(argv: list[str]) -> int:
    root = pathlib.Path(argv[1]) if len(argv) > 1 else pathlib.Path.cwd()
    failures = 0
    checked = 0
    for path in schema_files(root):
        rel = path.relative_to(root)
        try:
            schema = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            print(f"INVALID {rel}: {exc}", file=sys.stderr)
            failures += 1
            continue
        # A JSON Schema meta-file (draft declaration only) has no object shape to check.
        if not isinstance(schema, dict) or "properties" not in schema:
            continue
        checked += 1
        errors: list[str] = []
        walk(schema, "$", errors)
        if errors:
            failures += 1
            for error in errors:
                print(f"INVALID {rel} {error}", file=sys.stderr)
    if failures:
        print(f"output-schema check: {failures} file(s) not strict-mode valid", file=sys.stderr)
        return 1
    print(f"output-schema-ok ({checked} schema(s) checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
