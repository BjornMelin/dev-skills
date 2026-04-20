#!/usr/bin/env python3
"""Render a repo-local upgrade pack from a canonical upgrade-pack.yaml manifest."""

from __future__ import annotations

import argparse
import json
import re
import textwrap
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from common import load_yaml
from validate_upgrade_pack import validate_manifest

WRAP_WIDTH = 80
MARKDOWNLINT_PREAMBLE = "<!-- markdownlint-disable MD013 -->"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, help="Path to upgrade-pack.yaml")
    parser.add_argument("--output-dir", required=True, help="Directory to write the rendered pack into")
    parser.add_argument("--qualification-snapshot", help="Optional path to qualification-snapshot.json")
    return parser


def bullets(items: list[str]) -> str:
    wrapper = textwrap.TextWrapper(width=WRAP_WIDTH, subsequent_indent="  ")
    return "\n".join(wrapper.fill(f"- {item}") for item in items)


def numbered(items: list[str]) -> str:
    lines: list[str] = []
    for index, item in enumerate(items, start=1):
        prefix = f"{index}. "
        wrapper = textwrap.TextWrapper(width=WRAP_WIDTH, initial_indent=prefix, subsequent_indent=" " * len(prefix))
        lines.append(wrapper.fill(item))
    return "\n".join(lines)


def nested_sections(sections: dict[str, list[str]], level: int = 3) -> str:
    prefix = "#" * level
    chunks: list[str] = []
    for heading, items in sections.items():
        chunks.append(f"{prefix} {heading}\n")
        chunks.append(bullets(items))
        chunks.append("")
    return "\n".join(chunks).strip()


def fenced_block(lines: list[str]) -> str:
    return "```bash\n" + "\n".join(lines).rstrip() + "\n```"


def paragraph(text: str) -> str:
    return textwrap.fill(text, width=WRAP_WIDTH)


def trigger_bullet(text: str, *, indent: int = 0) -> str:
    """Wrap a trigger-prompt bullet line."""
    prefix = " " * indent + "- "
    return textwrap.fill(
        prefix + text,
        width=WRAP_WIDTH,
        initial_indent=prefix,
        subsequent_indent=" " * len(prefix),
    )


def package_manager_detection_block(manifest: dict[str, Any]) -> str:
    repo_context = manifest["repo_context"]
    variables = repo_context.get("command_variables", {})
    current = repo_context.get("package_manager", "unknown")
    detected_by = repo_context.get("detected_by", "unknown")
    lockfiles = ", ".join(repo_context.get("root_lockfiles") or []) or "(none)"
    hints = repo_context.get("docs_ci_hints") or {}
    hints_text = ", ".join(f"{key}:{value}" for key, value in hints.items()) or "(none)"
    items = [
        "Detect the repo command family before install, CLI, build, test, or audit runs.",
        "Prefer this order: `package.json#packageManager`, then lockfiles, then repo docs/CI as the tiebreaker.",
        f"Current repo detection: package manager=`{current}`, detected_by=`{detected_by}`, root_lockfiles=`{lockfiles}`, docs_ci_hints=`{hints_text}`.",
        "Use one consistent command family for the full run.",
        (
            "Current command variables: "
            f"`PM_DLX=\"{variables.get('PM_DLX', '<repo-native dlx command>')}\"`, "
            f"`PM_RUN=\"{variables.get('PM_RUN', '<repo-native run command>')}\"`, "
            f"`PM_TEST=\"{variables.get('PM_TEST', '<repo-native test command>')}\"`, "
            f"`PM_AUDIT=\"{variables.get('PM_AUDIT', '<repo-native audit command>')}\"`."
        ),
        (
            "Reference mapping for Bun repos: "
            '`PM_DLX="bunx"`; `PM_RUN="bun run"`; `PM_TEST="bun test"`; `PM_AUDIT="bun audit"`.'
        ),
        (
            "Reference mapping for pnpm repos: "
            '`PM_DLX="pnpm dlx"`; `PM_RUN="pnpm"`; `PM_TEST="pnpm test -- --run"`; '
            '`PM_AUDIT="pnpm audit --json"`.'
        ),
        (
            "Reference mapping for npm repos: "
            '`PM_DLX="npx"`; `PM_RUN="npm run"`; `PM_TEST="npm test -- --run"`; '
            '`PM_AUDIT="npm audit --json"`.'
        ),
        (
            "Reference mapping for Yarn repos: "
            '`PM_DLX="yarn dlx"`; `PM_RUN="yarn"`; `PM_TEST="yarn test --run"`; '
            '`PM_AUDIT="<repo-native yarn audit command>"`.'
        ),
    ]
    return bullets(items)


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def qualification_summary_block(snapshot: dict[str, Any] | None, manifest: dict[str, Any]) -> str:
    plan = manifest.get("qualification_plan") or {}
    if not snapshot:
        items = [
            "status: `pending`",
            f"snapshot file: `{plan.get('snapshot_filename', 'qualification-snapshot.json')}`",
            "qualification stage has not been run yet for this rendered pack",
        ]
        return bullets(items)

    summary = snapshot.get("summary") or {}
    items = [
        f"status: `{snapshot.get('qualification_status', 'unknown')}`",
        f"snapshot file: `{snapshot.get('snapshot_filename', plan.get('snapshot_filename', 'qualification-snapshot.json'))}`",
        f"doc checks: `{summary.get('doc_checks', 0)}` total, `{summary.get('doc_failures', 0)}` failed",
        f"source checks: `{summary.get('source_checks', 0)}` total, `{summary.get('source_failures', 0)}` failed",
        f"CLI checks: `{summary.get('cli_checks', 0)}` total, `{summary.get('cli_failures', 0)}` failed",
        f"repo-local overlays: `{summary.get('repo_local_overlays', 0)}` matched",
    ]
    caveats = snapshot.get("caveats") or []
    if caveats:
        items.append(f"caveats: `{'; '.join(str(item) for item in caveats[:3])}`")
    return bullets(items)


def repo_local_overlay_block(snapshot: dict[str, Any] | None) -> str:
    overlays = (snapshot or {}).get("repo_local_skill_overlays") or []
    if not overlays:
        return bullets(["no repo-local skill overlays detected for this family"])
    return bullets(
        [
            f"`{overlay.get('skill_name', 'unknown')}` at `{overlay.get('skill_path', 'unknown')}` -- {overlay.get('reason', 'matched by family overlay detection')}"
            for overlay in overlays
        ]
    )


def family_profile_block(manifest: dict[str, Any]) -> str:
    items = [
        f"display name: `{manifest['family_display_name']}`",
        f"family type: `{manifest['family_type']}`",
        f"mode: `{manifest['mode']}`",
        f"anchor package: `{manifest['anchor_package']}`",
        f"current repo version: `{manifest['current_version']}`",
        f"validated upstream version: `{manifest['validated_upstream_version']}`",
        f"validated doc date: `{manifest['validated_doc_date']}`",
    ]
    return bullets(items)


def target_surface_block(manifest: dict[str, Any]) -> str:
    target_surface = manifest["target_surface"]
    related = ", ".join(target_surface.get("related_workspaces") or []) or "(none)"
    items = [
        f"surface type: `{target_surface['surface_type']}`",
        f"owner workspace path: `{target_surface['workspace_path']}`",
        f"owner workspace name: `{target_surface['workspace_name']}`",
        f"owner workspace manifest: `{target_surface['workspace_package_json']}`",
        f"owner workspace slug: `{target_surface['workspace_slug']}`",
        f"related workspaces: `{related}`",
        f"verification strategy: `{target_surface['verification_strategy']}`",
        f"owner rationale: {target_surface['owner_reason']}",
    ]
    return bullets(items)


def embedded_tracker(manifest: dict[str, Any]) -> str:
    repo_context = manifest["repo_context"]
    frameworks = ", ".join(repo_context.get("frameworks_detected") or []) or "none"
    related = ", ".join(manifest["related_packages"])
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M:%SZ")

    seed_items = manifest["intake_checklist"] + [
        item for items in manifest["execution_plan"].values() for item in items
    ]
    ledger = bullets([f"[ ] {item}" for item in seed_items])

    return "\n".join(
        [
            "### Status Ledger",
            "",
            bullets(
                [
                    f"generated_at: `{timestamp}`",
                    f"family_slug: `{manifest['family_slug']}`",
                    f"anchor_package: `{manifest['anchor_package']}`",
                    f"related_packages: `{related}`",
                    f"repo_root: `{repo_context['repo_root']}`",
                    f"package_manager: `{repo_context['package_manager']}`",
                    f"frameworks_detected: `{frameworks}`",
                    f"mode: `{manifest['mode']}`",
                    "status: `not-started`",
                ]
            ),
            "",
            "### Repo Notes",
            "",
            "- [ ] Record repo-specific findings, assumptions, blockers, and explicit defers here.",
            "",
            "### Checkoff Ledger",
            "",
            ledger,
        ]
    )


def render_template(template_path: Path, replacements: dict[str, str]) -> str:
    text = template_path.read_text(encoding="utf-8")

    def replacer(match: re.Match[str]) -> str:
        key = match.group(1)
        return replacements[key]

    rendered = re.sub(r"\{\{([a-z0-9_]+)\}\}", replacer, text)
    return f"{MARKDOWNLINT_PREAMBLE}\n\n{rendered}"


def write_file(path: Path, content: str) -> None:
    path.write_text(content.rstrip() + "\n", encoding="utf-8")


def main() -> None:
    args = build_parser().parse_args()
    manifest_path = Path(args.manifest).expanduser().resolve()
    output_dir = Path(args.output_dir).expanduser().resolve()
    qualification_path = (
        Path(args.qualification_snapshot).expanduser().resolve()
        if args.qualification_snapshot
        else None
    )

    valid, errors = validate_manifest(manifest_path)
    if not valid:
        print("Manifest validation failed:")
        for error in errors:
            print(f"- {error}")
        raise SystemExit(1)

    manifest = load_yaml(manifest_path)
    qualification_plan = manifest.get("qualification_plan") or {}
    if qualification_path is None:
        default_snapshot = qualification_plan.get("snapshot_filename")
        if isinstance(default_snapshot, str) and default_snapshot.strip():
            candidate = manifest_path.parent / default_snapshot
            if candidate.exists():
                qualification_path = candidate
    qualification_snapshot = load_json(qualification_path) if qualification_path and qualification_path.exists() else None
    skill_root = Path(__file__).resolve().parents[1]
    templates = skill_root / "assets" / "templates"

    companion_files = [
        manifest["playbook_filename"],
        manifest["trigger_filename"],
        manifest["operator_filename"],
        "upgrade-pack.yaml",
    ]
    snapshot_filename = qualification_plan.get("snapshot_filename")
    if isinstance(snapshot_filename, str) and snapshot_filename.strip():
        companion_files.append(snapshot_filename)

    playbook_replacements = {
        "playbook_title": manifest["playbook_title"],
        "purpose": paragraph(manifest["purpose"]),
        "family_profile_block": family_profile_block(manifest),
        "target_surface_block": target_surface_block(manifest),
        "companion_files_block": bullets(companion_files),
        "qualification_summary_block": qualification_summary_block(qualification_snapshot, manifest),
        "repo_local_overlay_block": repo_local_overlay_block(qualification_snapshot),
        "use_when_block": bullets(manifest["use_when"]),
        "primary_goal_block": bullets(manifest["primary_goal"]),
        "non_goals_block": bullets(manifest["non_goals"]),
        "primary_persona": manifest["primary_persona"],
        "secondary_audience": manifest["secondary_audience"],
        "operating_goals_block": bullets(manifest["operating_goals"]),
        "source_hierarchy_block": numbered(manifest["source_hierarchy"]),
        "package_manager_detection_block": package_manager_detection_block(manifest),
        "repo_probes_block": nested_sections(manifest["repo_probes"], level=3),
        "upstream_validation_block": nested_sections(manifest["upstream_validation"], level=3),
        "framework_constraints_block": bullets(manifest["framework_constraints"]),
        "supported_features_block": bullets(manifest["supported_features"]),
        "unsupported_features_block": bullets(manifest["unsupported_features"]),
        "codemod_recommendations_block": bullets(manifest["codemod_recommendations"]),
        "skill_routing_playbook_block": bullets(manifest["skill_routing_playbook"]),
        "default_final_decisions_block": bullets(manifest["default_final_decisions"]),
        "intake_checklist_block": bullets([f"[ ] {item}" for item in manifest["intake_checklist"]]),
        "required_research_block": nested_sections(manifest["required_research"], level=3),
        "questions_to_resolve_block": bullets(manifest["questions_to_resolve"]),
        "canonical_end_state_block": bullets(manifest["canonical_end_state"]),
        "what_to_adopt_block": bullets(manifest["what_to_adopt"]),
        "what_to_avoid_block": bullets(manifest["what_to_avoid"]),
        "execution_plan_block": nested_sections(manifest["execution_plan"], level=3),
        "verification_commands_block": fenced_block(manifest["verification_commands"]),
        "report_heading": manifest["report_heading"],
        "report_requirements_block": bullets(manifest["report_requirements"]),
        "deliverables_block": bullets(manifest["deliverables"]),
        "embedded_tracker_block": embedded_tracker(manifest),
    }

    operator_replacements = {
        "operator_title": manifest["operator_title"],
        "family_profile_block": family_profile_block(manifest),
        "target_surface_block": target_surface_block(manifest),
        "qualification_summary_block": qualification_summary_block(qualification_snapshot, manifest),
        "repo_local_overlay_block": repo_local_overlay_block(qualification_snapshot),
        "framework_constraints_block": bullets(manifest["framework_constraints"]),
        "operator_defaults_block": bullets(manifest["operator_defaults"]),
        "operator_fast_intake_block": bullets([f"[ ] {item}" for item in manifest["operator_fast_intake"]]),
        "package_manager_detection_block": package_manager_detection_block(manifest),
        "operator_research_block": bullets([f"[ ] {item}" for item in manifest["operator_research"]]),
        "skill_routing_operator_block": bullets(manifest["skill_routing_operator"]),
        "operator_execute_block": bullets([f"[ ] {item}" for item in manifest["operator_execute"]]),
        "verification_commands_block": fenced_block(manifest["verification_commands"]),
        "operator_exit_criteria_block": bullets(manifest["operator_exit_criteria"]),
        "deliverables_block": bullets(manifest["deliverables"]),
    }

    trigger_lines = [
        paragraph(manifest["trigger_mission"]),
        "",
        "First step:",
        trigger_bullet(f"Read and load `./{manifest['playbook_filename']}` before doing any other work."),
        trigger_bullet("Treat that file as the source of truth for scope, workflow, verification, and closeout."),
        trigger_bullet("Use the `Embedded Tracker` section in that file as the live execution ledger."),
        trigger_bullet("As you work, keep the playbook updated in place:"),
        trigger_bullet("update `Status Ledger`", indent=2),
        trigger_bullet("append concise findings, assumptions, blockers, and deferred items under `Repo Notes`", indent=2),
        trigger_bullet("mark `Checkoff Ledger` items complete as they are actually completed", indent=2),
        trigger_bullet("leave the file in a final, accurate completed state before closing", indent=2),
        "",
        "Family profile:",
        family_profile_block(manifest),
        "",
        "Target surface:",
        target_surface_block(manifest),
        "",
        "Goals:",
        bullets(manifest["trigger_goals"]),
        "",
        "Required research:",
        bullets(manifest["trigger_required_research"]),
        "",
        "Required decisions:",
        bullets(manifest["trigger_required_decisions"]),
        "",
        "Required implementation outcomes:",
        bullets(manifest["trigger_required_outcomes"]),
        "",
        "Required deliverables:",
        bullets(manifest["trigger_required_deliverables"]),
        trigger_bullet(
            f"updated `./{manifest['playbook_filename']}` with final progress, notes, and completed checkoffs"
        ),
        "",
        "Verification expectation:",
        bullets(manifest["trigger_verification_expectation"]),
    ]
    trigger_text = "\n".join(trigger_lines).rstrip()
    trigger_replacements = {
        "trigger_title": manifest["trigger_title"],
        "trigger_code_block": f"```text\n{trigger_text}\n```",
    }

    output_dir.mkdir(parents=True, exist_ok=True)
    write_file(output_dir / "upgrade-pack.yaml", manifest_path.read_text(encoding="utf-8"))
    if qualification_snapshot and isinstance(snapshot_filename, str) and snapshot_filename.strip():
        write_file(
            output_dir / snapshot_filename,
            json.dumps(qualification_snapshot, indent=2, sort_keys=False),
        )
    write_file(
        output_dir / manifest["playbook_filename"],
        render_template(templates / "playbook.md.tmpl", playbook_replacements),
    )
    write_file(
        output_dir / manifest["operator_filename"],
        render_template(templates / "operator-mode.md.tmpl", operator_replacements),
    )
    write_file(
        output_dir / manifest["trigger_filename"],
        render_template(templates / "trigger-prompt.md.tmpl", trigger_replacements),
    )

    print(output_dir)


if __name__ == "__main__":
    main()
