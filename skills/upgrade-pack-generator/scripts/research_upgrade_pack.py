#!/usr/bin/env python3
"""Run read-only upstream and repo research for an upgrade-pack manifest."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import shlex
import subprocess
from pathlib import Path
from typing import Any

from common import repo_path
from enrich_manifest import fetch_doc_metadata
from validate_upgrade_pack import validate_manifest


URL_CATEGORIES = (
    "official_docs",
    "api_reference",
    "migration_guides",
    "release_history",
    "examples_cookbooks",
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, help="Path to upgrade-pack.yaml")
    parser.add_argument("--out", help="Optional alternate output path for research-snapshot.json")
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


def url_entries(urls: dict[str, str]) -> list[dict[str, Any]]:
    """Fetch metadata for a mapping of labeled URLs."""
    entries: list[dict[str, Any]] = []
    for label, url in urls.items():
        title, last_updated = fetch_doc_metadata(url)
        status = "ok" if not str(last_updated).startswith("unavailable") else "failed"
        entries.append(
            {
                "label": label,
                "url": url,
                "title": title,
                "last_updated": last_updated,
                "status": status,
            }
        )
    return entries


def category_status(entries: list[dict[str, Any]], configured_count: int) -> str:
    """Return the normalized status for a category."""
    if configured_count == 0:
        return "missing"
    ok_count = sum(1 for entry in entries if entry.get("status") == "ok")
    if ok_count == configured_count:
        return "ok"
    if ok_count == 0:
        return "failed"
    return "partial"


def configured_count(research_plan: dict[str, Any], category: str) -> int:
    """Return how many configured sources or checks back a category."""
    if category in URL_CATEGORIES:
        return len(research_plan.get(category) or {})
    if category == "source_evidence":
        return len(research_plan.get("source_specs") or [])
    if category == "repo_usage_mapping":
        return len(research_plan.get("repo_usage_queries") or [])
    return 0


def supporting_source_files(base: Path, prefixes: tuple[str, ...]) -> list[str]:
    """Return repo-relative filenames under an opensrc tree that match prefixes."""
    matches: list[str] = []
    for path in base.rglob("*"):
        if not path.is_file():
            continue
        name = path.name.upper()
        if any(name.startswith(prefix) for prefix in prefixes):
            matches.append(path.relative_to(base).as_posix())
    return matches[:20]


def example_source_paths(base: Path) -> list[str]:
    """Return high-signal example or cookbook paths under an opensrc tree."""
    matches: list[str] = []
    for path in base.rglob("*"):
        if not path.is_file():
            continue
        relative = path.relative_to(base).as_posix()
        lowered = relative.lower()
        if any(token in lowered for token in ("example", "examples/", "cookbook", "demo", "/docs/")):
            matches.append(relative)
    return matches[:20]


def source_entries(root: Path, specs: list[str]) -> list[dict[str, Any]]:
    """Resolve source specs and record supporting file hints."""
    entries: list[dict[str, Any]] = []
    for spec in specs:
        command = f"opensrc path {shlex.quote(spec)} --cwd {shlex.quote(str(root))}"
        result = run_shell(command, root)
        resolved_path = result["stdout_excerpt"][0] if result["stdout_excerpt"] else ""
        resolved = Path(resolved_path).expanduser().resolve() if resolved_path else None
        status = "ok" if result["status"] == "ok" and resolved and resolved.exists() else "failed"
        release_note_files: list[str] = []
        migration_files: list[str] = []
        example_paths: list[str] = []
        if status == "ok" and resolved is not None:
            release_note_files = supporting_source_files(resolved, ("CHANGELOG", "RELEASE", "BREAKING"))
            migration_files = supporting_source_files(resolved, ("MIGRATION", "UPGRADE"))
            example_paths = example_source_paths(resolved)
        entries.append(
            {
                "spec": spec,
                "command": command,
                "resolved_path": resolved_path,
                "status": status,
                "summary": result["summary"],
                "release_note_files": release_note_files,
                "migration_files": migration_files,
                "example_paths": example_paths,
            }
        )
    return entries


def repo_usage_entries(root: Path, checks: list[dict[str, str]]) -> list[dict[str, Any]]:
    """Run repo-usage mapping commands."""
    entries: list[dict[str, Any]] = []
    for check in checks:
        cwd_label = check["cwd"]
        cwd = root if cwd_label == "." else root / cwd_label
        result = run_shell(check["command"], cwd)
        status = result["status"]
        summary = result["summary"]
        stdout_excerpt = result["stdout_excerpt"]
        stderr_excerpt = result["stderr_excerpt"]
        if result["exit_code"] == 1 and check["command"].lstrip().startswith("rg "):
            status = "ok"
            summary = ["no matches"]
            stdout_excerpt = []
            stderr_excerpt = []
        entries.append(
            {
                "label": check["label"],
                "cwd": cwd_label,
                "command": check["command"],
                "exit_code": result["exit_code"],
                "status": status,
                "summary": summary,
                "stdout_excerpt": stdout_excerpt,
                "stderr_excerpt": stderr_excerpt,
            }
        )
    return entries


def summarize_categories(category_statuses: dict[str, str]) -> dict[str, int]:
    """Count normalized category states."""
    counts = {"ok": 0, "partial": 0, "failed": 0, "missing": 0}
    for status in category_statuses.values():
        counts[status] = counts.get(status, 0) + 1
    return counts


def research_status(required_categories: list[str], category_statuses: dict[str, str]) -> tuple[str, list[str]]:
    """Return the normalized overall research status and caveats."""
    caveats: list[str] = []
    statuses = [category_statuses.get(category, "missing") for category in required_categories]
    if statuses and all(status == "ok" for status in statuses):
        return "complete", caveats

    missing_categories = [category for category in required_categories if category_statuses.get(category) == "missing"]
    failed_categories = [category for category in required_categories if category_statuses.get(category) == "failed"]
    partial_categories = [category for category in required_categories if category_statuses.get(category) == "partial"]
    if missing_categories:
        caveats.append(f"missing research categories: {', '.join(missing_categories)}")
    if failed_categories:
        caveats.append(f"failed research categories: {', '.join(failed_categories)}")
    if partial_categories:
        caveats.append(f"partial research categories: {', '.join(partial_categories)}")

    if any(status in {"ok", "partial"} for status in statuses):
        return "partial", caveats
    return "insufficient-evidence", caveats


def snapshot_path(manifest_path: Path, manifest: dict[str, Any], explicit_out: str | None) -> Path:
    """Return the output path for the research snapshot."""
    if explicit_out:
        return Path(explicit_out).expanduser().resolve()
    research_plan = manifest.get("research_plan") or {}
    filename = str(research_plan.get("snapshot_filename") or "research-snapshot.json")
    return manifest_path.parent / filename


def generate_snapshot(manifest: dict[str, Any], root: Path) -> dict[str, Any]:
    """Generate the in-memory research snapshot for a manifest."""
    research_plan = manifest.get("research_plan") or {}

    category_entries: dict[str, list[dict[str, Any]]] = {
        category: url_entries(research_plan.get(category) or {})
        for category in URL_CATEGORIES
    }
    source_evidence = source_entries(root, research_plan.get("source_specs") or [])
    repo_usage_mapping = repo_usage_entries(root, research_plan.get("repo_usage_queries") or [])
    category_entries["source_evidence"] = source_evidence
    category_entries["repo_usage_mapping"] = repo_usage_mapping

    category_statuses = {
        category: category_status(entries, configured_count(research_plan, category))
        for category, entries in category_entries.items()
    }
    required_categories = research_plan.get("required_categories") or []
    overall_status, caveats = research_status(required_categories, category_statuses)
    summary_counts = summarize_categories({category: category_statuses.get(category, "missing") for category in required_categories})
    return {
        "schema_version": 1,
        "generated_at": iso_now(),
        "family_slug": manifest["family_slug"],
        "anchor_package": manifest["anchor_package"],
        "repo_root": manifest["repo_context"]["repo_root"],
        "snapshot_filename": research_plan.get("snapshot_filename", "research-snapshot.json"),
        "research_status": overall_status,
        "current_version": manifest.get("current_version", "unknown"),
        "target_version": research_plan.get("target_version", "unknown"),
        "target_version_policy": research_plan.get("target_version_policy", "latest-compatible-stable"),
        "compatibility_rationale": research_plan.get("compatibility_rationale", "unknown"),
        "release_range": research_plan.get("release_range", "unknown"),
        "required_categories": required_categories,
        "category_status": category_statuses,
        "summary": {
            "required_categories": len(required_categories),
            "ok_categories": summary_counts.get("ok", 0),
            "partial_categories": summary_counts.get("partial", 0),
            "failed_categories": summary_counts.get("failed", 0),
            "missing_categories": summary_counts.get("missing", 0),
            "official_docs": len(category_entries["official_docs"]),
            "api_reference": len(category_entries["api_reference"]),
            "migration_guides": len(category_entries["migration_guides"]),
            "release_history": len(category_entries["release_history"]),
            "examples_cookbooks": len(category_entries["examples_cookbooks"]),
            "source_evidence": len(category_entries["source_evidence"]),
            "repo_usage_mapping": len(category_entries["repo_usage_mapping"]),
        },
        "official_docs": category_entries["official_docs"],
        "api_reference": category_entries["api_reference"],
        "migration_guides": category_entries["migration_guides"],
        "release_history": category_entries["release_history"],
        "examples_cookbooks": category_entries["examples_cookbooks"],
        "source_evidence": category_entries["source_evidence"],
        "repo_usage_mapping": category_entries["repo_usage_mapping"],
        "caveats": caveats,
    }


def main() -> None:
    args = build_parser().parse_args()
    manifest_path = Path(args.manifest).expanduser().resolve()
    valid, errors = validate_manifest(manifest_path)
    if not valid:
        print("Manifest validation failed before research:")
        for error in errors:
            print(f"- {error}")
        raise SystemExit(1)

    manifest = load_manifest(manifest_path)
    root = repo_path(manifest["repo_context"]["repo_root"])
    snapshot = generate_snapshot(manifest, root)

    out_path = snapshot_path(manifest_path, manifest, args.out)
    out_path.write_text(json.dumps(snapshot, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(out_path)


if __name__ == "__main__":
    main()
