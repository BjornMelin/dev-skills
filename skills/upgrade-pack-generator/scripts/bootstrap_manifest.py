#!/usr/bin/env python3
"""Bootstrap an upgrade-pack manifest from repo context and optional family overrides."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

from common import (
    available_override_paths,
    detect_repo_context,
    dump_yaml,
    load_yaml,
    normalize_slug,
    recursive_merge,
    repo_path,
    titleize_package,
    unique_list,
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", required=True, help="Path to the target repo root.")
    parser.add_argument("--anchor-package", required=True, help="Primary package driving the pack.")
    parser.add_argument("--out", required=True, help="Path to write upgrade-pack.yaml.")
    parser.add_argument("--family-slug", help="Optional explicit family slug override.")
    return parser


def match_override(anchor_package: str, script_path: Path) -> dict[str, Any] | None:
    """Find a matching family override for an anchor package."""
    for path in available_override_paths(script_path):
        data = load_yaml(path) or {}
        candidates = set(data.get("match_packages") or [])
        if anchor_package in candidates:
            return data
    return None


def generic_manifest(anchor_package: str, repo_context: dict[str, Any], family_slug: str) -> dict[str, Any]:
    """Build a generic manifest when no family override exists."""
    display = titleize_package(anchor_package)
    plan_basename = f"{family_slug}-upgrade"
    return {
        "schema_version": 2,
        "family_slug": family_slug,
        "family_display_name": display,
        "family_type": "package",
        "mode": "upgrade",
        "plan_basename": plan_basename,
        "playbook_title": f"{display} Upgrade Playbook",
        "operator_title": f"{display} Upgrade Operator Mode",
        "trigger_title": f"{display} Upgrade Trigger Prompt",
        "anchor_package": anchor_package,
        "related_packages": [anchor_package],
        "repo_context": repo_context,
        "target_surface": {
            "surface_type": "repo-root",
            "workspace_path": ".",
            "workspace_name": "(root)",
            "workspace_package_json": "package.json",
            "workspace_slug": "root",
            "owner_reason": "Default root-scoped package family until enrichment proves a narrower owner workspace.",
            "related_workspaces": ["."],
            "verification_strategy": "root-only",
        },
        "qualification_plan": {
            "strategy": "separate-read-only-qualification",
            "snapshot_filename": "qualification-snapshot.json",
            "doc_urls": {},
            "source_specs": [],
            "cli_checks": [],
        },
        "current_version": "unknown",
        "validated_upstream_version": "unverified",
        "validated_doc_date": "unverified",
        "repo_probes": {
            "Repo posture": [
                "Run the family-specific enrichment step to record framework or package probes.",
            ],
        },
        "upstream_validation": {
            "Official guidance": [
                "Record the current official docs, version, and last-updated date during enrichment.",
            ],
        },
        "framework_constraints": [
            "Record family-specific constraints during enrichment before broad edits begin.",
        ],
        "supported_features": [
            "Record family-specific supported features during enrichment.",
        ],
        "unsupported_features": [
            "Record family-specific unsupported or intentionally out-of-scope features during enrichment.",
        ],
        "codemod_recommendations": [
            "Record family-specific codemods or automated migration helpers during enrichment.",
        ],
        "purpose": (
            f"Use this playbook to fully explore, research, plan, implement, verify, and "
            f"document a `{anchor_package}` upgrade in this repository."
        ),
        "use_when": [
            f"the repository already uses `{anchor_package}`",
            "the package likely has upgrade-specific cleanup or modernization work",
            "you need one repo-local artifact for research, decisions, execution, and closeout",
        ],
        "primary_goal": [
            f"upgrade `{anchor_package}` to the latest supportable version",
            "adopt package-native APIs where they simplify local code",
            "leave current docs, verification evidence, and a reviewable execution path",
        ],
        "non_goals": [
            "unrelated framework rewrites",
            "folding adjacent dependency waves into scope without evidence",
            "adding compatibility layers that are not justified by a real boundary",
        ],
        "primary_persona": "Codex acting as an architect-level modernization engineer.",
        "secondary_audience": "Human maintainers reviewing the migration, risks, and completion evidence.",
        "operating_goals": [
            "research before editing",
            "prefer official docs and upstream source over assumption",
            "keep one canonical implementation path",
            "delete stale wrappers, shims, and dead dependencies when justified",
            "keep a concise ledger of findings, decisions, completed work, and residual risks",
        ],
        "source_hierarchy": [
            f"official `{anchor_package}` docs and migration guidance",
            "official framework docs for the target repo",
            f"upstream `{anchor_package}` source code",
            "local repo code, tests, and docs",
            "package manager metadata, lockfiles, audits, and release notes",
        ],
        "default_final_decisions": [
            "prefer current package-native APIs over stale local abstractions",
            "use one hard-cut wave unless repo-specific evidence requires otherwise",
            f"keep scope centered on `{anchor_package}` and directly related packages",
        ],
        "intake_checklist": [
            "confirm framework and version, especially the package's host framework",
            f"record the current `{anchor_package}` version",
            "record package manager and lockfile shape",
            f"search all `{anchor_package}` usage",
            "search repo docs, AGENTS, and contributing guides for local policy",
            "build a touched-files map before editing",
        ],
        "required_research": {
            f"Upstream {display}": [
                "verify latest stable guidance",
                "verify migration notes, deprecations, and release constraints",
                "verify current exported types, APIs, and package-native capabilities",
            ],
            "Upstream framework": [
                "verify host-framework integration guidance that affects this package",
                "verify whether any framework-native optimization or config matters",
            ],
            "Upstream source inspection": [
                "inspect source when docs, types, or changelogs do not answer behavior or migration risk questions",
            ],
        },
        "questions_to_resolve": [
            f"what is the canonical final usage shape for `{anchor_package}` in this repo",
            "which related packages or peers are truly part of the same upgrade wave",
            "which local wrappers or abstractions no longer earn their keep after the upgrade",
            "which docs or policy files must move with the code change",
        ],
        "canonical_end_state": [
            f"all live `{anchor_package}` usage follows current verified guidance",
            "obsolete wrappers, shims, or compatibility paths are removed when no real boundary requires them",
            "docs and repo policy describe the final package usage shape",
            "lint, typecheck, tests, build, and audit remain green",
        ],
        "what_to_adopt": [
            "current package-native APIs and types",
            "current framework-native integration points",
            "existing local styling and architectural patterns unless the package now offers a simpler canonical path",
        ],
        "what_to_avoid": [
            "stale package APIs or deprecated imports",
            "mixed old and new conventions left in parallel",
            "local shims kept only to preserve a stale internal policy",
        ],
        "execution_plan": {
            "Phase 1: Discovery": [
                f"inventory all `{anchor_package}` usage",
                "inventory related packages and wrappers",
                "inventory docs and policy references that mention the package",
            ],
            "Phase 2: Decisions": [
                "lock the canonical final package usage shape",
                "lock scope for related packages and defers",
                "lock rollout style and boundary exceptions",
            ],
            "Phase 3: Code Changes": [
                "apply the current package-native usage shape",
                "remove obsolete wrappers, shims, and redundant dependencies",
                "update docs and repo policy to match the final state",
            ],
            "Phase 4: Verification": [
                "run repo-native validation",
                "prove removed patterns and dependencies are gone",
                "write the final upgrade report and residual risks",
            ],
        },
        "verification_commands": [
            "# inventory",
            f"rg -n '{anchor_package}' .",
            "",
            "# verification",
            "${PM_RUN} lint",
            "${PM_RUN} typecheck",
            "${PM_TEST}",
            "${PM_RUN} build",
            "${PM_AUDIT}",
        ],
        "report_heading": "Upgrade Report Requirements",
        "report_requirements": [
            "current starting state",
            "final decisions",
            "features and capabilities available in the current target version",
            "old patterns removed or discouraged",
            "deprecations and practical replacements",
            "repo-specific findings and residual risks",
        ],
        "deliverables": [
            "findings matrix",
            "decision log",
            "affected-files map",
            "migration checklist",
            "exact verification commands",
            "upgrade report",
            "residual risks or explicit defer reasons",
        ],
        "skill_routing_playbook": [
            "Always start in the upgrade lane: `$repo-modernize-upgrade-audit`, `$opensrc`, `$opensrc-inspect`, `$technical-writing`, and `$hard-cut`.",
            "Use official-doc lanes first for unstable claims: current upstream docs, changelogs, and source when needed.",
            "Use `$bun-dev` only when Bun is the repo's real runtime or package-manager lane.",
            "Use framework/plugin lanes only when the target repo actually detects them.",
            "Do not route this pack through `$imagegen` unless raster asset generation is explicitly part of the package-family surface.",
        ],
        "operator_defaults": [
            f"Final `{anchor_package}` usage follows current verified package-native guidance.",
            "One canonical implementation path.",
            "No stale compatibility shims unless a real external boundary requires them.",
        ],
        "operator_fast_intake": [
            "record framework and package version",
            "record package manager",
            f"grep for `{anchor_package}` usage and related packages",
            "find any AGENTS/docs that describe local package policy",
        ],
        "operator_research": [
            f"verify current `{anchor_package}` docs and migration guidance",
            "verify framework integration guidance",
            "inspect upstream source only if docs and types are insufficient",
        ],
        "operator_execute": [
            f"migrate all live `{anchor_package}` usage to the canonical final shape",
            "remove obsolete wrappers, shims, and redundant dependencies",
            "update docs/policy to match the final state",
        ],
        "operator_exit_criteria": [
            f"all live `{anchor_package}` usage follows the final verified shape",
            "obsolete compatibility code and dependencies are gone",
            "docs/policy match the final state",
            "verification is green",
        ],
        "skill_routing_operator": [
            "Default lane: `$repo-modernize-upgrade-audit`, `$opensrc`, `$opensrc-inspect`, `$technical-writing`, `$hard-cut`.",
            "Use `$bun-dev` only when Bun is actually part of repo posture.",
            "Use framework/plugin lanes only when explicitly relevant to the target repo.",
            "Do not route this migration through `$imagegen`.",
        ],
        "trigger_mission": f"Act as an architect-level modernization engineer performing a verification-first `{anchor_package}` upgrade in this repository.",
        "trigger_goals": [
            f"reach one canonical final end state for `{anchor_package}` usage",
            "delete stale wrappers, shims, docs, and unnecessary related deps when justified",
            "keep the diff reviewable and the final state simpler than the initial state",
        ],
        "trigger_required_research": [
            "inspect manifests, lockfiles, framework config, AGENTS/docs, and all package usage first",
            "verify current package and framework guidance before editing",
            "use official docs, upstream source, and repo-native commands",
            "build a touched-files map before broad edits",
        ],
        "trigger_required_decisions": [
            "confirm the canonical final usage shape in this repo",
            "decide which related packages are in or out of scope",
            "decide which local wrappers or abstractions can be deleted",
        ],
        "trigger_required_outcomes": [
            "standardize on the verified final package usage shape",
            "remove stale package usage patterns and unnecessary related dependencies",
            "update repo policy/docs to match the final convention",
        ],
        "trigger_required_deliverables": [
            "findings matrix",
            "decision log",
            "affected-files map",
            "migration checklist",
            "exact verification commands",
            "upgrade report",
            "residual risks or explicit defer reasons",
        ],
        "trigger_verification_expectation": [
            "run lint, typecheck, tests, build, and audit",
            "prove removed patterns and dependencies are gone with grep or equivalent checks",
        ],
    }


def finalize_manifest(manifest: dict[str, Any]) -> dict[str, Any]:
    """Fill derived fields after override merge."""
    basename = manifest["plan_basename"]
    manifest["playbook_filename"] = f"{basename}-playbook.md"
    manifest["trigger_filename"] = f"{basename}-trigger-prompt.md"
    manifest["operator_filename"] = f"{basename}-operator-mode.md"
    manifest["related_packages"] = unique_list(manifest["related_packages"])
    return manifest


def main() -> None:
    args = build_parser().parse_args()
    root = repo_path(args.repo_root)
    repo_context = detect_repo_context(root)
    override = match_override(args.anchor_package, Path(__file__))
    family_slug = (
        args.family_slug
        or (override or {}).get("family_slug")
        or normalize_slug(args.anchor_package)
    )

    manifest = generic_manifest(args.anchor_package, repo_context, family_slug)
    if override:
        manifest = recursive_merge(manifest, override)
        manifest["repo_context"] = repo_context
        for key in ("required_research", "execution_plan"):
            if key in override:
                manifest[key] = override[key]

    manifest = finalize_manifest(manifest)
    out_path = Path(args.out).expanduser().resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    dump_yaml(out_path, manifest)
    print(out_path)


if __name__ == "__main__":
    main()
