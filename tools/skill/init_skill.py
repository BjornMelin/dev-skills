#!/usr/bin/env python3
"""
Skill Initializer - Creates a new skill from a template.

Usage:
  python3 tools/skill/init_skill.py <skill-name> --path <path>

Examples:
  python3 tools/skill/init_skill.py my-new-skill --path skills
  python3 tools/skill/init_skill.py custom-skill --path /custom/location
"""

import sys
from pathlib import Path


SKILL_TEMPLATE = """---
name: {skill_name}
description: [TODO: Explain what the skill does and when to use it.]
---

# {skill_title}

## Overview

[TODO: 1-2 sentences explaining what this skill enables.]

## Workflow

[TODO: Add the core workflow, decision tree, and examples. Move large content to references/.]

## Resources

If needed, add:

- `scripts/` for deterministic helpers
- `references/` for long-form docs loaded on demand
- `assets/` or `templates/` for reusable artifacts
"""


def title_case_skill_name(skill_name: str) -> str:
    return " ".join(word.capitalize() for word in skill_name.split("-"))


def init_skill(skill_name: str, path: str | Path) -> Path | None:
    skill_dir = Path(path).resolve() / skill_name

    if skill_dir.exists():
        print(f"❌ Error: Skill directory already exists: {skill_dir}")
        return None

    try:
        skill_dir.mkdir(parents=True, exist_ok=False)
        (skill_dir / "references").mkdir()
        (skill_dir / "scripts").mkdir()
        (skill_dir / "assets").mkdir()
    except Exception as exc:
        print(f"❌ Error creating directory structure: {exc}")
        return None

    skill_title = title_case_skill_name(skill_name)
    (skill_dir / "SKILL.md").write_text(SKILL_TEMPLATE.format(skill_name=skill_name, skill_title=skill_title))
    print(f"✅ Initialized skill at: {skill_dir}")
    return skill_dir


def main() -> None:
    if len(sys.argv) < 4 or sys.argv[2] != "--path":
        print("Usage: python3 tools/skill/init_skill.py <skill-name> --path <path>")
        sys.exit(1)

    skill_name = sys.argv[1]
    path = sys.argv[3]

    result = init_skill(skill_name, path)
    sys.exit(0 if result else 1)


if __name__ == "__main__":
    main()

