#!/usr/bin/env python3
"""Validate TanStack-specific skill contracts and installed-copy parity."""
from __future__ import annotations

import argparse
import re
import sys
import tempfile
from collections.abc import Callable
from pathlib import Path

SKILLS = [
    "tanstack-start",
    "tanstack-router",
    "tanstack-query",
    "tanstack-integration",
]
AUTHORITY_PATH = Path("docs/reference/tanstack-current-authority.md")
STALE_PATTERNS = [
    ("legacy server-function validator", re.compile(r"\.inputValidator\s*\(")),
    (
        "old tanstack-start skill name",
        re.compile(r"tanstack-start-best-practices"),
    ),
    (
        "old tanstack-router skill name",
        re.compile(r"tanstack-router-best-practices"),
    ),
    (
        "old tanstack-query skill name",
        re.compile(r"tanstack-query-best-practices"),
    ),
    (
        "old tanstack-integration skill name",
        re.compile(r"tanstack-integration-best-practices"),
    ),
]
STALE_RUNTIME_PATTERNS = [
    ("legacy Start Vite plugin package", re.compile(r"@tanstack/start/plugin/vite")),
    ("deprecated Router Vite helper", re.compile(r"TanStackRouterVite")),
    (
        "old nested search serializer config",
        re.compile(r"search\s*:\s*\{\s*serialize"),
    ),
    (
        "removed TanStack MCP CLI guidance",
        re.compile(r"tanstack\s+mcp", re.IGNORECASE),
    ),
]


def fail(msg: str) -> None:
    """Print a validation failure and exit with a failing status."""
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(1)


def read(path: Path) -> str:
    """Read a required text file as UTF-8."""
    return path.read_text(encoding="utf-8")


def read_optional_text(path: Path) -> str | None:
    """Read a file as UTF-8, returning None for binary or invalid text."""
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return None


def load_quick_validator(root: Path) -> Callable[[str | Path], tuple[bool, str]]:
    """Load the repository's shared skill validator."""
    skill_tools = str(root / "tools" / "skill")
    if skill_tools not in sys.path:
        sys.path.insert(0, skill_tools)
    from quick_validate import validate_skill  # noqa: PLC0415

    return validate_skill


def markdown_section(text: str, heading: str) -> str:
    """Return the body of a second-level Markdown section."""
    marker = f"## {heading}"
    start = text.find(marker)
    if start == -1:
        return ""
    body_start = text.find("\n", start)
    if body_start == -1:
        return ""
    next_heading = text.find("\n## ", body_start + 1)
    if next_heading == -1:
        return text[body_start + 1 :]
    return text[body_start + 1 : next_heading]


def referenced_rules(skill_md: str) -> set[str]:
    """Extract rule file references from a skill's Rules section."""
    rules_section = markdown_section(skill_md, "Rules")
    if not rules_section:
        fail("SKILL.md is missing a ## Rules section")
    return set(re.findall(r"rules/([A-Za-z0-9_.-]+\.md)", rules_section))


def scan_text(
    label: str,
    text: str,
    patterns: list[tuple[str, re.Pattern[str]]],
    owner: str,
) -> None:
    """Fail when text contains any disallowed pattern."""
    for pattern_label, pattern in patterns:
        if pattern.search(text):
            fail(f"{label} contains {owner}: {pattern_label}")


def validate_rule_inventory(skill_dir: Path) -> None:
    """Ensure SKILL.md references every rule file exactly once."""
    skill_md = read(skill_dir / "SKILL.md")
    rules_dir = skill_dir / "rules"
    if not rules_dir.is_dir():
        fail(f"{skill_dir.name} missing rules directory")
    refs = referenced_rules(skill_md)
    files = {path.name for path in rules_dir.glob("*.md")}
    missing = refs - files
    unreferenced = files - refs
    if missing:
        fail(f"{skill_dir.name} references missing rules: {sorted(missing)}")
    if unreferenced:
        fail(f"{skill_dir.name} has unreferenced rules: {sorted(unreferenced)}")
    if not refs:
        fail(f"{skill_dir.name} references no rules")


def skill_text_files(skill_dir: Path) -> list[str]:
    """Return UTF-8 file contents below a skill directory."""
    texts: list[str] = []
    for path in skill_dir.rglob("*"):
        if not path.is_file():
            continue
        text = read_optional_text(path)
        if text is not None:
            texts.append(text)
    return texts


def validate_stale_guidance(skill_dir: Path) -> None:
    """Check skill text for stale TanStack guidance."""
    combined = "\n".join(skill_text_files(skill_dir))
    scan_text(skill_dir.name, combined, STALE_PATTERNS, "stale pattern")

    runtime_files = [
        skill_dir / "SKILL.md",
        skill_dir / "agents" / "openai.yaml",
        *sorted((skill_dir / "rules").glob("*.md")),
    ]
    runtime_text = "\n".join(read(path) for path in runtime_files if path.is_file())
    scan_text(
        skill_dir.name,
        runtime_text,
        STALE_RUNTIME_PATTERNS,
        "stale runtime guidance",
    )


def validate_authority_copy(root: Path, skill_dir: Path) -> None:
    """Ensure the packaged authority reference matches the canonical copy."""
    canonical = root / AUTHORITY_PATH
    copy = skill_dir / "references" / "current-authority.md"
    if not canonical.is_file():
        fail(f"missing canonical TanStack authority ledger: {canonical}")
    if not copy.is_file():
        fail(f"{skill_dir.name} missing packaged authority ledger copy: {copy}")
    if canonical.read_bytes() != copy.read_bytes():
        fail(f"{skill_dir.name} authority ledger copy drifted from {canonical}")


def validate_skill(
    root: Path,
    skill_dir: Path,
    quick_validate: Callable[[str | Path], tuple[bool, str]],
) -> None:
    """Run shared and TanStack-specific checks for one skill."""
    if not skill_dir.is_dir():
        fail(f"missing skill directory: {skill_dir}")
    valid, message = quick_validate(skill_dir)
    if not valid:
        fail(f"{skill_dir.name} quick validation failed: {message}")
    validate_rule_inventory(skill_dir)
    validate_authority_copy(root, skill_dir)
    validate_stale_guidance(skill_dir)


def compare_dirs(src: Path, dst: Path) -> list[str]:
    """Return installed-copy parity errors between source and destination."""
    errors: list[str] = []
    src_files = {path.relative_to(src) for path in src.rglob("*") if path.is_file()}
    dst_files = (
        {path.relative_to(dst) for path in dst.rglob("*") if path.is_file()}
        if dst.exists()
        else set()
    )
    for rel in sorted(src_files - dst_files):
        errors.append(f"missing installed file {dst / rel}")
    for rel in sorted(dst_files - src_files):
        errors.append(f"extra installed file {dst / rel}")
    for rel in sorted(src_files & dst_files):
        if (src / rel).read_bytes() != (dst / rel).read_bytes():
            errors.append(f"installed file differs: {dst / rel}")
    return errors


def run_self_test() -> None:
    """Exercise checker pattern and Markdown section helpers."""
    stale_examples = {
        "legacy server-function validator": ".inputValidator (value)",
        "old tanstack-start skill name": "tanstack-start-best-practices",
        "old tanstack-router skill name": "tanstack-router-best-practices",
        "old tanstack-query skill name": "tanstack-query-best-practices",
        "old tanstack-integration skill name": "tanstack-integration-best-practices",
    }
    runtime_examples = {
        "legacy Start Vite plugin package": "@tanstack/start/plugin/vite",
        "deprecated Router Vite helper": "TanStackRouterVite()",
        "old nested search serializer config": "search : { serialize }",
        "removed TanStack MCP CLI guidance": "TanStack MCP",
    }
    for label, example in stale_examples.items():
        if not dict(STALE_PATTERNS)[label].search(example):
            fail(f"self-test stale pattern did not match {label!r}")
    for label, example in runtime_examples.items():
        if not dict(STALE_RUNTIME_PATTERNS)[label].search(example):
            fail(f"self-test runtime pattern did not match {label!r}")
    sample = "## Rules\n\n- Read `rules/a.md`.\n\n## Notes\n\n- Read `rules/b.md`."
    if referenced_rules(sample) != {"a.md"}:
        fail("self-test rule-section extraction failed")
    with tempfile.TemporaryDirectory() as tmp:
        skill_dir = Path(tmp)
        (skill_dir / "text.md").write_text("current guidance", encoding="utf-8")
        (skill_dir / "binary.bin").write_bytes(b"\xff\xfe\xfd")
        if skill_text_files(skill_dir) != ["current guidance"]:
            fail("self-test binary file filtering failed")


def main() -> int:
    """Parse CLI arguments and run the requested TanStack checks."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run internal checker self-tests",
    )
    parser.add_argument(
        "--root",
        type=Path,
        default=Path.cwd(),
        help="dev-skills root",
    )
    parser.add_argument(
        "--skill",
        type=Path,
        action="append",
        help="skill path to validate instead of all TanStack skills",
    )
    parser.add_argument(
        "--installed-root",
        type=Path,
        help="optional installed .agents/skills root for parity",
    )
    args = parser.parse_args()

    if args.self_test:
        run_self_test()
        print("TanStack checker self-test passed")
        return 0

    quick_validate = load_quick_validator(args.root)
    paths = args.skill or [args.root / "skills" / name for name in SKILLS]
    for path in paths:
        validate_skill(args.root, path, quick_validate)

    if args.installed_root:
        for name in SKILLS:
            src = args.root / "skills" / name
            dst = args.installed_root / name
            for err in compare_dirs(src, dst):
                fail(err)
            old = args.installed_root / f"{name}-best-practices"
            if old.exists():
                fail(f"old installed skill still exists: {old}")

    print("TanStack skills are valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
