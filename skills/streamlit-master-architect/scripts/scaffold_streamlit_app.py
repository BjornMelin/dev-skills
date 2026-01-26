from __future__ import annotations

import argparse
import shutil
from pathlib import Path


def _templates_dir() -> Path:
    return Path(__file__).resolve().parents[1] / "templates"


def list_templates() -> list[str]:
    root = _templates_dir()
    if not root.exists():
        return []
    return sorted([p.name for p in root.iterdir() if p.is_dir() and not p.name.startswith("_")])


def copy_template(*, template: str, dest: Path) -> None:
    src = _templates_dir() / template
    if not src.exists() or not src.is_dir():
        raise FileNotFoundError(f"Template not found: {src}")
    shutil.copytree(src, dest, dirs_exist_ok=False)


def main() -> int:
    parser = argparse.ArgumentParser(description="Scaffold a Streamlit app from bundled templates.")
    parser.add_argument("--list", action="store_true", help="List available templates and exit.")
    parser.add_argument("--template", type=str, help="Template name (see --list).")
    parser.add_argument("--dest", type=str, help="Destination directory (must not exist).")
    args = parser.parse_args()

    if args.list:
        for name in list_templates():
            print(name)
        return 0

    if not args.template or not args.dest:
        parser.error("--template and --dest are required (or use --list).")

    dest = Path(args.dest).resolve()
    if dest.exists():
        raise FileExistsError(f"Destination already exists: {dest}")

    copy_template(template=args.template, dest=dest)
    print(f"[ok] Created {dest} from template '{args.template}'")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

