#!/usr/bin/env python3

import argparse
import os
import shutil


def main():
    parser = argparse.ArgumentParser(
        description="Initialize a minimal .agent workspace (AGENTS.md + PLANS.md) for Codex-driven planning."
    )
    parser.add_argument(
        "--dir",
        default=".",
        help="Target repository directory (default: current directory).",
    )
    parser.add_argument(
        "--no-execplans",
        action="store_true",
        help="Do not create execplans/ or copy the ExecPlan template.",
    )
    args = parser.parse_args()

    target_dir = os.path.abspath(args.dir)
    agent_dir = os.path.join(target_dir, ".agent")
    os.makedirs(agent_dir, exist_ok=True)

    skill_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    templates_dir = os.path.join(skill_root, "assets", "templates", "agent")

    for name in ["AGENTS.md", "PLANS.md"]:
        src = os.path.join(templates_dir, name)
        dst = os.path.join(agent_dir, name)
        if os.path.exists(dst):
            continue
        shutil.copyfile(src, dst)

    if not args.no_execplans:
        execplans_dir = os.path.join(target_dir, "execplans")
        os.makedirs(execplans_dir, exist_ok=True)
        src = os.path.join(skill_root, "assets", "templates", "execplan.md")
        dst = os.path.join(execplans_dir, "execplan-template.md")
        if not os.path.exists(dst):
            shutil.copyfile(src, dst)

    print(agent_dir)


if __name__ == "__main__":
    main()
