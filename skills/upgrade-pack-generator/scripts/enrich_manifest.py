#!/usr/bin/env python3
"""Enrich an upgrade-pack manifest with family-specific repo probes and live upstream facts."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import shlex
from pathlib import Path
from typing import Any
from urllib.error import URLError
from urllib.request import Request, urlopen

from common import (
    dlx_command,
    dump_yaml,
    manifests_declaring_package,
    next_repo_probes,
    normalize_package_version_for_source,
    package_versions_from_manifest,
    package_versions_from_repo,
    pick_script,
    repo_exists_any,
    repo_local_skill_overlays,
    repo_path,
    root_manifest_record,
    root_package_json_data,
    safe_read_text,
    source_files_under,
    unique_list,
    workspace_dir,
    workspace_display_path,
    workspace_exists_any,
    workspace_manifest_records,
    workspace_reference,
    workspace_script_command,
    workspace_slug,
)
from validate_upgrade_pack import validate_manifest


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, help="Path to upgrade-pack.yaml")
    parser.add_argument("--out", help="Optional alternate output path. Defaults to in-place update.")
    return parser


def load_manifest(path: Path) -> dict[str, Any]:
    import yaml

    data = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise SystemExit("manifest root must be a YAML dictionary")
    return data


def fetch_doc_metadata(url: str) -> tuple[str, str]:
    """Fetch a doc and return a short title/date pair when possible."""
    try:
        request = Request(
            url,
            headers={
                "User-Agent": "Mozilla/5.0 (compatible; upgrade-pack-generator/1.0; +https://openai.com)",
                "Accept": "text/html,application/xhtml+xml",
            },
        )
        with urlopen(request, timeout=20) as response:
            html = response.read().decode("utf-8", errors="ignore")
    except URLError as exc:
        return (url, f"unavailable ({exc.reason})")

    title_match = re.search(r"<title>(.*?)</title>", html, flags=re.IGNORECASE | re.DOTALL)
    title = re.sub(r"\s+", " ", title_match.group(1)).strip() if title_match else url
    date_match = re.search(
        r"Last updated(?: on)?\s*(?:<!--\s*-->\s*)?([A-Za-z]+\s+\d{1,2},\s+\d{4})",
        html,
        flags=re.IGNORECASE,
    )
    if date_match:
        return (title, date_match.group(1))

    iso_patterns = [
        r'"dateModified"\s*:\s*"([^"]+)"',
        r'"datePublished"\s*:\s*"([^"]+)"',
        r'<meta[^>]+(?:property|name)=["\']article:modified_time["\'][^>]+content=["\']([^"\']+)["\']',
        r'<meta[^>]+(?:property|name)=["\']article:published_time["\'][^>]+content=["\']([^"\']+)["\']',
        r'<time[^>]+datetime=["\']([^"\']+)["\']',
    ]
    for pattern in iso_patterns:
        iso_match = re.search(pattern, html, flags=re.IGNORECASE)
        if not iso_match:
            continue
        raw_value = iso_match.group(1).strip()
        for candidate in (raw_value, raw_value.replace("Z", "+00:00")):
            try:
                parsed = dt.datetime.fromisoformat(candidate)
            except ValueError:
                continue
            return (title, f"{parsed.strftime('%B')} {parsed.day}, {parsed.year}")

    return (title, "unknown")


def version_major(version: str, fallback: str) -> str:
    """Extract the first numeric major version from a semver-ish string."""
    match = re.search(r"(\d+)", version)
    return match.group(1) if match else fallback


def set_plan_identity(
    manifest: dict[str, Any],
    *,
    basename: str,
    playbook_title: str,
    operator_title: str,
    trigger_title: str,
) -> None:
    """Update the plan basename, titles, and derived filenames in one place."""
    manifest["plan_basename"] = basename
    manifest["playbook_title"] = playbook_title
    manifest["operator_title"] = operator_title
    manifest["trigger_title"] = trigger_title
    manifest["playbook_filename"] = f"{basename}-playbook.md"
    manifest["operator_filename"] = f"{basename}-operator-mode.md"
    manifest["trigger_filename"] = f"{basename}-trigger-prompt.md"


def ensure_manifest_qualification_defaults(manifest: dict[str, Any]) -> None:
    """Upgrade older manifests to the qualification-aware schema shape."""
    manifest["schema_version"] = 3
    qualification_plan = manifest.get("qualification_plan")
    if not isinstance(qualification_plan, dict):
        qualification_plan = {}
    qualification_plan.setdefault("strategy", "separate-read-only-qualification")
    qualification_plan.setdefault("snapshot_filename", "qualification-snapshot.json")
    qualification_plan.setdefault("doc_urls", {})
    qualification_plan.setdefault("source_specs", [])
    qualification_plan.setdefault("cli_checks", [])
    manifest["qualification_plan"] = qualification_plan


def ensure_manifest_research_defaults(manifest: dict[str, Any]) -> None:
    """Upgrade older manifests to the research-aware schema shape."""
    manifest["schema_version"] = 3
    research_plan = manifest.get("research_plan")
    if not isinstance(research_plan, dict):
        research_plan = {}
    research_plan.setdefault("strategy", "separate-read-only-research")
    research_plan.setdefault("snapshot_filename", "research-snapshot.json")
    research_plan.setdefault("bundle_filename", "research-bundle.json")
    research_plan.setdefault("web_findings_filename", "web-research-findings.json")
    research_plan.setdefault(
        "required_categories",
        [
            "official_docs",
            "api_reference",
            "migration_guides",
            "release_history",
            "examples_cookbooks",
            "source_evidence",
            "repo_usage_mapping",
        ],
    )
    research_plan.setdefault(
        "source_priority",
        [
            "official docs and API references first",
            "official migration guides and upgrade walkthroughs second",
            "official blog, release notes, and changelog sources third",
            "upstream source inspection fourth",
            "examples and cookbooks fifth",
            "repo-local usage mapping always required",
        ],
    )
    research_plan.setdefault("identity_confidence_threshold", 0.75)
    research_plan.setdefault("source_map_policy", "bundled-seed-then-verify")
    research_plan.setdefault(
        "required_web_confirmation_categories",
        [
            "official_docs",
            "api_reference",
        ],
    )
    research_plan.setdefault("target_version_policy", "latest-compatible-stable")
    research_plan.setdefault(
        "target_version",
        "latest supportable stable release to be confirmed during enrichment and research",
    )
    research_plan.setdefault(
        "compatibility_rationale",
        "Use the latest supportable stable release whose documented constraints fit the repo's framework, runtime, and policy boundaries.",
    )
    research_plan.setdefault(
        "release_range",
        "current repo version -> latest supportable stable release under verified repo constraints",
    )
    for key in (
        "official_docs",
        "api_reference",
        "migration_guides",
        "release_history",
        "examples_cookbooks",
    ):
        research_plan.setdefault(key, {})
    research_plan.setdefault("source_specs", [])
    research_plan.setdefault("repo_usage_queries", [])
    manifest["research_plan"] = research_plan


def qualification_cli_check(label: str, cwd: str, command: str) -> dict[str, str]:
    """Return a normalized CLI qualification check payload."""
    return {
        "label": label,
        "cwd": cwd,
        "command": command,
    }


def research_repo_usage_query(label: str, cwd: str, command: str) -> dict[str, str]:
    """Return a normalized repo-usage research query payload."""
    return {
        "label": label,
        "cwd": cwd,
        "command": command,
    }


def apply_qualification_plan(
    manifest: dict[str, Any],
    root: Path,
    *,
    doc_urls: dict[str, str],
    source_specs: list[str],
    cli_checks: list[dict[str, str]],
) -> None:
    """Write qualification metadata and repo-local overlay notes into a manifest."""
    ensure_manifest_qualification_defaults(manifest)
    overlays = repo_local_skill_overlays(root, manifest["family_slug"])
    qualification_plan = manifest["qualification_plan"]
    qualification_plan["doc_urls"] = doc_urls
    qualification_plan["source_specs"] = unique_list(source_specs)
    qualification_plan["cli_checks"] = cli_checks
    manifest["repo_local_skill_overlays"] = overlays


def apply_research_plan(
    manifest: dict[str, Any],
    *,
    official_docs: dict[str, str],
    api_reference: dict[str, str],
    migration_guides: dict[str, str],
    release_history: dict[str, str],
    examples_cookbooks: dict[str, str],
    source_specs: list[str],
    repo_usage_queries: list[dict[str, str]],
    target_version: str,
    compatibility_rationale: str,
    release_range: str,
    target_version_policy: str = "latest-compatible-stable",
) -> None:
    """Write research metadata into a manifest."""
    ensure_manifest_research_defaults(manifest)
    research_plan = manifest["research_plan"]
    research_plan["official_docs"] = official_docs
    research_plan["api_reference"] = api_reference
    research_plan["migration_guides"] = migration_guides
    research_plan["release_history"] = release_history
    research_plan["examples_cookbooks"] = examples_cookbooks
    research_plan["source_specs"] = unique_list(source_specs)
    research_plan["repo_usage_queries"] = repo_usage_queries
    research_plan["target_version_policy"] = target_version_policy
    research_plan["target_version"] = target_version
    research_plan["compatibility_rationale"] = compatibility_rationale
    research_plan["release_range"] = release_range


def build_target_surface(
    root: Path,
    owner: dict[str, Any],
    *,
    owner_reason: str,
    related_workspaces: list[dict[str, Any]] | None = None,
    verification_strategy: str | None = None,
    surface_type: str | None = None,
) -> dict[str, Any]:
    """Build the canonical target surface payload for a manifest."""
    related = related_workspaces or [owner]
    owner_path = workspace_display_path(owner)
    if surface_type is None:
        if owner.get("is_root"):
            surface_type = "repo-root"
        elif len(related) > 1:
            surface_type = "workspace-and-related"
        else:
            surface_type = "workspace"
    if verification_strategy is None:
        verification_strategy = "root-only" if owner.get("is_root") else "layered-root-and-workspace"

    return {
        "surface_type": surface_type,
        "workspace_path": owner_path,
        "workspace_name": str(owner.get("package_name") or "(unnamed workspace)"),
        "workspace_package_json": owner["package_json_path"],
        "workspace_slug": workspace_slug(owner),
        "owner_reason": owner_reason,
        "related_workspaces": [workspace_display_path(record) for record in related],
        "verification_strategy": verification_strategy,
    }


def add_script_command(
    commands: list[str],
    seen: set[str],
    package_manager: str,
    record: dict[str, Any],
    candidates: list[str],
) -> None:
    """Append the first available script command for a record."""
    script = pick_script(record, candidates)
    if not script:
        return
    command = workspace_script_command(package_manager, record, script)
    if command not in seen:
        commands.append(command)
        seen.add(command)


def package_versions_from_records(records: list[dict[str, Any]], packages: list[str]) -> dict[str, str]:
    """Resolve package versions using the first record that declares each package."""
    detected: dict[str, str] = {}
    for package in packages:
        for record in records:
            versions = package_versions_from_manifest(record, [package])
            if package in versions:
                detected[package] = versions[package]
                break
    return detected


def best_scored_owner(records: list[dict[str, Any]], score_fn) -> dict[str, Any]:
    """Return the highest-scoring workspace record."""
    scored: list[tuple[int, int, str, dict[str, Any]]] = []
    for record in records:
        score = score_fn(record)
        if score <= 0:
            continue
        scored.append(
            (
                score,
                0 if record.get("is_root") else 1,
                workspace_display_path(record),
                record,
            )
        )
    if not scored:
        return root_manifest_record(repo_path(records[0]["package_json_path"])) if records else {}
    return sorted(scored, key=lambda item: (item[0], item[1], item[2]), reverse=True)[0][3]


def pick_best_owner(root: Path, score_fn) -> dict[str, Any]:
    """Pick a workspace owner from all manifest records using a scoring function."""
    records = workspace_manifest_records(root)
    scored: list[tuple[int, int, str, dict[str, Any]]] = []
    for record in records:
        score = score_fn(record)
        if score <= 0:
            continue
        scored.append(
            (
                score,
                0 if record.get("is_root") else 1,
                workspace_display_path(record),
                record,
            )
        )
    if not scored:
        return root_manifest_record(root)
    return sorted(scored, key=lambda item: (item[0], item[1], item[2]), reverse=True)[0][3]


def owner_workspace_title(owner: dict[str, Any]) -> str:
    """Return a display title fragment for an owner workspace."""
    path = workspace_display_path(owner)
    return "repo root" if path == "." else path


def next_major(version: str) -> str:
    """Return the current Next.js major version string."""
    return version_major(version, "16")


def next_doc_urls(version: str) -> dict[str, str]:
    """Return official Next.js documentation URLs."""
    major = next_major(version)
    return {
        "upgrade guide": f"https://nextjs.org/docs/app/guides/upgrading/version-{major}",
        "codemods": "https://nextjs.org/docs/app/guides/upgrading/codemods",
        "cacheComponents": "https://nextjs.org/docs/app/api-reference/config/next-config-js/useCache",
        "typedRoutes": "https://nextjs.org/docs/app/api-reference/config/next-config-js/typedRoutes",
        "proxy": "https://nextjs.org/docs/app/getting-started/proxy",
        "release post": f"https://nextjs.org/blog/next-{major}",
    }


def next_research_sources(version: str) -> dict[str, dict[str, str]]:
    """Return categorized research sources for Next.js."""
    major = next_major(version)
    return {
        "official_docs": {
            "docs home": "https://nextjs.org/docs",
            "app router overview": "https://nextjs.org/docs/app",
        },
        "api_reference": {
            "cacheComponents": "https://nextjs.org/docs/app/api-reference/config/next-config-js/useCache",
            "typedRoutes": "https://nextjs.org/docs/app/api-reference/config/next-config-js/typedRoutes",
            "cookies": "https://nextjs.org/docs/app/api-reference/functions/cookies",
            "proxy": "https://nextjs.org/docs/app/getting-started/proxy",
        },
        "migration_guides": {
            "upgrade guide": f"https://nextjs.org/docs/app/guides/upgrading/version-{major}",
            "codemods": "https://nextjs.org/docs/app/guides/upgrading/codemods",
        },
        "release_history": {
            "release post": f"https://nextjs.org/blog/next-{major}",
            "github releases": "https://github.com/vercel/next.js/releases",
        },
        "examples_cookbooks": {
            "static exports": "https://nextjs.org/docs/app/guides/static-exports",
            "ai coding agents": "https://nextjs.org/docs/app/guides/ai-coding-agents",
        },
    }


def next_repo_usage_queries(owner: dict[str, Any]) -> list[dict[str, str]]:
    """Return repo-usage mapping commands for Next.js."""
    owner_cwd = workspace_display_path(owner)
    return [
        research_repo_usage_query("Next manifest declarations", ".", "rg -n '\"next\"|\"react\"|\"react-dom\"|\"@next/codemod\"' ."),
        research_repo_usage_query("Next config and route surfaces", ".", f"rg --files {shlex.quote(owner_cwd)} | rg '(next\\.config\\.|(^|/)proxy\\.|(^|/)middleware\\.|(^|/)app/|(^|/)src/app/)'"),
        research_repo_usage_query("Next API usage", ".", f"rg -n \"from ['\\\"]next/(font|image|link|navigation|server|cache)|\\bcookies\\(|\\bheaders\\(|\\bdraftMode\\(|\\bconnection\\(|\\brevalidateTag\\(|\\bupdateTag\\(|\\bcacheLife\\(|\\bcacheTag\\(\" {shlex.quote(owner_cwd)}"),
    ]


def detect_next_owner(root: Path) -> dict[str, Any]:
    """Pick the owning Next.js workspace."""
    records = workspace_manifest_records(root)

    def score(record: dict[str, Any]) -> int:
        total = 0
        deps = record.get("dependencies") or {}
        if "next" in deps:
            total += 10
        if workspace_exists_any(root, record, ["next.config.ts", "next.config.mjs", "next.config.js", "next.config.cjs"]):
            total += 6
        if workspace_exists_any(root, record, ["app", "src/app"]):
            total += 4
        if workspace_exists_any(root, record, ["pages", "src/pages"]):
            total += 2
        if workspace_exists_any(root, record, ["proxy.ts", "proxy.js", "src/proxy.ts", "src/proxy.js"]):
            total += 3
        if workspace_exists_any(root, record, ["middleware.ts", "middleware.js", "src/middleware.ts", "src/middleware.js"]):
            total += 1
        scripts = record.get("scripts") or {}
        if any(command.startswith("next ") for command in scripts.values()):
            total += 3
        if any("next typegen" in command for command in scripts.values()):
            total += 2
        return total

    return pick_best_owner(root, score)


def next_related_packages(owner: dict[str, Any], root_record: dict[str, Any]) -> list[str]:
    """Return the related package set for a Next.js wave."""
    ordered = ["next", "react", "react-dom", "@types/react", "@types/react-dom", "@next/codemod", "eslint-config-next"]
    versions = package_versions_from_records([owner, root_record], ordered)
    return unique_list([package for package in ordered if package in versions])


def current_next_surface_notes(root: Path, owner: dict[str, Any], root_record: dict[str, Any]) -> list[str]:
    """Return current repo and owner script posture notes for Next.js."""
    root_scripts = root_record.get("scripts") or {}
    owner_scripts = owner.get("scripts") or {}
    next_config = workspace_exists_any(root, owner, ["next.config.ts", "next.config.mjs", "next.config.js", "next.config.cjs"])
    next_config_text = safe_read_text(root / next_config) if next_config else ""
    notes = [
        f"Root script `lint`: `{root_scripts.get('lint', '(missing)')}`",
        f"Root script `typecheck`: `{root_scripts.get('typecheck', '(missing)')}`",
        f"Root script `validate:local:agent`: `{root_scripts.get('validate:local:agent', '(missing)')}`",
        f"Owner script `typegen`: `{owner_scripts.get('typegen', '(missing)')}`",
        f"Owner script `typecheck`: `{owner_scripts.get('typecheck', '(missing)')}`",
        f"Owner script `build`: `{owner_scripts.get('build', '(missing)')}`",
    ]
    if "transpilePackages" in next_config_text:
        notes.append("The owner workspace already uses `transpilePackages`; preserve shared-workspace compatibility when upgrading.")
    if "withSentryConfig" in next_config_text:
        notes.append("The Next config is wrapped with Sentry; verify framework changes against that integration boundary.")
    if "experimental.viewTransition" in next_config_text:
        notes.append("The owner workspace already enables `experimental.viewTransition`; review version guidance before changing that posture.")
    return notes


def next_constraints(root: Path, owner: dict[str, Any]) -> tuple[list[str], list[str], list[str]]:
    """Return Next.js constraints and feature posture for the owner workspace."""
    next_config = workspace_exists_any(root, owner, ["next.config.ts", "next.config.mjs", "next.config.js", "next.config.cjs"])
    next_config_text = safe_read_text(root / next_config) if next_config else ""
    scripts = owner.get("scripts") or {}
    typecheck_script = str(scripts.get(pick_script(owner, ["typecheck", "type-check"]) or "") or "")
    static_export = 'output: "export"' in next_config_text or "output: 'export'" in next_config_text

    constraints = [
        "Treat Next.js as a framework boundary; routing, config, cache posture, and deployment posture move together.",
        "Resolve framework decisions against the owner workspace, not only the repo root.",
        "Verify version-sensitive Next.js claims live against official docs before editing.",
    ]
    supported = [
        "App Router and route handlers.",
        "`next/font`, `next/image`, `next/link`, `next/navigation`, `next/server`, and `next/cache` when they fit the repo posture.",
        "Stable `typedRoutes` and `cacheComponents` in Next.js 16.",
    ]
    unsupported = [
        "Do not leave stale Next.js APIs or transitional config names in parallel.",
    ]

    if static_export:
        constraints.append("Static export is enabled for the owner workspace; do not introduce server-runtime-only features into that surface.")
        unsupported.extend(
            [
                "Do not introduce runtime-only request APIs or proxy into a static-export-only workspace.",
                "Do not remove the custom static-export image posture unless the deployment boundary changes too.",
            ]
        )
    else:
        constraints.append("This owner workspace is not static-export-only; preserve its runtime route and proxy boundaries during optimization work.")
        supported.extend(
            [
                "Runtime `proxy.ts` and request-time APIs when supported by the deployed app posture.",
                "`connection`, `cacheLife`, `cacheTag`, and `updateTag` where they are already in the live surface.",
            ]
        )

    if "next typegen" in typecheck_script:
        supported.append("`next typegen` already participates in type validation and should stay in the verification path.")
    else:
        supported.append("Consider adding `next typegen` when async request-time props or typed routes need stronger guarantees.")

    return (unique_list(constraints), unique_list(supported), unique_list(unsupported))


def next_codemods(owner: dict[str, Any], manifest: dict[str, Any], current_version: str) -> list[str]:
    """Return Next.js codemod and automation suggestions."""
    pm_dlx = (manifest["repo_context"].get("command_variables") or {}).get("PM_DLX", "npx")
    workspace_path = workspace_display_path(owner)
    scope_prefix = "." if workspace_path == "." else workspace_path
    recommendations = [
        f"Use `{pm_dlx} @next/codemod@canary upgrade latest --dir {scope_prefix}` when a live Next.js major or patch upgrade is required.",
    ]
    if workspace_exists_any(repo_path(manifest["repo_context"]["repo_root"]), owner, ["middleware.ts", "middleware.js", "src/middleware.ts", "src/middleware.js"]):
        recommendations.append(f"Use `{pm_dlx} @next/codemod@latest middleware-to-proxy {scope_prefix}` if any deprecated `middleware.*` surface remains.")
    recommendations.append(f"Use `{pm_dlx} @next/codemod@latest remove-unstable-prefix {scope_prefix}` if `unstable_cacheLife` or `unstable_cacheTag` remain.")
    major = next_major(current_version)
    recommendations.append(f"Use `{pm_dlx} next typegen` before and after high-risk request API migrations to refresh generated Next.js {major} helpers.")
    return unique_list(recommendations)


def next_verification_commands(root: Path, owner: dict[str, Any], root_record: dict[str, Any], manifest: dict[str, Any]) -> list[str]:
    """Build layered root and workspace verification commands for Next.js."""
    package_manager = manifest["repo_context"]["package_manager"]
    base_path = workspace_display_path(owner)
    base_arg = shlex.quote(base_path)
    manifest_path = shlex.quote(owner["package_json_path"])
    commands = [
        "# inventory",
        f"rg -n '\"next\"|\"react\"|\"react-dom\"|\"@next/codemod\"' {manifest_path}",
        f"rg --files {base_arg} | rg '(next\\.config\\.|(^|/)proxy\\.|(^|/)middleware\\.|(^|/)app/|(^|/)src/app/)'",
        f"rg -n \"from ['\\\"]next/(font|image|link|navigation|server|cache)\" {base_arg}",
        f"rg -n \"\\bcookies\\(|\\bheaders\\(|\\bdraftMode\\(|\\bconnection\\(\" {base_arg}",
        f"rg -n \"\\brevalidateTag\\(|\\bupdateTag\\(|\\bcacheLife\\(|\\bcacheTag\\(|\\bunstable_cache(Life|Tag)\\(\" {base_arg}",
        "",
        "# verification",
    ]
    seen: set[str] = set()
    for record, candidates in (
        (root_record, ["lint", "validate:local:agent"]),
        (owner, ["typecheck", "type-check", "lint", "test", "build"]),
        (root_record, ["test", "build"]),
    ):
        for candidate in candidates:
            add_script_command(commands, seen, package_manager, record, [candidate])
    commands.append("${PM_AUDIT}")
    return commands


def enrich_next_manifest(manifest: dict[str, Any], root: Path) -> dict[str, Any]:
    """Enrich a Next.js upgrade pack manifest."""
    owner = detect_next_owner(root)
    root_record = root_manifest_record(root)
    versions = package_versions_from_records([owner, root_record], next_related_packages(owner, root_record))
    current_version = versions.get("next", "unknown")
    docs = next_doc_urls(current_version)
    doc_metadata = {label: fetch_doc_metadata(url) for label, url in docs.items()}
    doc_dates = [date for _, date in doc_metadata.values() if re.match(r"[A-Za-z]+\s+\d{1,2},\s+\d{4}", date)]

    current_major = next_major(current_version)
    owner_path = workspace_display_path(owner)
    owner_reason = (
        f"`{owner_path}` is the highest-signal Next.js owner because it declares `next`, owns Next config and route files, and carries Next-specific scripts."
    )
    manifest["target_surface"] = build_target_surface(root, owner, owner_reason=owner_reason)
    if owner_path != ".":
        basename = f"nextjs-{workspace_slug(owner)}-v{current_major}-upgrade-and-optimization"
        set_plan_identity(
            manifest,
            basename=basename,
            playbook_title=f"Next.js {owner_path} v{current_major} Upgrade And Optimization Playbook",
            operator_title=f"Next.js {owner_path} v{current_major} Upgrade And Optimization Operator Mode",
            trigger_title=f"Next.js {owner_path} v{current_major} Upgrade And Optimization Trigger Prompt",
        )
    else:
        set_plan_identity(
            manifest,
            basename=f"nextjs-v{current_major}-upgrade-and-optimization",
            playbook_title=f"Next.js v{current_major} Upgrade And Optimization Playbook",
            operator_title=f"Next.js v{current_major} Upgrade And Optimization Operator Mode",
            trigger_title=f"Next.js v{current_major} Upgrade And Optimization Trigger Prompt",
        )

    constraints, supported, unsupported = next_constraints(root, owner)
    repo_probes = next_repo_probes(root, owner)
    repo_probes["Current repo scripts and posture"] = current_next_surface_notes(root, owner, root_record)

    manifest["family_display_name"] = "Next.js"
    manifest["family_type"] = "framework"
    manifest["mode"] = "upgrade+optimize"
    manifest["related_packages"] = unique_list(next_related_packages(owner, root_record))
    manifest["current_version"] = current_version
    manifest["validated_upstream_version"] = f"Next.js {current_major}"
    manifest["validated_doc_date"] = max(doc_dates) if doc_dates else "unknown"
    manifest["repo_probes"] = repo_probes
    manifest["upstream_validation"] = {
        "Official Next.js docs": [
            f"{label}: `{url}`; title=`{title}`; last_updated=`{date}`"
            for label, url in docs.items()
            for title, date in [doc_metadata[label]]
        ],
        "Current major guidance": [
            f"Workspace `{owner_path}` currently uses `next` version `{current_version}` and maps to major `{current_major}`.",
            "Next.js 16 guidance confirms `cacheComponents`, stable `typedRoutes`, `proxy`, and version-specific codemods.",
        ],
    }
    manifest["framework_constraints"] = constraints
    manifest["supported_features"] = supported
    manifest["unsupported_features"] = unsupported
    manifest["codemod_recommendations"] = next_codemods(owner, manifest, current_version)
    manifest["verification_commands"] = next_verification_commands(root, owner, root_record, manifest)
    package_manager = manifest["repo_context"]["package_manager"]
    owner_cwd = workspace_display_path(owner)
    apply_qualification_plan(
        manifest,
        root,
        doc_urls=docs,
        source_specs=[f"next@{normalize_package_version_for_source(current_version)}"] if current_version != "unknown" else [],
        cli_checks=[
            qualification_cli_check(
                "Next.js codemod help",
                owner_cwd,
                dlx_command(package_manager, "@next/codemod@canary", "--help"),
            ),
            qualification_cli_check(
                "Next.js CLI help",
                owner_cwd,
                dlx_command(package_manager, "next", "--help"),
            ),
        ],
    )
    research_sources = next_research_sources(current_version)
    apply_research_plan(
        manifest,
        official_docs=research_sources["official_docs"],
        api_reference=research_sources["api_reference"],
        migration_guides=research_sources["migration_guides"],
        release_history=research_sources["release_history"],
        examples_cookbooks=research_sources["examples_cookbooks"],
        source_specs=[f"next@{normalize_package_version_for_source(current_version)}"] if current_version != "unknown" else [],
        repo_usage_queries=next_repo_usage_queries(owner),
        target_version=f"Next.js {current_major}",
        compatibility_rationale=(
            "Stay on the latest compatible stable Next.js 16 surface that fits the repo's runtime, "
            "static-export, and deployment constraints while adopting current official APIs."
        ),
        release_range=f"{current_version} -> Next.js {current_major}",
    )
    return manifest


def expo_sdk_major(version: str) -> str:
    """Return the Expo SDK major version string."""
    return version_major(version, "55")


def expo_doc_urls() -> dict[str, str]:
    """Return official Expo and EAS documentation URLs."""
    return {
        "upgrade guide": "https://docs.expo.dev/workflow/upgrading-expo-sdk-walkthrough/",
        "monorepos": "https://docs.expo.dev/guides/monorepos/",
        "EAS build config": "https://docs.expo.dev/build/eas-json/",
        "EAS overview": "https://docs.expo.dev/eas",
        "new architecture": "https://docs.expo.dev/guides/new-architecture/",
    }


def expo_research_sources(version: str) -> dict[str, dict[str, str]]:
    """Return categorized research sources for Expo and EAS."""
    sdk_major = expo_sdk_major(version)
    return {
        "official_docs": {
            "expo docs home": "https://docs.expo.dev/",
            "eas overview": "https://docs.expo.dev/eas",
        },
        "api_reference": {
            "expo sdk reference": "https://docs.expo.dev/versions/latest/",
            "expo router reference": "https://docs.expo.dev/versions/latest/sdk/router/",
            "eas build config": "https://docs.expo.dev/build/eas-json/",
        },
        "migration_guides": {
            "upgrade guide": "https://docs.expo.dev/workflow/upgrading-expo-sdk-walkthrough/",
            "monorepos": "https://docs.expo.dev/guides/monorepos/",
            "new architecture": "https://docs.expo.dev/guides/new-architecture/",
        },
        "release_history": {
            f"sdk {sdk_major} changelog": f"https://expo.dev/changelog/sdk-{sdk_major}",
            "expo changelog": "https://expo.dev/changelog",
        },
        "examples_cookbooks": {
            "eas workflows": "https://docs.expo.dev/eas/workflows/introduction/",
            "eas json": "https://docs.expo.dev/build/eas-json/",
        },
    }


def expo_repo_usage_queries(owner: dict[str, Any]) -> list[dict[str, str]]:
    """Return repo-usage mapping commands for Expo and EAS."""
    owner_cwd = workspace_display_path(owner)
    return [
        research_repo_usage_query("Expo manifest declarations", ".", "rg -n '\"expo\"|\"expo-router\"|\"react-native\"|\"expo-updates\"|\"expo-dev-client\"' ."),
        research_repo_usage_query("Expo config surfaces", ".", f"rg --files {shlex.quote(owner_cwd)} | rg '(app\\.json|app\\.config\\.|eas\\.json|expo-env\\.d\\.ts|metro\\.config|babel\\.config)'"),
        research_repo_usage_query("Expo and EAS script usage", ".", f"rg -n 'expo start|expo-doctor|expo install|eas ' {shlex.quote(owner_cwd)} {shlex.quote(owner['package_json_path'])}"),
    ]


def detect_expo_owner(root: Path) -> dict[str, Any]:
    """Pick the owning Expo/EAS workspace."""
    def score(record: dict[str, Any]) -> int:
        total = 0
        deps = record.get("dependencies") or {}
        data = record.get("data") or {}
        scripts = record.get("scripts") or {}
        if "expo" in deps:
            total += 10
        if "react-native" in deps:
            total += 4
        if workspace_exists_any(root, record, ["app.json", "app.config.ts", "app.config.js", "app.config.mjs"]):
            total += 6
        if workspace_exists_any(root, record, ["eas.json"]):
            total += 5
        if data.get("main") == "expo-router/entry":
            total += 3
        if any("expo " in command for command in scripts.values()):
            total += 3
        if any("eas" in name or "eas " in command for name, command in scripts.items()):
            total += 3
        return total

    return pick_best_owner(root, score)


def expo_related_packages(owner: dict[str, Any], root_record: dict[str, Any]) -> list[str]:
    """Return the related package set for an Expo/EAS wave."""
    ordered = [
        "expo",
        "expo-router",
        "expo-updates",
        "expo-dev-client",
        "react-native",
        "react-native-web",
        "react",
        "react-dom",
        "expo-doctor",
    ]
    versions = package_versions_from_records([owner, root_record], ordered)
    return unique_list([package for package in ordered if package in versions])


def expo_repo_probes(root: Path, owner: dict[str, Any]) -> dict[str, list[str]]:
    """Build Expo/EAS probe data for a workspace owner."""
    base = workspace_dir(root, owner)
    data = owner.get("data") or {}
    scripts = owner.get("scripts") or {}
    app_config = workspace_exists_any(root, owner, ["app.json", "app.config.ts", "app.config.js", "app.config.mjs"])
    eas_json = workspace_exists_any(root, owner, ["eas.json"])
    has_ios = (base / "ios").exists()
    has_android = (base / "android").exists()
    eas_profiles: list[str] = []
    if eas_json:
        try:
            payload = json.loads(safe_read_text(root / eas_json))
            build_profiles = payload.get("build") or {}
            if isinstance(build_profiles, dict):
                eas_profiles = sorted(str(key) for key in build_profiles)
        except json.JSONDecodeError:
            eas_profiles = []

    posture = [
        f"owner workspace: `{workspace_reference(owner)}`",
        f"app config: `{app_config or 'not found'}`",
        f"eas.json: `{eas_json or 'not found'}`",
        f"expo-router entry: `{'yes' if data.get('main') == 'expo-router/entry' else 'no'}`",
        f"native ios dir present: `{'yes' if has_ios else 'no'}`",
        f"native android dir present: `{'yes' if has_android else 'no'}`",
        f"`expo-doctor` script present: `{'yes' if pick_script(owner, ['doctor', 'doctor:ci']) else 'no'}`",
        f"`expo install --check` or deps check script present: `{'yes' if pick_script(owner, ['deps:check']) else 'no'}`",
        f"EAS workflows validation script present: `{'yes' if pick_script(owner, ['workflows:validate', 'workflows:contract:check']) else 'no'}`",
    ]
    inventory = [
        f"EAS build profiles: `{', '.join(eas_profiles) or 'none detected'}`",
        f"Owner scripts sample: `{', '.join(sorted(scripts)[:8]) or 'none'}`",
        f"Expo config files under owner: `{', '.join(path.name for path in source_files_under(base) if path.name in {'eas.json', 'app.json', 'app.config.ts', 'app.config.js', 'app.config.mjs'}) or 'none'}`",
    ]
    return {
        "Repo posture": posture,
        "Surface inventory": inventory,
    }


def expo_constraints(root: Path, owner: dict[str, Any]) -> tuple[list[str], list[str], list[str]]:
    """Return Expo/EAS constraints and feature posture."""
    base = workspace_dir(root, owner)
    has_native_dirs = (base / "ios").exists() or (base / "android").exists()
    constraints = [
        "Treat Expo SDK alignment as a family boundary: Expo, Expo Router, React Native, and EAS config move together.",
        "Resolve the mobile app as the owner workspace in monorepos; root tooling commands are supporting validation, not the primary app contract.",
        "Use official Expo SDK and EAS guidance before changing app config, monorepo setup, or build profiles.",
    ]
    supported = [
        "Expo SDK upgrades via `expo install` plus `expo-doctor` validation.",
        "Workspace-aware Expo monorepo setups backed by official workspaces guidance.",
        "EAS build profiles and environment-specific `eas.json` configuration.",
        "React Native New Architecture defaults in current Expo SDK guidance.",
    ]
    unsupported = [
        "Do not hand-pin Expo family packages outside the SDK support matrix without explicit proof.",
        "Do not apply bare-workflow-only steps when the workspace uses Continuous Native Generation.",
    ]
    if has_native_dirs:
        constraints.append("Native directories exist in the owner workspace; native project regeneration and platform-specific cleanup may be part of the upgrade wave.")
        supported.append("Native project cleanup and prebuild-aware validation when the workspace actually contains `ios/` or `android/`.")
    else:
        constraints.append("No native directories are present; treat this as a CNG-first workspace and skip bare-only cleanup steps unless native dirs appear.")
    return (unique_list(constraints), unique_list(supported), unique_list(unsupported))


def expo_codemods(manifest: dict[str, Any], current_version: str) -> list[str]:
    """Return Expo/EAS upgrade helper commands."""
    pm_dlx = (manifest["repo_context"].get("command_variables") or {}).get("PM_DLX", "npx")
    sdk_major = expo_sdk_major(current_version)
    return [
        f"Use `{pm_dlx} expo install expo@^${{TARGET_SDK:-{sdk_major}}}.0.0` when moving the owner workspace to a new Expo SDK.",
        f"Use `{pm_dlx} expo install --fix` immediately after the SDK bump to align Expo family versions.",
        f"Use `{pm_dlx} expo-doctor` after dependency alignment and before broad cleanup.",
    ]


def expo_verification_commands(root: Path, owner: dict[str, Any], root_record: dict[str, Any], manifest: dict[str, Any]) -> list[str]:
    """Build layered root and workspace verification commands for Expo/EAS."""
    package_manager = manifest["repo_context"]["package_manager"]
    base_path = workspace_display_path(owner)
    base_arg = shlex.quote(base_path)
    manifest_path = shlex.quote(owner["package_json_path"])
    commands = [
        "# inventory",
        f"rg -n '\"expo\"|\"expo-router\"|\"react-native\"|\"expo-updates\"' {manifest_path}",
        f"rg --files {base_arg} | rg '(app\\.json|app\\.config\\.|eas\\.json|expo-env\\.d\\.ts|metro\\.config|babel\\.config)'",
        f"rg -n 'expo start|expo-doctor|eas ' {base_arg} {manifest_path}",
        "",
        "# verification",
    ]
    seen: set[str] = set()
    for record, candidates in (
        (root_record, ["lint", "validate:local:agent"]),
        (owner, ["doctor:ci", "doctor", "deps:check", "typecheck", "lint", "test", "workflows:validate", "build:smoke:android"]),
        (root_record, ["test"]),
    ):
        for candidate in candidates:
            add_script_command(commands, seen, package_manager, record, [candidate])
    commands.append("${PM_AUDIT}")
    return commands


def enrich_expo_manifest(manifest: dict[str, Any], root: Path) -> dict[str, Any]:
    """Enrich an Expo/EAS upgrade pack manifest."""
    owner = detect_expo_owner(root)
    root_record = root_manifest_record(root)
    versions = package_versions_from_records([owner, root_record], expo_related_packages(owner, root_record))
    current_version = versions.get("expo", "unknown")
    sdk_major = expo_sdk_major(current_version)
    docs = expo_doc_urls()
    doc_metadata = {label: fetch_doc_metadata(url) for label, url in docs.items()}
    doc_dates = [date for _, date in doc_metadata.values() if re.match(r"[A-Za-z]+\s+\d{1,2},\s+\d{4}", date)]
    owner_path = workspace_display_path(owner)
    owner_reason = (
        f"`{owner_path}` is the highest-signal Expo owner because it declares `expo`, owns app config and `eas.json`, and carries Expo/EAS scripts."
    )
    manifest["target_surface"] = build_target_surface(root, owner, owner_reason=owner_reason)
    basename = (
        f"expo-eas-{workspace_slug(owner)}-sdk{sdk_major}-upgrade-and-optimization"
        if owner_path != "."
        else f"expo-eas-sdk{sdk_major}-upgrade-and-optimization"
    )
    title_prefix = f"Expo EAS {owner_path} SDK {sdk_major}" if owner_path != "." else f"Expo EAS SDK {sdk_major}"
    set_plan_identity(
        manifest,
        basename=basename,
        playbook_title=f"{title_prefix} Upgrade And Optimization Playbook",
        operator_title=f"{title_prefix} Upgrade And Optimization Operator Mode",
        trigger_title=f"{title_prefix} Upgrade And Optimization Trigger Prompt",
    )

    constraints, supported, unsupported = expo_constraints(root, owner)
    manifest["family_display_name"] = "Expo And EAS"
    manifest["family_type"] = "framework"
    manifest["mode"] = "upgrade+optimize"
    manifest["related_packages"] = unique_list(expo_related_packages(owner, root_record))
    manifest["current_version"] = current_version
    manifest["validated_upstream_version"] = f"Expo SDK {sdk_major}"
    manifest["validated_doc_date"] = max(doc_dates) if doc_dates else "unknown"
    manifest["repo_probes"] = expo_repo_probes(root, owner)
    manifest["upstream_validation"] = {
        "Official Expo docs": [
            f"{label}: `{url}`; title=`{title}`; last_updated=`{date}`"
            for label, url in docs.items()
            for title, date in [doc_metadata[label]]
        ],
        "Current Expo guidance": [
            f"Workspace `{owner_path}` currently uses `expo` version `{current_version}` and maps to SDK `{sdk_major}`.",
            "Current Expo docs emphasize workspace-aware monorepo support, SDK-aligned dependency upgrades, `expo-doctor`, and `eas.json` profile governance.",
        ],
    }
    manifest["framework_constraints"] = constraints
    manifest["supported_features"] = supported
    manifest["unsupported_features"] = unsupported
    manifest["codemod_recommendations"] = expo_codemods(manifest, current_version)
    manifest["verification_commands"] = expo_verification_commands(root, owner, root_record, manifest)
    package_manager = manifest["repo_context"]["package_manager"]
    owner_cwd = workspace_display_path(owner)
    source_specs = [f"expo@{normalize_package_version_for_source(current_version)}"] if current_version != "unknown" else []
    apply_qualification_plan(
        manifest,
        root,
        doc_urls=docs,
        source_specs=source_specs,
        cli_checks=[
            qualification_cli_check(
                "Expo Doctor help",
                owner_cwd,
                dlx_command(package_manager, "expo-doctor", "--help"),
            ),
            qualification_cli_check(
                "EAS CLI help",
                owner_cwd,
                dlx_command(package_manager, "eas-cli", "--help"),
            ),
        ],
    )
    research_sources = expo_research_sources(current_version)
    apply_research_plan(
        manifest,
        official_docs=research_sources["official_docs"],
        api_reference=research_sources["api_reference"],
        migration_guides=research_sources["migration_guides"],
        release_history=research_sources["release_history"],
        examples_cookbooks=research_sources["examples_cookbooks"],
        source_specs=source_specs,
        repo_usage_queries=expo_repo_usage_queries(owner),
        target_version=f"Expo SDK {sdk_major}",
        compatibility_rationale=(
            "Stay on the latest Expo SDK and EAS guidance compatible with the owner workspace's React Native, "
            "Expo Router, and CI or release posture."
        ),
        release_range=f"{current_version} -> Expo SDK {sdk_major}",
    )
    return manifest


def convex_doc_urls() -> dict[str, str]:
    """Return official Convex documentation URLs."""
    return {
        "CLI": "https://docs.convex.dev/cli",
        "generated code": "https://docs.convex.dev/generated-api/",
        "indexes": "https://docs.convex.dev/database/reading-data/indexes/",
        "deploy keys": "https://docs.convex.dev/cli/deploy-key-types",
        "agent mode": "https://docs.convex.dev/cli/agent-mode",
    }


def convex_research_sources() -> dict[str, dict[str, str]]:
    """Return categorized research sources for Convex."""
    return {
        "official_docs": {
            "docs home": "https://docs.convex.dev/home",
            "cli": "https://docs.convex.dev/cli",
        },
        "api_reference": {
            "generated api": "https://docs.convex.dev/generated-api/",
            "indexes": "https://docs.convex.dev/database/reading-data/indexes/",
            "react client": "https://docs.convex.dev/client/react",
        },
        "migration_guides": {
            "agent mode": "https://docs.convex.dev/cli/agent-mode",
            "deploy keys": "https://docs.convex.dev/cli/deploy-key-types",
        },
        "release_history": {
            "github releases": "https://github.com/get-convex/convex-backend/releases",
        },
        "examples_cookbooks": {
            "tutorial": "https://docs.convex.dev/tutorial/",
            "react quickstart": "https://docs.convex.dev/quickstart/react",
        },
    }


def convex_repo_usage_queries(owner: dict[str, Any]) -> list[dict[str, str]]:
    """Return repo-usage mapping commands for Convex."""
    owner_cwd = workspace_display_path(owner)
    return [
        research_repo_usage_query("Convex manifest declarations", ".", "rg -n '\"convex\"|\"convex-helpers\"|\"@convex-dev/' ."),
        research_repo_usage_query("Convex source surfaces", ".", f"rg --files {shlex.quote(owner_cwd)} | rg '(^|/)convex/|function_spec_|convex\\.json'"),
        research_repo_usage_query("Convex API usage", ".", f"rg -n 'defineTable|withIndex|convex/_generated|CONVEX_DEPLOY_KEY|CONVEX_AGENT_MODE' {shlex.quote(owner_cwd)}"),
    ]


def detect_convex_owner(root: Path) -> dict[str, Any]:
    """Pick the owning Convex workspace."""
    def score(record: dict[str, Any]) -> int:
        total = 0
        deps = record.get("dependencies") or {}
        scripts = record.get("scripts") or {}
        if "convex" in deps:
            total += 6
        if any(name.startswith("convex:") for name in scripts):
            total += 5
        if (workspace_dir(root, record) / "convex").exists():
            total += 10
        if any(dep.startswith("@convex-dev/") for dep in deps):
            total += 4
        if "convex-helpers" in deps or "convex-test" in deps:
            total += 2
        return total

    return pick_best_owner(root, score)


def convex_related_packages(owner: dict[str, Any], root_record: dict[str, Any]) -> list[str]:
    """Return the related package set for a Convex wave."""
    ordered = [
        "convex",
        "convex-helpers",
        "convex-test",
        "@convex-dev/aggregate",
        "@convex-dev/migrations",
        "@convex-dev/r2",
        "@convex-dev/rate-limiter",
        "@convex-dev/resend",
        "@convex-dev/workflow",
        "@convex-dev/workpool",
    ]
    versions = package_versions_from_records([owner, root_record], ordered)
    return unique_list([package for package in ordered if package in versions])


def convex_repo_probes(root: Path, owner: dict[str, Any]) -> dict[str, list[str]]:
    """Build Convex probe data for a workspace owner."""
    base = workspace_dir(root, owner)
    scripts = owner.get("scripts") or {}
    convex_dir = base / "convex"
    convex_files = [path for path in source_files_under(convex_dir) if path.suffix in {".ts", ".tsx", ".js", ".jsx"}] if convex_dir.exists() else []
    generated_dir = convex_dir / "_generated"
    import_patterns = {
        "defineTable": r"\bdefineTable\(",
        "withIndex": r"\.withIndex\(",
        "convex generated api": r"convex/_generated",
        "CONVEX_DEPLOY_KEY": r"CONVEX_DEPLOY_KEY",
        "CONVEX_AGENT_MODE": r"CONVEX_AGENT_MODE",
    }
    counts: dict[str, int] = {}
    for label, pattern in import_patterns.items():
        compiled = re.compile(pattern)
        total = 0
        for path in convex_files:
            total += len(compiled.findall(safe_read_text(path)))
        for name, command in scripts.items():
            total += len(compiled.findall(name))
            total += len(compiled.findall(command))
        counts[label] = total

    posture = [
        f"owner workspace: `{workspace_reference(owner)}`",
        f"`convex/` directory present: `{'yes' if convex_dir.exists() else 'no'}`",
        f"`convex/_generated` present: `{'yes' if generated_dir.exists() else 'no'}`",
        f"`convex:codegen:strict` script present: `{'yes' if 'convex:codegen:strict' in scripts else 'no'}`",
        f"`convex:verify:release` script present: `{'yes' if 'convex:verify:release' in scripts else 'no'}`",
        f"deploy-key-based scripts present: `{'yes' if any('CONVEX_DEPLOY_KEY' in command for command in scripts.values()) else 'no'}`",
        f"agent-mode script present: `{'yes' if any('CONVEX_AGENT_MODE' in command for command in scripts.values()) else 'no'}`",
    ]
    inventory = [
        f"Convex source files detected: `{len(convex_files)}`",
        f"Convex source sample: `{', '.join(path.relative_to(root).as_posix() for path in convex_files[:6]) or 'none'}`",
        f"Convex package inventory: `{', '.join(dep for dep in sorted(owner.get('dependencies', {})) if dep == 'convex' or dep.startswith('@convex-dev/') or dep.startswith('convex')) or 'none'}`",
        f"Signal counts: `{', '.join(f'{label}={count}' for label, count in counts.items())}`",
    ]
    return {
        "Repo posture": posture,
        "Surface inventory": inventory,
    }


def convex_constraints(root: Path, owner: dict[str, Any]) -> tuple[list[str], list[str], list[str]]:
    """Return Convex constraints and feature posture."""
    base = workspace_dir(root, owner)
    has_generated = (base / "convex" / "_generated").exists()
    constraints = [
        "Treat the Convex backend workspace as the owner surface even if client workspaces also depend on `convex`.",
        "Generated Convex API and data model files move with schema and function changes; do not treat codegen as optional drift.",
        "Use official Convex CLI and deploy-key guidance before changing non-interactive scripts or deployment posture.",
    ]
    supported = [
        "Typed generated API/dataModel code checked into the repo.",
        "Index-backed queries and explicit schema/index ownership.",
        "Non-interactive deploy-key-based verification and preview/deploy flows.",
        "Agent mode for isolated background-agent development where the repo already uses it.",
    ]
    unsupported = [
        "Do not leave stale generated code, schema drift, or deploy-key posture undocumented.",
        "Do not anchor the Convex pack on client-only workspaces that merely consume the client package.",
    ]
    if not has_generated:
        constraints.append("Generated Convex files are missing; any upgrade wave must explicitly decide whether to regenerate and commit them.")
    return (unique_list(constraints), unique_list(supported), unique_list(unsupported))


def convex_codemods(manifest: dict[str, Any]) -> list[str]:
    """Return Convex helper commands."""
    pm_dlx = (manifest["repo_context"].get("command_variables") or {}).get("PM_DLX", "npx")
    return [
        f"Use `{pm_dlx} convex codegen` to refresh generated API/data-model files when the upgrade changes schema or function contracts.",
        f"Use `{pm_dlx} convex dev --once` or the repo's strict wrapper scripts when validating codegen and backend health in CI-like flows.",
        f"Use `{pm_dlx} convex deploy` only through the repo's verified deploy-key or wrapper-script posture for production-facing checks.",
    ]


def convex_verification_commands(root: Path, owner: dict[str, Any], root_record: dict[str, Any], manifest: dict[str, Any]) -> list[str]:
    """Build layered root and workspace verification commands for Convex."""
    package_manager = manifest["repo_context"]["package_manager"]
    base_path = workspace_display_path(owner)
    base_arg = shlex.quote(base_path)
    manifest_path = shlex.quote(owner["package_json_path"])
    commands = [
        "# inventory",
        f"rg -n '\"convex\"|\"convex-helpers\"|\"@convex-dev/' {manifest_path}",
        f"rg --files {base_arg} | rg '(^|/)convex/|function_spec_|convex\\.json'",
        f"rg -n 'defineTable|withIndex|convex/_generated|CONVEX_DEPLOY_KEY|CONVEX_AGENT_MODE' {base_arg}",
        "",
        "# verification",
    ]
    seen: set[str] = set()
    for record, candidates in (
        (root_record, ["lint", "validate:local:agent", "convex:verify:release"]),
        (owner, ["lint", "typecheck", "convex:codegen:strict", "convex:dev:once:strict", "convex:verify:release", "test"]),
    ):
        for candidate in candidates:
            add_script_command(commands, seen, package_manager, record, [candidate])
    commands.append("${PM_AUDIT}")
    return commands


def enrich_convex_manifest(manifest: dict[str, Any], root: Path) -> dict[str, Any]:
    """Enrich a Convex upgrade pack manifest."""
    owner = detect_convex_owner(root)
    root_record = root_manifest_record(root)
    versions = package_versions_from_records([owner, root_record], convex_related_packages(owner, root_record))
    current_version = versions.get("convex", "unknown")
    current_major = version_major(current_version, "1")
    docs = convex_doc_urls()
    doc_metadata = {label: fetch_doc_metadata(url) for label, url in docs.items()}
    doc_dates = [date for _, date in doc_metadata.values() if re.match(r"[A-Za-z]+\s+\d{1,2},\s+\d{4}", date)]
    owner_path = workspace_display_path(owner)
    owner_reason = (
        f"`{owner_path}` is the highest-signal Convex owner because it owns the `convex/` directory and the Convex-specific verification scripts."
    )
    manifest["target_surface"] = build_target_surface(root, owner, owner_reason=owner_reason)
    basename = (
        f"convex-{workspace_slug(owner)}-upgrade-and-optimization"
        if owner_path != "."
        else "convex-upgrade-and-optimization"
    )
    title_prefix = f"Convex {owner_path}" if owner_path != "." else "Convex"
    set_plan_identity(
        manifest,
        basename=basename,
        playbook_title=f"{title_prefix} Upgrade And Optimization Playbook",
        operator_title=f"{title_prefix} Upgrade And Optimization Operator Mode",
        trigger_title=f"{title_prefix} Upgrade And Optimization Trigger Prompt",
    )

    constraints, supported, unsupported = convex_constraints(root, owner)
    manifest["family_display_name"] = "Convex"
    manifest["family_type"] = "framework"
    manifest["mode"] = "upgrade+optimize"
    manifest["related_packages"] = unique_list(convex_related_packages(owner, root_record))
    manifest["current_version"] = current_version
    manifest["validated_upstream_version"] = f"Convex {current_major}.x"
    manifest["validated_doc_date"] = max(doc_dates) if doc_dates else "unknown"
    manifest["repo_probes"] = convex_repo_probes(root, owner)
    manifest["upstream_validation"] = {
        "Official Convex docs": [
            f"{label}: `{url}`; title=`{title}`; last_updated=`{date}`"
            for label, url in docs.items()
            for title, date in [doc_metadata[label]]
        ],
        "Current Convex guidance": [
            f"Workspace `{owner_path}` currently uses `convex` version `{current_version}`.",
            "Current Convex docs emphasize generated code ownership, typed schema/query patterns, deploy-key-based automation, and explicit index usage.",
        ],
    }
    manifest["framework_constraints"] = constraints
    manifest["supported_features"] = supported
    manifest["unsupported_features"] = unsupported
    manifest["codemod_recommendations"] = convex_codemods(manifest)
    manifest["verification_commands"] = convex_verification_commands(root, owner, root_record, manifest)
    package_manager = manifest["repo_context"]["package_manager"]
    owner_cwd = workspace_display_path(owner)
    source_specs = [f"convex@{normalize_package_version_for_source(current_version)}"] if current_version != "unknown" else []
    apply_qualification_plan(
        manifest,
        root,
        doc_urls=docs,
        source_specs=source_specs,
        cli_checks=[
            qualification_cli_check(
                "Convex CLI help",
                owner_cwd,
                dlx_command(package_manager, "convex", "--help"),
            ),
            qualification_cli_check(
                "Convex codegen help",
                owner_cwd,
                dlx_command(package_manager, "convex", "codegen --help"),
            ),
        ],
    )
    research_sources = convex_research_sources()
    apply_research_plan(
        manifest,
        official_docs=research_sources["official_docs"],
        api_reference=research_sources["api_reference"],
        migration_guides=research_sources["migration_guides"],
        release_history=research_sources["release_history"],
        examples_cookbooks=research_sources["examples_cookbooks"],
        source_specs=source_specs,
        repo_usage_queries=convex_repo_usage_queries(owner),
        target_version=f"Convex {current_major}.x",
        compatibility_rationale=(
            "Stay on the latest compatible stable Convex family surface that preserves generated-code, "
            "schema, and deploy-key contracts used by the owner backend workspace."
        ),
        release_range=f"{current_version} -> Convex {current_major}.x",
    )
    return manifest


def turborepo_doc_urls() -> dict[str, str]:
    """Return official Turborepo documentation URLs."""
    return {
        "run reference": "https://turborepo.com/docs/reference/run",
        "package configurations": "https://turborepo.com/repo/docs/reference/package-configurations",
        "repository understanding": "https://turborepo.com/docs/crafting-your-repository/understanding-your-repository",
        "skipping tasks": "https://turborepo.com/repo/docs/core-concepts/monorepos/skipping-tasks",
    }


def turborepo_research_sources() -> dict[str, dict[str, str]]:
    """Return categorized research sources for Turborepo."""
    return {
        "official_docs": {
            "docs home": "https://turborepo.dev/docs",
            "run reference": "https://turborepo.dev/docs/reference/run",
        },
        "api_reference": {
            "package configurations": "https://turborepo.dev/docs/reference/package-configurations",
            "ls reference": "https://turborepo.dev/docs/reference/ls",
            "query reference": "https://turborepo.dev/docs/reference/query",
        },
        "migration_guides": {
            "repository understanding": "https://turborepo.dev/docs/crafting-your-repository/understanding-your-repository",
            "skipping tasks": "https://turborepo.dev/docs/crafting-your-repository/configuring-tasks",
        },
        "release_history": {
            "github releases": "https://github.com/vercel/turborepo/releases",
        },
        "examples_cookbooks": {
            "examples": "https://github.com/vercel/turborepo/tree/main/examples",
            "run reference": "https://turborepo.dev/docs/reference/run",
        },
    }


def turborepo_repo_usage_queries() -> list[dict[str, str]]:
    """Return repo-usage mapping commands for Turborepo."""
    return [
        research_repo_usage_query("Turbo manifest and config declarations", ".", "rg -n '\"turbo\"|turbo run|--affected|\"extends\"|\"$TURBO_EXTENDS$\"' ."),
        research_repo_usage_query("Turbo config files", ".", "rg --files . | rg '(^|/)turbo\\.json$'"),
        research_repo_usage_query("Turbo package graph usage", ".", "rg -n 'turbo query|turbo ls|turbo run' ."),
    ]


def generic_repo_usage_queries(anchor_package: str, owner: dict[str, Any]) -> list[dict[str, str]]:
    """Return generic repo-usage mapping commands for a package family."""
    owner_cwd = workspace_display_path(owner)
    return [
        research_repo_usage_query(
            f"{anchor_package} manifest declarations",
            ".",
            f"rg -n '\"{anchor_package}\"' .",
        ),
        research_repo_usage_query(
            f"{anchor_package} repo usage",
            ".",
            f"rg -n '{anchor_package}' {shlex.quote(owner_cwd) if owner_cwd != '.' else '.'}",
        ),
    ]


def detect_turborepo_owner(root: Path) -> tuple[dict[str, Any], list[dict[str, Any]]]:
    """Return the root owner and related package-config workspaces for Turborepo."""
    root_record = root_manifest_record(root)
    related = [
        record
        for record in workspace_manifest_records(root)
        if not record.get("is_root") and (workspace_dir(root, record) / "turbo.json").exists()
    ]
    return root_record, related


def turborepo_related_packages(root_record: dict[str, Any]) -> list[str]:
    """Return the related package set for a Turborepo wave."""
    ordered = ["turbo"]
    versions = package_versions_from_manifest(root_record, ordered)
    return unique_list([package for package in ordered if package in versions])


def turborepo_repo_probes(root: Path, owner: dict[str, Any], related: list[dict[str, Any]]) -> dict[str, list[str]]:
    """Build Turborepo probe data for the monorepo root."""
    turbo_path = root / "turbo.json"
    tasks: list[str] = []
    future_flags: list[str] = []
    global_dependencies: list[str] = []
    if turbo_path.exists():
        data = json.loads(safe_read_text(turbo_path))
        if isinstance(data.get("tasks"), dict):
            tasks = sorted(str(name) for name in data["tasks"])
        if isinstance(data.get("futureFlags"), dict):
            future_flags = sorted(str(name) for name in data["futureFlags"])
        if isinstance(data.get("globalDependencies"), list):
            global_dependencies = [str(item) for item in data["globalDependencies"]]

    posture = [
        f"owner workspace: `{workspace_reference(owner)}`",
        f"root turbo.json present: `{'yes' if turbo_path.exists() else 'no'}`",
        f"workspace package-config turbo.json count: `{len(related)}`",
        f"root `build` script delegates to turbo: `{'yes' if 'turbo run' in str((owner.get('scripts') or {}).get('build', '')) else 'no'}`",
        f"root `typecheck` script delegates to turbo directly or via wrapper: `{'yes' if 'typecheck' in (owner.get('scripts') or {}) else 'no'}`",
    ]
    inventory = [
        f"Root tasks: `{', '.join(tasks) or 'none'}`",
        f"Workspace package configs: `{', '.join(workspace_display_path(record) for record in related) or 'none'}`",
        f"Future flags: `{', '.join(future_flags) or 'none'}`",
        f"Global dependencies count: `{len(global_dependencies)}`",
    ]
    return {
        "Repo posture": posture,
        "Surface inventory": inventory,
    }


def turborepo_constraints(related: list[dict[str, Any]]) -> tuple[list[str], list[str], list[str]]:
    """Return Turborepo constraints and feature posture."""
    constraints = [
        "Treat Turborepo as a monorepo-root concern with optional package configurations that refine workspace behavior.",
        "Keep task logic in package scripts and let root scripts delegate via `turbo run` rather than re-implementing task logic at root.",
        "When package configs exist, remember that array fields replace inherited arrays unless `$TURBO_EXTENDS$` is used.",
    ]
    supported = [
        "Root `turbo.json` plus workspace package configurations with `extends: [\"//\"]`.",
        "`turbo run ... --affected` for changed-package validation.",
        "`turbo query` and `turbo ls --output=json` for graph and package inspection.",
        "Workspace-specific overrides via package `turbo.json` files.",
    ]
    unsupported = [
        "Do not move package task logic into bespoke root shell pipelines that bypass Turborepo.",
        "Do not treat workspace package configs as inert docs; they actively alter inherited task behavior.",
    ]
    if not related:
        constraints.append("No workspace package configurations are present; keep the pack focused on root task graph and delegation policy.")
    else:
        constraints.append("Workspace package configurations exist and must be audited alongside the root task graph.")
    return (unique_list(constraints), unique_list(supported), unique_list(unsupported))


def turborepo_codemods(manifest: dict[str, Any]) -> list[str]:
    """Return Turborepo helper commands."""
    pm_dlx = (manifest["repo_context"].get("command_variables") or {}).get("PM_DLX", "npx")
    return [
        f"Use `{pm_dlx} turbo ls --output=json` to verify package ownership and workspace discovery.",
        f"Use `{pm_dlx} turbo query \"query {{ packages {{ items {{ name path }} }} }}\"` to inspect package-graph state when workspace ownership is unclear.",
        f"Use `{pm_dlx} turbo run build --affected --dry=json` when validating that task filters and affected detection line up with the intended monorepo scope.",
    ]


def turborepo_verification_commands(root_record: dict[str, Any], manifest: dict[str, Any]) -> list[str]:
    """Build verification commands for a Turborepo wave."""
    package_manager = manifest["repo_context"]["package_manager"]
    commands = [
        "# inventory",
        "rg -n '\"turbo\"|turbo run|--affected|\"extends\"|\"$TURBO_EXTENDS$\"' package.json turbo.json apps packages",
        "rg --files . | rg '(^|/)turbo\\.json$'",
        "",
        "# verification",
    ]
    seen: set[str] = set()
    for candidate in ["lint", "typecheck", "test", "build", "validate:local:agent"]:
        add_script_command(commands, seen, package_manager, root_record, [candidate])
    commands.extend(
        [
            "${PM_DLX} turbo ls --output=json",
            "${PM_DLX} turbo run build --affected --dry=json",
            "${PM_AUDIT}",
        ]
    )
    return commands


def enrich_turborepo_manifest(manifest: dict[str, Any], root: Path) -> dict[str, Any]:
    """Enrich a Turborepo upgrade pack manifest."""
    owner, related = detect_turborepo_owner(root)
    versions = package_versions_from_manifest(owner, turborepo_related_packages(owner))
    current_version = versions.get("turbo", "unknown")
    current_major = version_major(current_version, "2")
    docs = turborepo_doc_urls()
    doc_metadata = {label: fetch_doc_metadata(url) for label, url in docs.items()}
    doc_dates = [date for _, date in doc_metadata.values() if re.match(r"[A-Za-z]+\s+\d{1,2},\s+\d{4}", date)]
    manifest["target_surface"] = build_target_surface(
        root,
        owner,
        owner_reason="Turborepo is owned at the monorepo root, with workspace package configurations as related task-graph surfaces.",
        related_workspaces=[owner, *related] if related else [owner],
        surface_type="repo-root-and-workspaces" if related else "repo-root",
        verification_strategy="root-plus-package-configs" if related else "root-only",
    )
    set_plan_identity(
        manifest,
        basename="turborepo-monorepo-upgrade-and-optimization",
        playbook_title="Turborepo Monorepo Upgrade And Optimization Playbook",
        operator_title="Turborepo Monorepo Upgrade And Optimization Operator Mode",
        trigger_title="Turborepo Monorepo Upgrade And Optimization Trigger Prompt",
    )

    constraints, supported, unsupported = turborepo_constraints(related)
    manifest["family_display_name"] = "Turborepo"
    manifest["family_type"] = "tooling"
    manifest["mode"] = "upgrade+optimize"
    manifest["related_packages"] = unique_list(turborepo_related_packages(owner))
    manifest["current_version"] = current_version
    manifest["validated_upstream_version"] = f"Turborepo {current_major}.x"
    manifest["validated_doc_date"] = max(doc_dates) if doc_dates else "unknown"
    manifest["repo_probes"] = turborepo_repo_probes(root, owner, related)
    manifest["upstream_validation"] = {
        "Official Turborepo docs": [
            f"{label}: `{url}`; title=`{title}`; last_updated=`{date}`"
            for label, url in docs.items()
            for title, date in [doc_metadata[label]]
        ],
        "Current Turborepo guidance": [
            f"Root workspace currently uses `turbo` version `{current_version}`.",
            "Current Turborepo docs emphasize package tasks over root task logic, package configurations via `extends`, and `--affected`/graph inspection tooling.",
        ],
    }
    manifest["framework_constraints"] = constraints
    manifest["supported_features"] = supported
    manifest["unsupported_features"] = unsupported
    manifest["codemod_recommendations"] = turborepo_codemods(manifest)
    manifest["verification_commands"] = turborepo_verification_commands(owner, manifest)
    package_manager = manifest["repo_context"]["package_manager"]
    source_specs = [f"turbo@{normalize_package_version_for_source(current_version)}"] if current_version != "unknown" else []
    apply_qualification_plan(
        manifest,
        root,
        doc_urls=docs,
        source_specs=source_specs,
        cli_checks=[
            qualification_cli_check(
                "Turbo query help",
                ".",
                dlx_command(package_manager, "turbo", "query --help"),
            ),
            qualification_cli_check(
                "Turbo ls packages",
                ".",
                dlx_command(package_manager, "turbo", "query ls"),
            ),
        ],
    )
    research_sources = turborepo_research_sources()
    apply_research_plan(
        manifest,
        official_docs=research_sources["official_docs"],
        api_reference=research_sources["api_reference"],
        migration_guides=research_sources["migration_guides"],
        release_history=research_sources["release_history"],
        examples_cookbooks=research_sources["examples_cookbooks"],
        source_specs=source_specs,
        repo_usage_queries=turborepo_repo_usage_queries(),
        target_version=f"Turborepo {current_major}.x",
        compatibility_rationale=(
            "Stay on the latest compatible stable Turborepo surface that preserves the repo's root task graph, "
            "workspace package configurations, and affected-graph validation posture."
        ),
        release_range=f"{current_version} -> Turborepo {current_major}.x",
    )
    return manifest


def detect_generic_owner(root: Path, anchor_package: str) -> dict[str, Any]:
    """Pick a primary owner for a generic package family."""
    candidates = manifests_declaring_package(root, anchor_package)
    if not candidates:
        return root_manifest_record(root)
    non_root = [record for record in candidates if not record.get("is_root")]
    if len(non_root) == 1:
        return non_root[0]
    if len(candidates) == 1:
        return candidates[0]
    root_candidate = next((record for record in candidates if record.get("is_root")), None)
    return root_candidate or sorted(non_root or candidates, key=lambda item: workspace_display_path(item))[0]


def generic_verification_commands(root: Path, owner: dict[str, Any], root_record: dict[str, Any], manifest: dict[str, Any]) -> list[str]:
    """Build generic verification commands with owner awareness."""
    package_manager = manifest["repo_context"]["package_manager"]
    owner_manifest = shlex.quote(owner["package_json_path"])
    commands = [
        "# inventory",
        f"rg -n '\"{manifest['anchor_package']}\"' {owner_manifest}",
        f"rg -n '{re.escape(manifest['anchor_package'])}' {shlex.quote(workspace_display_path(owner)) if not owner.get('is_root') else '.'}",
        "",
        "# verification",
    ]
    seen: set[str] = set()
    if not owner.get("is_root"):
        for candidate in ["lint", "validate:local:agent"]:
            add_script_command(commands, seen, package_manager, root_record, [candidate])
    for candidate in ["typecheck", "type-check", "lint", "test", "build"]:
        add_script_command(commands, seen, package_manager, owner, [candidate])
    if owner.get("is_root"):
        for candidate in ["lint", "typecheck", "type-check", "test", "build"]:
            add_script_command(commands, seen, package_manager, root_record, [candidate])
    commands.append("${PM_AUDIT}")
    return unique_list(commands)


def enrich_generic_manifest(manifest: dict[str, Any], root: Path) -> dict[str, Any]:
    """Enrich a generic manifest with owner-aware package versions and commands."""
    root_record = root_manifest_record(root)
    owner = detect_generic_owner(root, manifest["anchor_package"])
    versions = package_versions_from_records([owner, root_record], manifest["related_packages"])
    existing_qualification = manifest.get("qualification_plan") or {}
    existing_research = manifest.get("research_plan") or {}
    owner_reason = (
        f"`{workspace_display_path(owner)}` is the primary owner because it declares `{manifest['anchor_package']}` in its package manifest."
    )
    manifest["target_surface"] = build_target_surface(root, owner, owner_reason=owner_reason)
    if not owner.get("is_root"):
        base_display = manifest["family_display_name"]
        owner_path = workspace_display_path(owner)
        set_plan_identity(
            manifest,
            basename=f"{manifest['family_slug']}-{workspace_slug(owner)}-upgrade",
            playbook_title=f"{base_display} {owner_path} Upgrade Playbook",
            operator_title=f"{base_display} {owner_path} Upgrade Operator Mode",
            trigger_title=f"{base_display} {owner_path} Upgrade Trigger Prompt",
        )
    manifest["related_packages"] = unique_list(manifest["related_packages"])
    manifest["current_version"] = versions.get(manifest["anchor_package"], "unknown")
    manifest["verification_commands"] = generic_verification_commands(root, owner, root_record, manifest)
    version_source_specs = (
        [
            f"{manifest['anchor_package']}@{normalize_package_version_for_source(manifest['current_version'])}"
        ]
        if manifest["current_version"] != "unknown"
        else []
    )
    qualification_source_specs = unique_list(version_source_specs + list(existing_qualification.get("source_specs") or []))
    research_source_specs = unique_list(version_source_specs + list(existing_research.get("source_specs") or []))
    apply_qualification_plan(
        manifest,
        root,
        doc_urls=existing_qualification.get("doc_urls") or {},
        source_specs=qualification_source_specs,
        cli_checks=existing_qualification.get("cli_checks") or [],
    )
    apply_research_plan(
        manifest,
        official_docs=existing_research.get("official_docs") or {},
        api_reference=existing_research.get("api_reference") or {},
        migration_guides=existing_research.get("migration_guides") or {},
        release_history=existing_research.get("release_history") or {},
        examples_cookbooks=existing_research.get("examples_cookbooks") or {},
        source_specs=research_source_specs,
        repo_usage_queries=existing_research.get("repo_usage_queries") or generic_repo_usage_queries(manifest["anchor_package"], owner),
        target_version=str(existing_research.get("target_version") or "latest supportable stable release to be confirmed during research"),
        compatibility_rationale=str(
            existing_research.get("compatibility_rationale")
            or "Use the latest supportable stable release whose official docs, release notes, and repo usage mapping show a compatible migration path for this repository."
        ),
        release_range=str(existing_research.get("release_range") or f"{manifest['current_version']} -> latest supportable stable release"),
        target_version_policy=str(existing_research.get("target_version_policy") or "latest-compatible-stable"),
    )
    return manifest


def main() -> None:
    args = build_parser().parse_args()
    manifest_path = Path(args.manifest).expanduser().resolve()
    manifest = load_manifest(manifest_path)
    ensure_manifest_qualification_defaults(manifest)
    ensure_manifest_research_defaults(manifest)
    root = repo_path(manifest["repo_context"]["repo_root"])

    family_slug = str(manifest.get("family_slug") or "")
    anchor_package = str(manifest.get("anchor_package") or "")
    if family_slug == "nextjs" or anchor_package == "next":
        manifest = enrich_next_manifest(manifest, root)
    elif family_slug == "expo-eas" or anchor_package in {"expo", "expo-router", "eas-cli"}:
        manifest = enrich_expo_manifest(manifest, root)
    elif family_slug == "convex" or anchor_package == "convex":
        manifest = enrich_convex_manifest(manifest, root)
    elif family_slug == "turborepo" or anchor_package in {"turbo", "turborepo"}:
        manifest = enrich_turborepo_manifest(manifest, root)
    else:
        manifest = enrich_generic_manifest(manifest, root)

    output_path = Path(args.out).expanduser().resolve() if args.out else manifest_path
    dump_yaml(output_path, manifest)

    valid, errors = validate_manifest(output_path)
    if not valid:
        print("Manifest validation failed after enrichment:")
        for error in errors:
            print(f"- {error}")
        raise SystemExit(1)

    print(output_path)


if __name__ == "__main__":
    main()
