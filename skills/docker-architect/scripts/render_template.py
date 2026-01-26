from __future__ import annotations

import argparse
import json
import os
import re
from dataclasses import dataclass
from pathlib import Path


PLACEHOLDER_RE = re.compile(r"{{\s*([A-Z0-9_]+)\s*}}")


@dataclass(frozen=True)
class RenderSpec:
    template: str
    out: str
    variables: dict[str, str]
    allow_missing: bool
    chmod_x: bool


def _default_templates_dir() -> Path:
    return Path(__file__).resolve().parent.parent / "assets" / "templates"


def _parse_vars(var_args: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    for v in var_args:
        if "=" not in v:
            raise ValueError(f"Invalid --var (expected KEY=VALUE): {v}")
        k, value = v.split("=", 1)
        k = k.strip()
        if not k:
            raise ValueError(f"Invalid --var (empty key): {v}")
        out[k] = value
    return out


def _load_vars_file(path: Path) -> dict[str, str]:
    raw = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        raise ValueError("--vars-file must be a JSON object")
    out: dict[str, str] = {}
    for k, v in raw.items():
        if not isinstance(k, str) or not isinstance(v, (str, int, float, bool)):
            raise ValueError("--vars-file values must be JSON scalars")
        out[k] = str(v)
    return out


def render(spec: RenderSpec, templates_dir: Path) -> None:
    template_path = (templates_dir / spec.template).resolve()
    templates_dir = templates_dir.resolve()
    if templates_dir not in template_path.parents:
        raise ValueError("Template path escapes templates directory")

    template_text = template_path.read_text(encoding="utf-8")

    def repl(match: re.Match[str]) -> str:
        key = match.group(1)
        if key in spec.variables:
            return spec.variables[key]
        env_key = f"DOCKER_ARCH_{key}"
        if env_key in os.environ:
            return os.environ[env_key]
        if spec.allow_missing:
            return match.group(0)
        raise KeyError(f"Missing variable: {key} (set --var {key}=... or env {env_key})")

    rendered = PLACEHOLDER_RE.sub(repl, template_text)
    if not spec.allow_missing and PLACEHOLDER_RE.search(rendered):
        remaining = sorted(set(PLACEHOLDER_RE.findall(rendered)))
        raise ValueError(f"Unresolved placeholders remain: {', '.join(remaining)}")

    out_path = Path(spec.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(rendered, encoding="utf-8")
    if spec.chmod_x:
        out_path.chmod(out_path.stat().st_mode | 0o111)


def main() -> int:
    parser = argparse.ArgumentParser(description="Render a Docker Architect template into a repo file.")
    parser.add_argument(
        "--templates-dir",
        type=Path,
        default=_default_templates_dir(),
        help="Templates directory (default: skill assets/templates)",
    )
    parser.add_argument("--template", required=True, help="Template path relative to templates dir")
    parser.add_argument("--out", required=True, help="Output file path")
    parser.add_argument("--var", action="append", default=[], help="Template variable KEY=VALUE (repeatable)")
    parser.add_argument("--vars-file", type=Path, help="JSON file with variables (merged, overridden by --var)")
    parser.add_argument("--allow-missing", action="store_true", help="Leave unresolved placeholders as-is")
    parser.add_argument("--chmod-x", action="store_true", help="Mark output file executable")
    args = parser.parse_args()

    variables: dict[str, str] = {}
    if args.vars_file:
        variables.update(_load_vars_file(args.vars_file))
    variables.update(_parse_vars(args.var))

    spec = RenderSpec(
        template=args.template,
        out=args.out,
        variables=variables,
        allow_missing=args.allow_missing,
        chmod_x=args.chmod_x,
    )
    render(spec, templates_dir=args.templates_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

