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


def raw_bullets(items: list[str]) -> str:
    """Render bullets without wrapping, for path- and link-sensitive lines."""
    return "\n".join(f"- {item}" for item in items)


def raw_numbered(items: list[str]) -> str:
    """Render a numbered list without wrapping, for path- and link-sensitive lines."""
    return "\n".join(f"{index}. {item}" for index, item in enumerate(items, start=1))


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


def labeled_bullets(pairs: list[tuple[str, str]]) -> str:
    """Render bullets with a stable `label: value` pattern."""
    items = [f"{label}: {value}" for label, value in pairs]
    return bullets(items)


def markdown_link(target: str, label: str) -> str:
    """Render a relative markdown link."""
    return f"[{label}]({target})"


def playbook_anchor_link(manifest: dict[str, Any], anchor: str, label: str) -> str:
    """Build a markdown link to a heading within the rendered playbook."""
    return markdown_link(f"./{manifest['playbook_filename']}#{anchor}", label)


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


def trigger_summary_block(manifest: dict[str, Any], snapshot: dict[str, Any] | None) -> str:
    """Render the compact identity + surface summary kept in the trigger prompt."""
    target_surface = manifest["target_surface"]
    qualification_status = (snapshot or {}).get("qualification_status", "pending")
    return labeled_bullets(
        [
            ("family", f"`{manifest['family_display_name']}`"),
            ("anchor package", f"`{manifest['anchor_package']}`"),
            ("current repo version", f"`{manifest['current_version']}`"),
            ("validated upstream version", f"`{manifest['validated_upstream_version']}`"),
            ("owner surface", f"`{target_surface['workspace_path']}` (`{target_surface['surface_type']}`)"),
            ("verification strategy", f"`{target_surface['verification_strategy']}`"),
            ("qualification status", f"`{qualification_status}`"),
        ]
    )


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


def pack_map_block(manifest: dict[str, Any]) -> str:
    """Render file roles, reading order, and writable-file rules for the pack."""
    snapshot_filename = (
        manifest.get("qualification_plan", {}).get("snapshot_filename") or "qualification-snapshot.json"
    )
    file_roles = bullets(
        [
            f"`{manifest['playbook_filename']}` -- authoritative handoff doc and the only writable pack file during implementation.",
            f"`{manifest['operator_filename']}` -- execution delta card for fast runs; if it conflicts with the playbook, the playbook wins.",
            f"`{manifest['trigger_filename']}` -- copy/paste launcher for a fresh Codex session.",
            "`upgrade-pack.yaml` -- canonical structured source for the rendered pack.",
            f"`{snapshot_filename}` -- machine-readable qualification evidence from the read-only qualify stage.",
        ]
    )
    reading_order = raw_numbered(
        [
            f"Start with `./{manifest['trigger_filename']}` only when launching a new Codex session.",
            f"Read {playbook_anchor_link(manifest, 'pack-map', 'Pack Map')} and {playbook_anchor_link(manifest, 'current-state-and-evidence', 'Current State And Evidence')} in `./{manifest['playbook_filename']}`.",
            f"Use {playbook_anchor_link(manifest, 'decisions-and-end-state', 'Decisions And End State')} before editing and {playbook_anchor_link(manifest, 'execution-and-verification', 'Execution And Verification')} while implementing.",
            f"Use `./{manifest['operator_filename']}` only as a quick execution aid after the playbook is loaded.",
            f"Consult `upgrade-pack.yaml` and `{snapshot_filename}` when raw structured evidence or exact qualification details are needed.",
        ]
    )
    writable_file = raw_bullets(
        [
            f"Writable during implementation: `{manifest['playbook_filename']}` only.",
            f"Update {playbook_anchor_link(manifest, 'live-tracker-and-closeout', 'Live Tracker And Closeout')} in place as work progresses.",
            "Do not hand-edit the operator, trigger, manifest, or qualification snapshot during implementation unless you are intentionally regenerating the pack.",
        ]
    )
    return "\n".join(
        [
            "### File Roles",
            "",
            file_roles,
            "",
            "### Reading Order",
            "",
            reading_order,
            "",
            "### Writable File Rule",
            "",
            writable_file,
        ]
    )


def live_tracker_and_closeout_block(manifest: dict[str, Any]) -> str:
    """Render the playbook-only mutable execution state and seeded closeout sections."""
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
            "### Findings Matrix",
            "",
            "- [ ] Finding -- evidence -- impact -- planned action.",
            "",
            "### Decision Log",
            "",
            "- [ ] Decision -- alternatives considered -- rationale -- score if applicable.",
            "",
            "### Affected Files Map",
            "",
            "- [ ] Path -- reason it changed -- verification notes.",
            "",
            "### Change Checklist",
            "",
            ledger,
            "",
            "### Verification Evidence",
            "",
            "- [ ] Command -- result -- notable warnings/failures.",
            "",
            f"### {manifest['report_heading']}",
            "",
            bullets(manifest["report_requirements"]),
            "",
            "### Deliverables",
            "",
            bullets(manifest["deliverables"]),
            "",
            "### Residual Risks / Defers",
            "",
            "- [ ] Record anything intentionally deferred or still risky, with an explicit reason.",
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
        "pack_map_block": pack_map_block(manifest),
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
        "repo_probes_block": nested_sections(manifest["repo_probes"], level=4),
        "upstream_validation_block": nested_sections(manifest["upstream_validation"], level=4),
        "framework_constraints_block": bullets(manifest["framework_constraints"]),
        "supported_features_block": bullets(manifest["supported_features"]),
        "unsupported_features_block": bullets(manifest["unsupported_features"]),
        "codemod_recommendations_block": bullets(manifest["codemod_recommendations"]),
        "skill_routing_playbook_block": bullets(manifest["skill_routing_playbook"]),
        "default_final_decisions_block": bullets(manifest["default_final_decisions"]),
        "intake_checklist_block": bullets([f"[ ] {item}" for item in manifest["intake_checklist"]]),
        "required_research_block": nested_sections(manifest["required_research"], level=4),
        "questions_to_resolve_block": bullets(manifest["questions_to_resolve"]),
        "canonical_end_state_block": bullets(manifest["canonical_end_state"]),
        "what_to_adopt_block": bullets(manifest["what_to_adopt"]),
        "what_to_avoid_block": bullets(manifest["what_to_avoid"]),
        "execution_plan_block": nested_sections(manifest["execution_plan"], level=4),
        "verification_commands_block": fenced_block(manifest["verification_commands"]),
        "live_tracker_and_closeout_block": live_tracker_and_closeout_block(manifest),
    }

    playbook_file = manifest["playbook_filename"]
    operator_read_this_first_block = raw_bullets(
        [
            f"Load {markdown_link(f'./{playbook_file}', playbook_file)} first and treat it as the source of truth.",
            f"Use {playbook_anchor_link(manifest, 'current-state-and-evidence', 'Current State And Evidence')} for repo-specific evidence and {playbook_anchor_link(manifest, 'decisions-and-end-state', 'Decisions And End State')} for the intended final posture.",
            f"Update only {playbook_anchor_link(manifest, 'live-tracker-and-closeout', 'Live Tracker And Closeout')} while implementing.",
            f"Qualification status for this pack: `{(qualification_snapshot or {}).get('qualification_status', 'pending')}`.",
        ]
    )
    operator_guardrails_block = bullets(manifest["framework_constraints"] + manifest["operator_defaults"])
    operator_replacements = {
        "operator_title": manifest["operator_title"],
        "operator_read_this_first_block": operator_read_this_first_block,
        "operator_guardrails_block": operator_guardrails_block,
        "operator_execute_block": bullets([f"[ ] {item}" for item in manifest["operator_execute"]]),
        "verification_commands_block": fenced_block(manifest["verification_commands"]),
        "operator_exit_criteria_block": bullets(manifest["operator_exit_criteria"]),
        "operator_required_closeout_block": raw_bullets(
            [
                f"Record progress, findings, affected files, verification evidence, and residual risks in {playbook_anchor_link(manifest, 'live-tracker-and-closeout', 'Live Tracker And Closeout')}.",
                f"Use {playbook_anchor_link(manifest, 'execution-and-verification', 'Execution And Verification')} if you need the full intake, research, or execution context.",
            ]
        ),
    }

    trigger_playbook_ref = f"./{manifest['playbook_filename']}"
    trigger_lines = [
        paragraph(manifest["trigger_mission"]),
        "",
        "Read first:",
        f"1. Set `PLAYBOOK={trigger_playbook_ref}`.",
        "2. Read `${PLAYBOOK}#pack-map`, `${PLAYBOOK}#current-state-and-evidence`, and `${PLAYBOOK}#decisions-and-end-state`.",
        "3. Treat `${PLAYBOOK}` as the source of truth and the only writable pack artifact.",
        "4. Update `${PLAYBOOK}#live-tracker-and-closeout` in place as work progresses.",
        "",
        "Repo-specific summary:",
        trigger_summary_block(manifest, qualification_snapshot),
        "",
        "Execution contract:",
        "- Follow `${PLAYBOOK}#execution-and-verification` for intake, research, execution, and repo-native verification.",
        "- Use `${PLAYBOOK}#live-tracker-and-closeout` for findings, decisions, affected files, verification evidence, and residual risks.",
        "- Finish with the playbook left in a final, accurate completed state.",
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
