#!/usr/bin/env python3
"""Generate bounded Codex subagent fanout plans."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python < 3.11 fallback.
    tomllib = None


REPO_ROOT = Path(__file__).resolve().parents[3]
DEFAULT_TEMPLATE_DIRS = [
    REPO_ROOT / "skills/deep-researcher/templates/agents",
    REPO_ROOT / "skills/subagent-creator/templates/agents",
]
BUILT_IN_ROLES = {
    "default": "Codex built-in default agent.",
    "explorer": "Codex built-in read-only codebase explorer.",
    "worker": "Codex built-in execution and implementation worker.",
}
BUILT_IN_NAMES = frozenset(BUILT_IN_ROLES)
NAME_RE = re.compile(r"^[a-z][a-z0-9_]*$")
RETURN_HEADINGS = (
    "- Status",
    "- Evidence",
    "- Files inspected/changed",
    "- Commands run",
    "- Findings",
    "- Risks/blockers",
)
TEMPLATE_REQUIRED_SNIPPETS = (
    "Return format:",
    "- Status",
    "- Risks/blockers",
)
SYNTHESIS_CHECKLIST = (
    "Wait for every spawned subagent in the planned batch.",
    "Record each agent as completed, failed, timed out, or closed.",
    "Merge overlapping findings and drop duplicates.",
    "Surface conflicts with evidence from each side.",
    "Resolve disagreements in the parent synthesis before next work.",
    "Run or identify verification before acting on proposed changes.",
)
PRESETS = {
    "research": (
        "openai_docs_researcher",
        "github_researcher",
        "citation_auditor",
    ),
    "dependency": (
        "context7_researcher",
        "source_validator",
        "github_researcher",
    ),
    "review": (
        "reviewer",
        "false_positive_validator",
        "test_runner",
    ),
    "implementation": (
        "repo_explorer",
        "implementation_worker",
        "test_runner",
    ),
    "docs": (
        "docs_researcher",
        "docs_auditor",
        "citation_auditor",
    ),
}


@dataclass(frozen=True)
class Role:
    """Resolved subagent role metadata."""

    name: str
    description: str
    model: str
    reasoning: str
    sandbox: str
    source: str
    path: str | None = None

    def to_dict(self) -> dict[str, str | None]:
        """Return JSON-serializable role metadata."""

        return {
            "name": self.name,
            "description": self.description,
            "model": self.model,
            "reasoning": self.reasoning,
            "sandbox": self.sandbox,
            "source": self.source,
            "path": self.path,
        }


@dataclass(frozen=True)
class Registry:
    """Role registry plus validation metadata."""

    roles: dict[str, Role]
    duplicates: dict[str, list[str]]
    issues: list[str]


def emit_json(data: Any) -> None:
    """Print stable JSON output."""

    print(json.dumps(data, indent=2, sort_keys=True))


def split_csv(values: list[str]) -> list[str]:
    """Split repeated comma-separated option values."""

    out: list[str] = []
    for value in values:
        out.extend(part.strip() for part in value.split(",") if part.strip())
    return out


def read_toml(path: Path) -> dict[str, Any]:
    """Read a TOML file as a dictionary."""

    if tomllib is None:
        raise SystemExit("Python 3.11+ is required for TOML parsing")
    with path.open("rb") as handle:
        return tomllib.load(handle)


def template_dirs(paths: list[str]) -> list[Path]:
    """Resolve template directory arguments."""

    if paths:
        return [Path(path).expanduser().resolve() for path in paths]
    return DEFAULT_TEMPLATE_DIRS


def built_in_roles() -> dict[str, Role]:
    """Return metadata for built-in Codex agent roles."""

    return {
        name: Role(
            name=name,
            description=description,
            model="inherited",
            reasoning="inherited",
            sandbox="runtime default",
            source="built-in",
            path=None,
        )
        for name, description in BUILT_IN_ROLES.items()
    }


def role_from_template(path: Path) -> tuple[Role | None, list[str]]:
    """Load one custom role template."""

    issues: list[str] = []
    try:
        data = read_toml(path)
    except tomllib.TOMLDecodeError as exc:
        return None, [f"{path}: invalid TOML: {exc}"]

    name = str(data.get("name", "")).strip()
    if not name:
        issues.append(f"{path}: missing name")
        name = path.stem
    if name != path.stem:
        issues.append(f"{path}: name does not match filename stem")
    if not NAME_RE.match(name):
        issues.append(f"{path}: name must be snake_case")
    if name in BUILT_IN_NAMES:
        issues.append(f"{path}: custom role shadows built-in role")

    instructions = str(data.get("developer_instructions", ""))
    for snippet in TEMPLATE_REQUIRED_SNIPPETS:
        if snippet not in instructions:
            issues.append(f"{path}: missing return contract snippet {snippet}")

    role = Role(
        name=name,
        description=str(data.get("description", "")).strip(),
        model=str(data.get("model", "inherited")).strip(),
        reasoning=str(data.get("model_reasoning_effort", "inherited")).strip(),
        sandbox=str(data.get("sandbox_mode", "runtime default")).strip(),
        source="template",
        path=str(path),
    )
    return role, issues


def load_registry(paths: list[str]) -> Registry:
    """Load built-in and custom role metadata."""

    roles = built_in_roles()
    duplicates: dict[str, list[str]] = {}
    issues: list[str] = []

    for directory in template_dirs(paths):
        if not directory.exists():
            issues.append(f"{directory}: template directory missing")
            continue
        if not directory.is_dir():
            issues.append(f"{directory}: template path is not a directory")
            continue
        for path in sorted(directory.glob("*.toml")):
            if not path.is_file():
                continue
            role, role_issues = role_from_template(path)
            issues.extend(role_issues)
            if role is None:
                continue
            if role.name in roles:
                duplicates.setdefault(role.name, [roles[role.name].path or ""])
                duplicates[role.name].append(str(path))
                continue
            roles[role.name] = role

    return Registry(roles=roles, duplicates=duplicates, issues=issues)


def selected_roles(args: argparse.Namespace, registry: Registry) -> list[Role]:
    """Resolve requested roles from CLI arguments or presets."""

    names = split_csv(args.role)
    if not names:
        if args.preset == "custom":
            raise SystemExit("pass --role or select a non-custom --preset")
        names = list(PRESETS[args.preset])

    if len(names) > args.max_agents and not args.allow_large_batch:
        raise SystemExit(
            f"selected {len(names)} roles; max is {args.max_agents}"
        )

    unknown = [name for name in names if name not in registry.roles]
    if unknown:
        available = ", ".join(sorted(registry.roles))
        raise SystemExit(
            f"unknown role(s): {', '.join(unknown)}. Available: {available}"
        )

    return [registry.roles[name] for name in names]


def scope_text(scopes: list[str]) -> str:
    """Return a stable scope string for generated prompts."""

    if not scopes:
        return "User-provided context only; do not broaden scope."
    return "; ".join(scopes)


def wait_text(wait_policy: str) -> str:
    """Return the prompt wait instruction."""

    if wait_policy == "strict":
        return "parent will wait for all spawned agents before substantive next work"
    return "user explicitly requested asynchronous delegation"


def mode_text(mode: str, scope: str) -> str:
    """Return mode-specific prompt text."""

    if mode == "read-only":
        return "read-only; do not edit files, stage changes, or commit"
    return f"may edit only named owned files or surfaces: {scope}"


def build_prompt(
    *,
    task: str,
    scope: str,
    mode: str,
    wait_policy: str,
    role: Role,
) -> str:
    """Build one mandatory spawn contract prompt."""

    lines = [
        f"Task: {task}",
        f"Scope: {scope}",
        f"Mode: {mode_text(mode, scope)}.",
        f"Wait: {wait_text(wait_policy)}.",
        f"Role: {role.name}.",
        f"Model: {role.model}.",
        f"Reasoning: {role.reasoning}.",
    ]
    if mode == "edit":
        lines.extend(
            [
                "You are not alone in the codebase.",
                "Do not revert edits made by others.",
                "Adjust your implementation for concurrent changes.",
            ]
        )
    lines.append("Return format:")
    lines.extend(RETURN_HEADINGS)
    return "\n".join(lines)


def build_plan(args: argparse.Namespace) -> dict[str, Any]:
    """Build a subspawn orchestration plan."""

    registry = load_registry(args.template_dir)
    roles = selected_roles(args, registry)
    scope = scope_text(args.scope)
    prompts = [
        {
            "role": role.name,
            "prompt": build_prompt(
                task=args.task,
                scope=scope,
                mode=args.mode,
                wait_policy=args.wait_policy,
                role=role,
            ),
        }
        for role in roles
    ]
    return {
        "task": args.task,
        "mode": args.mode,
        "scope": scope,
        "scope_items": args.scope,
        "wait_policy": args.wait_policy,
        "rendezvous_required": args.wait_policy == "strict",
        "roles": [role.to_dict() for role in roles],
        "prompts": prompts,
        "synthesis_checklist": list(SYNTHESIS_CHECKLIST),
        "registry_issues": registry.issues,
        "duplicate_roles_ignored": registry.duplicates,
    }


def print_plan(plan: dict[str, Any]) -> None:
    """Print a human-readable subspawn plan."""

    print("Subspawn plan")
    print(f"Task: {plan['task']}")
    print(f"Mode: {plan['mode']}")
    print(f"Wait policy: {plan['wait_policy']}")
    print()
    print("Roles:")
    for role in plan["roles"]:
        print(
            f"- {role['name']}: "
            f"model={role['model']}, "
            f"effort={role['reasoning']}, "
            f"sandbox={role['sandbox']}"
        )
    if plan["registry_issues"]:
        print()
        print("Registry issues:")
        for issue in plan["registry_issues"]:
            print(f"- {issue}")
    if plan["duplicate_roles_ignored"]:
        print()
        print("Duplicate role templates ignored:")
        for name, paths in plan["duplicate_roles_ignored"].items():
            print(f"- {name}: {', '.join(path for path in paths if path)}")
    print()
    print("Spawn prompts:")
    for item in plan["prompts"]:
        print(f"\n--- {item['role']} ---")
        print(item["prompt"])
    print()
    print("Synthesis checklist:")
    for item in plan["synthesis_checklist"]:
        print(f"- {item}")


def cmd_plan(args: argparse.Namespace) -> int:
    """Handle the plan command."""

    plan = build_plan(args)
    if args.json:
        emit_json(plan)
    else:
        print_plan(plan)
    return 1 if plan["registry_issues"] else 0


def cmd_validate_roles(args: argparse.Namespace) -> int:
    """Validate role names and stable return contracts."""

    registry = load_registry(args.template_dir)
    report = {
        "roles": [role.to_dict() for role in registry.roles.values()],
        "issues": registry.issues,
        "duplicate_roles_ignored": registry.duplicates,
    }
    if args.json:
        emit_json(report)
    else:
        for role in sorted(registry.roles.values(), key=lambda item: item.name):
            print(f"{role.name}: {role.source}")
        if registry.duplicates:
            print("duplicate role templates ignored:")
            for name, paths in registry.duplicates.items():
                print(f"- {name}: {', '.join(path for path in paths if path)}")
        for issue in registry.issues:
            print(f"ERROR: {issue}", file=sys.stderr)
    return 1 if registry.issues else 0


def cmd_list_presets(args: argparse.Namespace) -> int:
    """List available orchestration presets."""

    report = {"presets": PRESETS}
    if args.json:
        emit_json(report)
    else:
        for name, roles in PRESETS.items():
            print(f"{name}: {', '.join(roles)}")
    return 0


def add_registry_args(parser: argparse.ArgumentParser) -> None:
    """Add shared role-registry options to a parser."""

    parser.add_argument(
        "--template-dir",
        action="append",
        default=[],
        help="custom role template directory; can be repeated",
    )


def build_parser() -> argparse.ArgumentParser:
    """Build the command-line parser."""

    parser = argparse.ArgumentParser(
        description="Generate bounded Codex subagent orchestration plans.",
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    plan = subparsers.add_parser("plan", help="Generate a subspawn plan")
    plan.add_argument("--task", required=True)
    plan.add_argument("--scope", action="append", default=[])
    plan.add_argument(
        "--role",
        action="append",
        default=[],
        help="role name or comma-separated role names; can be repeated",
    )
    plan.add_argument(
        "--preset",
        choices=("custom", *sorted(PRESETS)),
        default="custom",
    )
    plan.add_argument(
        "--mode",
        choices=("read-only", "edit"),
        default="read-only",
    )
    plan.add_argument(
        "--wait-policy",
        choices=("strict", "async"),
        default="strict",
    )
    plan.add_argument("--max-agents", type=int, default=3)
    plan.add_argument("--allow-large-batch", action="store_true")
    plan.add_argument("--json", action="store_true")
    add_registry_args(plan)
    plan.set_defaults(func=cmd_plan)

    validate = subparsers.add_parser(
        "validate-roles",
        help="Validate role names and return contracts",
    )
    validate.add_argument("--json", action="store_true")
    add_registry_args(validate)
    validate.set_defaults(func=cmd_validate_roles)

    presets = subparsers.add_parser("list-presets", help="List plan presets")
    presets.add_argument("--json", action="store_true")
    presets.set_defaults(func=cmd_list_presets)

    return parser


def main(argv: list[str] | None = None) -> int:
    """Run the subspawn plan CLI."""

    parser = build_parser()
    args = parser.parse_args(argv)
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
