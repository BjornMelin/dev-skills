#!/usr/bin/env python3

import argparse
import datetime as dt
import os
import re
import sys


def utc_now_stamp() -> str:
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%d %H:%MZ")


def slugify(value: str) -> str:
    value = value.strip().lower()
    value = re.sub(r"[^a-z0-9]+", "-", value)
    value = value.strip("-")
    return value or "plan"


def read_template(skill_root: str) -> str:
    template_path = os.path.join(skill_root, "assets", "templates", "execplan.md")
    with open(template_path, "r", encoding="utf-8") as f:
        return f.read()


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Create a new ExecPlan file under execplans/ from the codex-sdk ExecPlan template."
    )
    parser.add_argument(
        "--dir",
        default=".",
        help="Target repository directory (default: current directory).",
    )
    parser.add_argument(
        "--name",
        required=True,
        help="Short name for the plan (used in filename). Example: 'migrate-bun' or 'add-mcp-tools'.",
    )
    parser.add_argument(
        "--title",
        default=None,
        help="Title for the plan (first heading). Defaults to a title-cased variant of --name.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite if the plan file already exists.",
    )
    args = parser.parse_args()

    target_dir = os.path.abspath(args.dir)
    execplans_dir = os.path.join(target_dir, "execplans")
    os.makedirs(execplans_dir, exist_ok=True)

    plan_slug = slugify(args.name)
    plan_path = os.path.join(execplans_dir, f"execplan-{plan_slug}.md")

    if os.path.exists(plan_path) and not args.force:
        sys.stderr.write(f"Refusing to overwrite existing plan: {plan_path}\n")
        return 2

    skill_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    template = read_template(skill_root)

    title = args.title or " ".join(w.capitalize() for w in plan_slug.split("-"))
    lines = template.splitlines()

    # Replace the first heading with the provided title.
    if lines and lines[0].startswith("# "):
        lines[0] = f"# {title}"
    else:
        lines.insert(0, f"# {title}")

    # Replace the first Progress timestamp placeholder.
    stamp = utc_now_stamp()
    updated = []
    replaced_progress = False
    for line in lines:
        if (not replaced_progress) and "## Progress" in line:
            updated.append(line)
            replaced_progress = True
            continue
        if replaced_progress and line.strip().startswith("- [ ] (YYYY-"):
            updated.append(f"- [ ] ({stamp}) Initial draft created.")
            replaced_progress = False  # only replace once
            continue
        updated.append(line)

    with open(plan_path, "w", encoding="utf-8") as f:
        f.write("\n".join(updated).rstrip() + "\n")

    sys.stdout.write(plan_path + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

