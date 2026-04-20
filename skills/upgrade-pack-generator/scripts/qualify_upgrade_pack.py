#!/usr/bin/env python3
"""Run read-only qualification for an upgrade-pack manifest."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import shlex
import subprocess
from pathlib import Path
from typing import Any

from common import repo_local_skill_overlays, repo_path
from enrich_manifest import fetch_doc_metadata
from validate_upgrade_pack import validate_manifest


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, help="Path to upgrade-pack.yaml")
    parser.add_argument("--out", help="Optional alternate output path for qualification-snapshot.json")
    parser.add_argument("--research-snapshot", help="Optional path to research-snapshot.json")
    return parser


def load_manifest(path: Path) -> dict[str, Any]:
    import yaml

    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise SystemExit("manifest root must be a YAML dictionary")
    return data


def iso_now() -> str:
    """Return the current UTC timestamp."""
    return dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def clip_output(text: str, *, limit: int = 40) -> list[str]:
    """Return a short normalized output excerpt."""
    lines = [line.rstrip() for line in text.splitlines() if line.strip()]
    return lines[:limit]


def run_shell(command: str, cwd: Path) -> dict[str, Any]:
    """Run a read-only shell command and capture structured output."""
    completed = subprocess.run(
        ["zsh", "-lc", command],
        cwd=str(cwd),
        capture_output=True,
        text=True,
        check=False,
    )
    stdout = clip_output(completed.stdout)
    stderr = clip_output(completed.stderr)
    combined = stdout + stderr
    return {
        "exit_code": completed.returncode,
        "status": "ok" if completed.returncode == 0 else "failed",
        "stdout_excerpt": stdout,
        "stderr_excerpt": stderr,
        "summary": combined[:10],
    }


def qualify_docs(doc_urls: dict[str, str]) -> tuple[list[dict[str, Any]], int]:
    """Fetch live doc metadata for each qualification URL."""
    checks: list[dict[str, Any]] = []
    failures = 0
    for label, url in doc_urls.items():
        title, last_updated = fetch_doc_metadata(url)
        status = "ok" if not str(last_updated).startswith("unavailable") else "failed"
        if status != "ok":
            failures += 1
        checks.append(
            {
                "label": label,
                "url": url,
                "title": title,
                "last_updated": last_updated,
                "status": status,
            }
        )
    return checks, failures


def qualify_source(root: Path, specs: list[str]) -> tuple[list[dict[str, Any]], int]:
    """Resolve pinned opensrc paths for each requested source spec."""
    checks: list[dict[str, Any]] = []
    failures = 0
    for spec in specs:
        command = f"opensrc path {shlex.quote(spec)} --cwd {shlex.quote(str(root))}"
        result = run_shell(command, root)
        resolved_path = result["stdout_excerpt"][0] if result["stdout_excerpt"] else ""
        status = "ok" if result["status"] == "ok" and resolved_path else "failed"
        if status != "ok":
            failures += 1
        checks.append(
            {
                "spec": spec,
                "command": command,
                "resolved_path": resolved_path,
                "status": status,
                "summary": result["summary"],
            }
        )
    return checks, failures


def qualify_cli(root: Path, checks: list[dict[str, str]]) -> tuple[list[dict[str, Any]], int]:
    """Run family-native read-only CLI qualification commands."""
    results: list[dict[str, Any]] = []
    failures = 0
    for check in checks:
        cwd_label = check["cwd"]
        cwd = root if cwd_label == "." else root / cwd_label
        result = run_shell(check["command"], cwd)
        if result["status"] != "ok":
            failures += 1
        results.append(
            {
                "label": check["label"],
                "cwd": cwd_label,
                "command": check["command"],
                "exit_code": result["exit_code"],
                "status": result["status"],
                "summary": result["summary"],
            }
        )
    return results, failures


def qualification_status(summary: dict[str, int], research_snapshot: dict[str, Any] | None) -> tuple[str, list[str]]:
    """Return a normalized overall qualification result and caveats."""
    caveats: list[str] = []
    research_status = (research_snapshot or {}).get("research_status", "pending")
    if (
        summary["cli_failures"] == 0
        and summary["source_failures"] == 0
        and summary["doc_failures"] == 0
        and research_status == "complete"
    ):
        return "ready", caveats
    if research_status != "complete":
        caveats.append(f"research stage is `{research_status}`")
    if summary["cli_checks"] == summary["cli_failures"]:
        caveats.append("all CLI qualification checks failed")
    if summary["source_checks"] and summary["source_checks"] == summary["source_failures"]:
        caveats.append("all source qualification checks failed")
    if summary["doc_checks"] and summary["doc_checks"] == summary["doc_failures"]:
        caveats.append("all doc qualification checks failed")
    if research_status == "insufficient-evidence":
        return "insufficient-evidence", caveats
    if caveats and any(item.startswith("all ") for item in caveats):
        return "insufficient-evidence", caveats
    caveats.append(
        "one or more research or qualification checks are incomplete; inspect the snapshots before treating the pack as fully qualified"
    )
    return "ready-with-caveats", caveats


def snapshot_path(manifest_path: Path, manifest: dict[str, Any], explicit_out: str | None) -> Path:
    """Return the output path for the qualification snapshot."""
    if explicit_out:
        return Path(explicit_out).expanduser().resolve()
    qualification_plan = manifest.get("qualification_plan") or {}
    filename = str(qualification_plan.get("snapshot_filename") or "qualification-snapshot.json")
    return manifest_path.parent / filename


def research_snapshot_path(manifest_path: Path, manifest: dict[str, Any], explicit_path: str | None) -> Path | None:
    """Return the expected research snapshot path when present."""
    if explicit_path:
        return Path(explicit_path).expanduser().resolve()
    research_plan = manifest.get("research_plan") or {}
    filename = str(research_plan.get("snapshot_filename") or "").strip()
    if not filename:
        return None
    return manifest_path.parent / filename


def main() -> None:
    args = build_parser().parse_args()
    manifest_path = Path(args.manifest).expanduser().resolve()
    valid, errors = validate_manifest(manifest_path)
    if not valid:
        print("Manifest validation failed before qualification:")
        for error in errors:
            print(f"- {error}")
        raise SystemExit(1)

    manifest = load_manifest(manifest_path)
    root = repo_path(manifest["repo_context"]["repo_root"])
    qualification_plan = manifest.get("qualification_plan") or {}
    research_path = research_snapshot_path(manifest_path, manifest, args.research_snapshot)
    research_snapshot = None
    if research_path and research_path.exists():
        research_snapshot = json.loads(research_path.read_text(encoding="utf-8"))

    doc_checks, doc_failures = qualify_docs(qualification_plan.get("doc_urls") or {})
    source_checks, source_failures = qualify_source(root, qualification_plan.get("source_specs") or [])
    cli_checks, cli_failures = qualify_cli(root, qualification_plan.get("cli_checks") or [])
    overlays = repo_local_skill_overlays(root, manifest["family_slug"])

    summary = {
        "doc_checks": len(doc_checks),
        "doc_failures": doc_failures,
        "source_checks": len(source_checks),
        "source_failures": source_failures,
        "cli_checks": len(cli_checks),
        "cli_failures": cli_failures,
        "repo_local_overlays": len(overlays),
        "research_status": (research_snapshot or {}).get("research_status", "pending"),
    }
    status, caveats = qualification_status(summary, research_snapshot)
    snapshot = {
        "schema_version": 1,
        "generated_at": iso_now(),
        "family_slug": manifest["family_slug"],
        "anchor_package": manifest["anchor_package"],
        "repo_root": manifest["repo_context"]["repo_root"],
        "snapshot_filename": qualification_plan.get("snapshot_filename", "qualification-snapshot.json"),
        "qualification_status": status,
        "summary": summary,
        "doc_checks": doc_checks,
        "source_checks": source_checks,
        "cli_checks": cli_checks,
        "research_snapshot": {
            "path": str(research_path) if research_path else None,
            "status": (research_snapshot or {}).get("research_status", "pending"),
        },
        "repo_local_skill_overlays": overlays,
        "caveats": caveats,
    }

    out_path = snapshot_path(manifest_path, manifest, args.out)
    out_path.write_text(json.dumps(snapshot, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(out_path)


if __name__ == "__main__":
    main()
