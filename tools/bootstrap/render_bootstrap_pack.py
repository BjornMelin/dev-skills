#!/usr/bin/env python3
"""Render manifest-backed Codex repo bootstrap packs."""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from string import Template
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
PACK_ROOT = REPO_ROOT / "bootstrap" / "packs"
TEMPLATE_ROOT = REPO_ROOT / "bootstrap" / "templates"
TEMPLATE_ROOT_RESOLVED = TEMPLATE_ROOT.resolve()
SCHEMA = "dev-skills.bootstrap-pack.v1"


@dataclass(frozen=True)
class RenderedFile:
    """One planned or written bootstrap file."""

    target: Path
    template: Path
    action: str


def build_parser() -> argparse.ArgumentParser:
    """Build the command parser."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", help="Bootstrap pack name or manifest path.")
    parser.add_argument("--out", type=Path, help="Directory to render into.")
    parser.add_argument("--repo-name", default="new-repo")
    parser.add_argument("--primary-language", default="unspecified")
    parser.add_argument("--generated-at", help="RFC3339 UTC timestamp override.")
    parser.add_argument("--var", action="append", default=[], help="Extra key=value template variable.")
    parser.add_argument("--list", action="store_true", help="List available packs.")
    parser.add_argument("--validate", action="store_true", help="Validate pack manifests and templates.")
    parser.add_argument("--dry-run", action="store_true", help="Print planned writes without creating files.")
    parser.add_argument("--force", action="store_true", help="Overwrite existing target files.")
    parser.add_argument("--json", action="store_true", help="Emit JSON output.")
    return parser


def load_pack(path_or_name: str) -> tuple[Path, dict[str, Any]]:
    """Load a bootstrap pack manifest by name or path."""

    candidate = Path(path_or_name)
    if not candidate.exists():
        candidate = PACK_ROOT / f"{path_or_name}.json"
    if not candidate.is_file():
        raise SystemExit(f"unknown bootstrap pack: {path_or_name}")
    try:
        payload = json.loads(candidate.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid pack JSON {candidate}: {exc}") from exc
    return candidate, payload


def iter_pack_paths() -> list[Path]:
    """Return known pack manifests in stable order."""

    return sorted(PACK_ROOT.glob("*.json"))


def safe_relative_path(value: str, *, source: Path) -> Path:
    """Validate a manifest path is relative and cannot escape the output root."""

    path = Path(value)
    if path.is_absolute() or ".." in path.parts:
        raise SystemExit(f"unsafe relative path in {source}: {value}")
    return path


def resolve_contained_path(base: Path, relative: str, *, source: Path) -> Path:
    """Resolve a safe relative path and require it to stay under base."""

    relative_path = safe_relative_path(relative, source=source)
    root = base.resolve()
    resolved = (root / relative_path).resolve()
    if resolved != root and root not in resolved.parents:
        raise SystemExit(f"path escapes {root}: {relative}")
    return resolved


def validate_pack(path: Path, payload: dict[str, Any]) -> list[str]:
    """Validate one pack manifest and referenced templates."""

    errors: list[str] = []
    if payload.get("schema") != SCHEMA:
        errors.append(f"{path}: schema must be {SCHEMA}")
    if not isinstance(payload.get("name"), str) or not payload["name"]:
        errors.append(f"{path}: name must be a non-empty string")
    files = payload.get("files")
    if not isinstance(files, list) or not files:
        errors.append(f"{path}: files must be a non-empty array")
        return errors
    for index, item in enumerate(files):
        if not isinstance(item, dict):
            errors.append(f"{path}: files[{index}] must be an object")
            continue
        target = item.get("target")
        template = item.get("template")
        if not isinstance(target, str) or not target:
            errors.append(f"{path}: files[{index}].target must be a string")
        else:
            try:
                safe_relative_path(target, source=path)
            except SystemExit as exc:
                errors.append(str(exc))
        if not isinstance(template, str) or not template:
            errors.append(f"{path}: files[{index}].template must be a string")
            continue
        try:
            template_path = resolve_contained_path(
                TEMPLATE_ROOT_RESOLVED,
                template,
                source=path,
            )
        except SystemExit as exc:
            errors.append(str(exc))
            continue
        if not template_path.is_file():
            errors.append(f"{path}: missing template {template}")
    advisory_checks = payload.get("advisory_host_checks", [])
    if not isinstance(advisory_checks, list):
        errors.append(f"{path}: advisory_host_checks must be an array when present")
    else:
        for index, item in enumerate(advisory_checks):
            if not isinstance(item, str) or not item:
                errors.append(
                    f"{path}: advisory_host_checks[{index}] must be a non-empty string"
                )
    return errors


def parse_vars(values: list[str]) -> dict[str, str]:
    """Parse repeated key=value template variables."""

    parsed: dict[str, str] = {}
    for value in values:
        if "=" not in value:
            raise SystemExit(f"--var must use key=value form: {value}")
        key, raw = value.split("=", 1)
        if not key:
            raise SystemExit(f"--var key cannot be empty: {value}")
        parsed[key] = raw
    return parsed


def template_vars(args: argparse.Namespace, payload: dict[str, Any]) -> dict[str, str]:
    """Build default and caller-supplied template variables."""

    generated_at = args.generated_at or datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")
    values = {
        "dev_skills_path": "/path/to/dev-skills",
        "pack_name": str(payload["name"]),
        "repo_name": args.repo_name,
        "primary_language": args.primary_language,
        "generated_at": generated_at,
    }
    values.update(parse_vars(args.var))
    return values


def render_pack(
    manifest_path: Path,
    payload: dict[str, Any],
    *,
    out: Path,
    variables: dict[str, str],
    dry_run: bool,
    force: bool,
) -> list[RenderedFile]:
    """Render one pack into an output directory."""

    errors = validate_pack(manifest_path, payload)
    if errors:
        raise SystemExit("\n".join(errors))

    output_root = out.expanduser().resolve()
    rendered: list[RenderedFile] = []
    for item in payload["files"]:
        target = resolve_contained_path(
            output_root,
            item["target"],
            source=manifest_path,
        )
        template = resolve_contained_path(
            TEMPLATE_ROOT_RESOLVED,
            item["template"],
            source=manifest_path,
        )
        action = "would_write" if dry_run else "written"
        if target.exists():
            if not dry_run and not force:
                raise SystemExit(f"target exists; pass --force to overwrite: {target}")
            action = "would_overwrite" if dry_run else "overwritten"
        rendered.append(RenderedFile(target=target, template=template, action=action))
        if dry_run:
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        content = Template(template.read_text(encoding="utf-8")).safe_substitute(variables)
        target.write_text(content, encoding="utf-8")
    return rendered


def print_output(result: dict[str, Any], *, as_json: bool) -> None:
    """Print command output."""

    if as_json:
        print(json.dumps(result, indent=2, sort_keys=True))
        return
    if "packs" in result:
        for pack in result["packs"]:
            print(f"{pack['name']}: {pack['description']}")
        return
    if "ok" in result:
        if result["ok"]:
            print("ok")
            return
        for error in result.get("errors", []):
            print(f"failed: {error}")
        return
    for file in result.get("files", []):
        print(f"{file['action']}: {file['target']}")
    for check in result.get("advisory_host_checks", []):
        print(f"advisory_check: {check}")


def main(argv: list[str] | None = None) -> int:
    """Run the bootstrap renderer."""

    args = build_parser().parse_args(argv)
    if args.list:
        packs = []
        for path in iter_pack_paths():
            payload = json.loads(path.read_text(encoding="utf-8"))
            packs.append(
                {
                    "name": payload.get("name", path.stem),
                    "path": str(path.relative_to(REPO_ROOT)),
                    "description": payload.get("description", ""),
                }
            )
        print_output({"schema": SCHEMA, "packs": packs}, as_json=args.json)
        return 0

    if args.validate:
        errors: list[str] = []
        paths = [load_pack(args.pack)[0]] if args.pack else iter_pack_paths()
        for path in paths:
            payload = json.loads(path.read_text(encoding="utf-8"))
            errors.extend(validate_pack(path, payload))
        result = {"schema": SCHEMA, "ok": not errors, "errors": errors}
        print_output(result, as_json=args.json)
        return 0 if not errors else 1

    if not args.pack or not args.out:
        raise SystemExit("--pack and --out are required unless using --list or --validate")
    manifest_path, payload = load_pack(args.pack)
    files = render_pack(
        manifest_path,
        payload,
        out=args.out,
        variables=template_vars(args, payload),
        dry_run=args.dry_run,
        force=args.force,
    )
    result = {
        "schema": SCHEMA,
        "pack": payload["name"],
        "out": str(args.out),
        "dry_run": args.dry_run,
        "advisory_host_checks": payload.get("advisory_host_checks", []),
        "files": [
            {
                "target": str(file.target),
                "template": str(file.template.relative_to(REPO_ROOT)),
                "action": file.action,
            }
            for file in files
        ],
    }
    print_output(result, as_json=args.json)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
