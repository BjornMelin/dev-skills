#!/usr/bin/env python3
"""Validate upgrade-pack.yaml before rendering."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

from common import load_yaml


REQUIRED_STRING_KEYS = {
    "family_display_name",
    "family_type",
    "mode",
    "family_slug",
    "plan_basename",
    "playbook_title",
    "operator_title",
    "trigger_title",
    "playbook_filename",
    "operator_filename",
    "trigger_filename",
    "anchor_package",
    "current_version",
    "validated_upstream_version",
    "validated_doc_date",
    "purpose",
    "primary_persona",
    "secondary_audience",
    "report_heading",
    "trigger_mission",
}

REQUIRED_LIST_KEYS = {
    "related_packages",
    "use_when",
    "primary_goal",
    "non_goals",
    "operating_goals",
    "source_hierarchy",
    "default_final_decisions",
    "intake_checklist",
    "questions_to_resolve",
    "canonical_end_state",
    "what_to_adopt",
    "what_to_avoid",
    "verification_commands",
    "framework_constraints",
    "supported_features",
    "unsupported_features",
    "codemod_recommendations",
    "report_requirements",
    "deliverables",
    "skill_routing_playbook",
    "operator_defaults",
    "operator_fast_intake",
    "operator_research",
    "operator_execute",
    "operator_exit_criteria",
    "skill_routing_operator",
    "trigger_goals",
    "trigger_required_research",
    "trigger_required_decisions",
    "trigger_required_outcomes",
    "trigger_required_deliverables",
    "trigger_verification_expectation",
}

REQUIRED_DICT_KEYS = {
    "repo_context",
    "target_surface",
    "repo_probes",
    "upstream_validation",
    "required_research",
    "execution_plan",
}


def _validate_string_list(name: str, value: Any, *, allow_empty: bool = False) -> list[str]:
    errors: list[str] = []
    if not isinstance(value, list) or not value:
        return [f"{name} must be a non-empty list"]
    if allow_empty:
        if not all(isinstance(item, str) for item in value):
            errors.append(f"{name} must contain only strings")
        return errors
    if not all(isinstance(item, str) and item.strip() for item in value):
        errors.append(f"{name} must contain only non-empty strings")
    return errors


def _validate_url_map(name: str, value: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(value, dict):
        return [f"{name} must be a dictionary"]
    for key, entry in value.items():
        if not isinstance(key, str) or not key.strip():
            errors.append(f"{name} keys must be non-empty strings")
        if not isinstance(entry, str) or not entry.strip():
            errors.append(f"{name}.{key} must be a non-empty string")
    return errors


def _validate_shell_checks(name: str, value: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(value, list):
        return [f"{name} must be a list"]
    for index, check in enumerate(value):
        if not isinstance(check, dict):
            errors.append(f"{name}[{index}] must be a dictionary")
            continue
        for key in ("label", "cwd", "command"):
            entry = check.get(key)
            if not isinstance(entry, str) or not entry.strip():
                errors.append(f"{name}[{index}].{key} must be a non-empty string")
    return errors


def validate_manifest(path: str | Path) -> tuple[bool, list[str]]:
    """Validate a manifest path."""
    manifest = load_yaml(path)
    if not isinstance(manifest, dict):
        return False, ["manifest root must be a YAML dictionary"]

    errors: list[str] = []

    schema_version = manifest.get("schema_version")
    if schema_version not in {1, 2, 3}:
        errors.append("schema_version must equal 1, 2, or 3")

    for key in REQUIRED_STRING_KEYS:
        value = manifest.get(key)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"{key} must be a non-empty string")

    for key in REQUIRED_LIST_KEYS:
        allow_empty = key == "verification_commands"
        errors.extend(_validate_string_list(key, manifest.get(key), allow_empty=allow_empty))

    for key in REQUIRED_DICT_KEYS:
        value = manifest.get(key)
        if not isinstance(value, dict) or not value:
            errors.append(f"{key} must be a non-empty dictionary")

    repo_context = manifest.get("repo_context") or {}
    for key in ("repo_root", "package_manager", "detected_by"):
        value = repo_context.get(key)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"repo_context.{key} must be a non-empty string")

    target_surface = manifest.get("target_surface") or {}
    for key in (
        "surface_type",
        "workspace_path",
        "workspace_name",
        "workspace_package_json",
        "workspace_slug",
        "owner_reason",
        "verification_strategy",
    ):
        value = target_surface.get(key)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"target_surface.{key} must be a non-empty string")
    errors.extend(_validate_string_list("target_surface.related_workspaces", target_surface.get("related_workspaces")))

    qualification_plan = manifest.get("qualification_plan") or {}
    if schema_version in {2, 3}:
        for key in ("strategy", "snapshot_filename"):
            value = qualification_plan.get(key)
            if not isinstance(value, str) or not value.strip():
                errors.append(f"qualification_plan.{key} must be a non-empty string")
        errors.extend(_validate_url_map("qualification_plan.doc_urls", qualification_plan.get("doc_urls")))
        source_specs = qualification_plan.get("source_specs")
        if not isinstance(source_specs, list):
            errors.append("qualification_plan.source_specs must be a list")
        elif not all(isinstance(item, str) and item.strip() for item in source_specs):
            errors.append("qualification_plan.source_specs must contain only non-empty strings")
        errors.extend(_validate_shell_checks("qualification_plan.cli_checks", qualification_plan.get("cli_checks")))

    if schema_version == 3:
        research_plan = manifest.get("research_plan") or {}
        if not isinstance(research_plan, dict) or not research_plan:
            errors.append("research_plan must be a non-empty dictionary")
            research_plan = {}
        for key in (
            "strategy",
            "snapshot_filename",
            "bundle_filename",
            "web_findings_filename",
            "source_map_policy",
            "target_version_policy",
            "target_version",
            "compatibility_rationale",
            "release_range",
        ):
            value = research_plan.get(key)
            if not isinstance(value, str) or not value.strip():
                errors.append(f"research_plan.{key} must be a non-empty string")
        confidence_threshold = research_plan.get("identity_confidence_threshold")
        if not isinstance(confidence_threshold, (int, float)):
            errors.append("research_plan.identity_confidence_threshold must be a number")
        elif not 0 <= float(confidence_threshold) <= 1:
            errors.append("research_plan.identity_confidence_threshold must be between 0 and 1")
        errors.extend(_validate_string_list("research_plan.required_categories", research_plan.get("required_categories")))
        errors.extend(_validate_string_list("research_plan.source_priority", research_plan.get("source_priority")))
        errors.extend(
            _validate_string_list(
                "research_plan.required_web_confirmation_categories",
                research_plan.get("required_web_confirmation_categories"),
            )
        )
        for key in (
            "official_docs",
            "api_reference",
            "migration_guides",
            "release_history",
            "examples_cookbooks",
        ):
            errors.extend(_validate_url_map(f"research_plan.{key}", research_plan.get(key)))
        source_specs = research_plan.get("source_specs")
        if not isinstance(source_specs, list):
            errors.append("research_plan.source_specs must be a list")
        elif not all(isinstance(item, str) and item.strip() for item in source_specs):
            errors.append("research_plan.source_specs must contain only non-empty strings")
        errors.extend(_validate_shell_checks("research_plan.repo_usage_queries", research_plan.get("repo_usage_queries")))

    for key in ("repo_probes", "upstream_validation", "required_research", "execution_plan"):
        section = manifest.get(key) or {}
        if isinstance(section, dict):
            for name, items in section.items():
                if not isinstance(name, str) or not name.strip():
                    errors.append(f"{key} headings must be non-empty strings")
                errors.extend(_validate_string_list(f"{key}.{name}", items))

    return not errors, errors


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("manifest", help="Path to upgrade-pack.yaml")
    return parser


def main() -> None:
    args = build_parser().parse_args()
    valid, errors = validate_manifest(args.manifest)
    if valid:
        print("Upgrade pack manifest is valid.")
        return

    print("Upgrade pack manifest is invalid:")
    for error in errors:
        print(f"- {error}")
    raise SystemExit(1)


if __name__ == "__main__":
    main()
