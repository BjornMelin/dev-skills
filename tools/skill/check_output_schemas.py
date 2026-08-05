#!/usr/bin/env python3
"""Verify every skill output schema is valid for strict structured outputs.

Schemas under `skills/*/references/*schema*.json` are handed to
`codex exec --output-schema`, which compiles to OpenAI structured outputs.
Strict mode is stricter than JSON Schema:

* every key in `properties` must also appear in `required` --
  optional-by-omission is illegal
* `additionalProperties` must be `false` on every object

A schema that violates either is still valid JSON and still valid JSON Schema,
so `jq empty` and a `json.load` both pass. It fails at request time with a 400,
after the model has been dispatched. This check closes that gap: the failure
becomes a CI error instead of a runtime surprise.

Optionality is expressed by making the key required and its type nullable,
e.g. `{"type": ["string", "null"]}`.

Usage: python3 tools/skill/check_output_schemas.py [root]
"""

from __future__ import annotations

import json
import pathlib
import sys

SKIP_DIR_PARTS = {"node_modules", "target", ".git", "archive"}


def _type_is_object(schema_type: object) -> bool:
    """True when a schema type is object, including nullable arrays.

    Arrays like `["object", "null"]` express required-and-nullable fields,
    the shape strict structured outputs use for optional objects.

    Args:
        schema_type: The value of a schema node's `type` key.

    Returns:
        True when the type is `"object"` or an array containing `"object"`.
    """
    if schema_type == "object":
        return True
    return isinstance(schema_type, list) and "object" in schema_type


def walk(node: object, path: str, errors: list[str]) -> None:
    """Recursively check one schema node and its children for strict mode.

    Args:
        node: The schema fragment to check.
        path: Dot-separated location of the fragment for error messages.
        errors: Accumulates human-readable violations in place.

    Returns:
        None. Violations are appended to ``errors``.
    """
    if isinstance(node, dict):
        # A node counts as an object either by declared type or by carrying
        # `properties` without one. Both are checked, but the violation is
        # recorded once: reporting the same path twice made a single defect
        # read as two.
        declares_object = _type_is_object(node.get("type"))
        props = node.get("properties")
        is_object = declares_object or isinstance(props, dict)
        if is_object and node.get("additionalProperties") is not False:
            errors.append(
                f'{path}: objects must set "additionalProperties": false'
            )
        if isinstance(props, dict):
            required = node.get("required")
            if not isinstance(required, list):
                errors.append(
                    f"{path}: has 'properties' but no 'required' array"
                )
                required = []
            missing = [k for k in props if k not in required]
            if missing:
                errors.append(
                    f"{path}: 'required' must include every key in "
                    f"'properties'; missing {missing}"
                )
            extra = [k for k in required if k not in props]
            if extra:
                errors.append(
                    f"{path}: 'required' names keys absent from "
                    f"'properties': {extra}"
                )
            for key, sub in props.items():
                walk(sub, f"{path}.{key}", errors)
        for key, sub in node.items():
            if key != "properties":
                walk(sub, f"{path}.{key}" if key == "items" else path, errors)
    elif isinstance(node, list):
        for item in node:
            walk(item, path, errors)


def _has_object(node: object) -> bool:
    """True when the schema declares an object anywhere.

    A node counts even without `properties`: strict mode still requires
    `additionalProperties: false` on bare object types and on nullable
    object arrays.

    Args:
        node: The schema fragment to inspect.

    Returns:
        True when any nested type is an object.
    """
    if isinstance(node, dict):
        if _type_is_object(node.get("type")) or "properties" in node:
            return True
        nested = (v for k, v in node.items() if k != "description")
        return any(_has_object(v) for v in nested)
    if isinstance(node, list):
        return any(_has_object(v) for v in node)
    return False


def schema_files(root: pathlib.Path) -> list[pathlib.Path]:
    """Find schema files under skills and plugins.

    Args:
        root: Repository root directory.

    Returns:
        Sorted list of candidate schema paths, skipping generated and
        archived directories.
    """
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
    """Check all candidate schemas and report strict-mode violations.

    Args:
        argv: Command-line arguments; an optional root directory.

    Returns:
        0 when every schema is strict-mode valid, 1 otherwise.
    """
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
        if not isinstance(schema, dict):
            continue
        # Do not require `properties` at the root: a root `{"type": "object"}`
        # with no properties, or an `anyOf` of object branches, still has
        # objects that strict mode requires to set additionalProperties: false.
        # Skipping them left that class unchecked.
        if not _has_object(schema):
            continue
        checked += 1
        errors: list[str] = []
        walk(schema, "$", errors)
        if errors:
            failures += 1
            for error in errors:
                print(f"INVALID {rel} {error}", file=sys.stderr)
    if failures:
        print(
            f"output-schema check: {failures} file(s) not strict-mode valid",
            file=sys.stderr,
        )
        return 1
    print(f"output-schema-ok ({checked} schema(s) checked)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
