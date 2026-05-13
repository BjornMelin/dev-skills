#!/usr/bin/env python3
"""Dependency upgrade intelligence orchestrator.

This script analyzes a repository, gathers outdated dependency signals,
retrieves GitHub releases/changelogs, and produces a refactor-focused report.
"""

from __future__ import annotations

import argparse
import json
import re
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from collect_deps import aggregate_dependencies, collect_dependencies
from detect_repo import detect_repo_context
from fallback_registry_fetch import collect_fallback_links
from gh_release_fetch import GitHubApiError, GitHubClient, filter_releases_between
from impact_analyzer import analyze_dependency_changes
from outdated_probe import probe_outdated
from render_report import write_reports
from repo_resolver import flush_cache as flush_registry_cache
from repo_resolver import resolve_dependency
from runtime_policy import choose_target_version
from utils import ensure_dir, now_iso, run_cmd, write_json


def _build_outdated_lookup(scan: dict[str, Any], ecosystem: str) -> dict[str, dict[str, str]]:
    if ecosystem == "npm":
        return scan.get("outdated", {}).get("js", {})
    if ecosystem == "pypi":
        return scan.get("outdated", {}).get("python", {})
    return {}


def _normalize(name: str) -> str:
    return name.strip().lower()


def _dep_matches_selector(dep_name: str, selector: str) -> bool:
    dep = _normalize(dep_name)
    sel = _normalize(selector)
    if not sel:
        return False
    if dep == sel:
        return True
    if dep.endswith("/" + sel):
        return True
    return sel in dep


def _filter_scan_dependencies(scan: dict[str, Any], selectors: list[str]) -> tuple[dict[str, Any], list[str]]:
    if not selectors:
        return scan, []
    deps = scan.get("dependencies", [])
    if not isinstance(deps, list):
        return scan, []

    warnings: list[str] = []
    selected: list[dict[str, Any]] = []
    for dep in deps:
        name = dep.get("name")
        if not isinstance(name, str):
            continue
        if any(_dep_matches_selector(name, s) for s in selectors):
            selected.append(dep)

    if not selected:
        warnings.append(f"No dependencies matched selectors: {', '.join(selectors)}")

    out = dict(scan)
    out["dependencies"] = selected
    out["targeted_dependencies"] = selectors
    return out, warnings


def _repo_usage_map_for_dependency(repo_root: Path, dep: dict[str, Any], limit: int = 120) -> dict[str, Any]:
    name = str(dep.get("name") or "")
    ecosystem = str(dep.get("ecosystem") or "")
    if not name:
        return {"summary": "No dependency name available.", "hits": [], "files": []}

    patterns: list[str] = []
    if ecosystem == "npm":
        escaped = re.escape(name)
        patterns.append(rf"from\\s+['\\\"]{escaped}['\\\"]")
        patterns.append(rf"require\\(\\s*['\\\"]{escaped}['\\\"]\\s*\\)")
        patterns.append(rf"import\\(\\s*['\\\"]{escaped}['\\\"]\\s*\\)")
        patterns.append(rf"['\\\"]{escaped}['\\\"]")
    elif ecosystem == "pypi":
        mod = name.replace("-", "_")
        patterns.append(rf"^\\s*import\\s+{re.escape(mod)}\\b")
        patterns.append(rf"^\\s*from\\s+{re.escape(mod)}\\b")
        patterns.append(rf"['\\\"]{re.escape(name)}['\\\"]")
    else:
        patterns.append(re.escape(name))

    glob_args = [
        "--glob",
        "!.git/**",
        "--glob",
        "!node_modules/**",
        "--glob",
        "!.venv/**",
        "--glob",
        "!venv/**",
        "--glob",
        "!dist/**",
        "--glob",
        "!build/**",
        "--glob",
        "!.next/**",
        "--glob",
        "!.turbo/**",
    ]

    hits: list[dict[str, Any]] = []
    seen = set()
    for pat in patterns:
        cmd = ["rg", "-n", "--no-heading", "--color", "never", "--hidden", *glob_args, pat, str(repo_root)]
        proc = run_cmd(cmd, check=False)
        if proc.returncode not in (0, 1):
            continue
        for line in (proc.stdout or "").splitlines():
            # path:line:text (text may contain colons)
            parts = line.split(":", 2)
            if len(parts) < 3:
                continue
            path, line_no, text = parts[0], parts[1], parts[2]
            key = (path, line_no, text)
            if key in seen:
                continue
            seen.add(key)
            hits.append({"path": path, "line": int(line_no) if line_no.isdigit() else None, "text": text.strip()})
            if len(hits) >= limit:
                break
        if len(hits) >= limit:
            break

    files = sorted({h["path"] for h in hits if isinstance(h.get("path"), str)})
    summary = f"Found {len(hits)} reference hits across {len(files)} files."
    if not hits:
        summary = "No direct usage references found with default patterns; validate dynamic/runtime usage manually."
    return {"summary": summary, "hits": hits, "files": files}


def run_scan(repo_root: Path) -> dict[str, Any]:
    ctx = detect_repo_context(repo_root)
    dep_rows = collect_dependencies(ctx)
    deps = aggregate_dependencies(dep_rows)
    outdated = probe_outdated(ctx)

    return {
        "generated_at": now_iso(),
        "repo_root": str(repo_root.resolve()),
        "repo_context": ctx,
        "dependencies": deps,
        "outdated": outdated,
    }


def _extract_compare_summary(compare_obj: dict[str, Any] | None) -> str:
    if not compare_obj:
        return ""
    commits = compare_obj.get("commits")
    if not isinstance(commits, list):
        return ""
    lines: list[str] = []
    for c in commits[:25]:
        if not isinstance(c, dict):
            continue
        sha = str(c.get("sha") or "")[:7]
        msg = ""
        commit = c.get("commit")
        if isinstance(commit, dict):
            m = commit.get("message")
            if isinstance(m, str):
                msg = m.splitlines()[0]
        if msg:
            lines.append(f"- {sha}: {msg}")
    return "\n".join(lines)


def _find_tag_for_version(tags: list[dict[str, Any]], version: str | None) -> str | None:
    if not version:
        return None
    candidates = {version, f"v{version}"}
    for tag in tags:
        name = tag.get("name")
        if isinstance(name, str) and name in candidates:
            return name
    for tag in tags:
        name = tag.get("name")
        if isinstance(name, str) and version in name:
            return name
    return None


def _enrich_one_dependency(
    dep: dict[str, Any],
    repo_context: dict[str, Any],
    outdated_lookup: dict[str, dict[str, str]],
    mode: str,
    compatibility_policy: str,
) -> tuple[dict[str, Any], list[str]]:
    warnings: list[str] = []

    resolved = resolve_dependency(dep)
    target = choose_target_version(
        dep,
        outdated_lookup,
        resolved,
        repo_context,
        compatibility_policy=compatibility_policy,
    )

    row: dict[str, Any] = {
        "ecosystem": dep.get("ecosystem"),
        "name": dep.get("name"),
        "specs": dep.get("specs") or [],
        "contexts": dep.get("contexts") or [],
        "current_version": target.get("current"),
        "latest_available": target.get("latest_available"),
        "target_version": target.get("target"),
        "target_reason": target.get("reason"),
        "outdated_source": target.get("outdated_source"),
        "resolved": {
            "source_repo": resolved.get("source_repo"),
            "source_repo_url": resolved.get("source_repo_url"),
            "links": resolved.get("links") or {},
        },
        "release_notes": [],
        "changelog_text": "",
        "fallback_links": [],
        "source_links": [],
    }

    source_repo = resolved.get("source_repo")
    if not isinstance(source_repo, str) or "/" not in source_repo:
        row["fallback_links"] = collect_fallback_links(dep, resolved)
        row["source_links"] = row["fallback_links"]
        return row, warnings

    owner, repo = source_repo.split("/", 1)
    gh = GitHubClient(mode=mode)

    try:
        releases = gh.get_releases(owner, repo)
        selected = filter_releases_between(releases, row.get("current_version"), row.get("target_version"), max_items=25)
        row["release_notes"] = selected

        changelog = gh.get_changelog(owner, repo)
        if changelog:
            row["changelog_text"] = changelog.get("text") or ""
            if changelog.get("html_url"):
                row["source_links"].append({"label": "changelog", "url": changelog["html_url"]})

        # If no releases were found in range, attempt tag compare notes.
        if not row["release_notes"]:
            tags = gh.get_tags(owner, repo)
            base = _find_tag_for_version(tags, row.get("current_version"))
            head = _find_tag_for_version(tags, row.get("target_version"))
            if base and head and base != head:
                compare_obj = gh.get_compare(owner, repo, base, head)
                compare_text = _extract_compare_summary(compare_obj)
                if compare_text:
                    row["release_notes"] = [
                        {
                            "name": f"Compare {base}...{head}",
                            "tag_name": f"{base}...{head}",
                            "version": row.get("target_version"),
                            "published_at": None,
                            "html_url": f"https://github.com/{owner}/{repo}/compare/{base}...{head}",
                            "body": compare_text,
                            "draft": False,
                            "prerelease": False,
                        }
                    ]

        if not row["release_notes"]:
            warnings.append(f"No GitHub releases/compare notes found for {source_repo} in selected range.")

    except GitHubApiError as exc:
        warnings.append(f"GitHub API error for {source_repo}: {exc}")

    fallback = collect_fallback_links(dep, resolved)
    row["fallback_links"] = fallback

    links: list[dict[str, str]] = []
    links.append({"label": "repository", "url": f"https://github.com/{owner}/{repo}"})
    for rel in row.get("release_notes") or []:
        url = rel.get("html_url")
        if isinstance(url, str) and url.startswith("http"):
            links.append({"label": f"release:{rel.get('tag_name') or rel.get('name')}", "url": url})
    links.extend(fallback)

    deduped: list[dict[str, str]] = []
    seen: set[str] = set()
    for item in links:
        url = item.get("url")
        if not url or url in seen:
            continue
        seen.add(url)
        deduped.append(item)
    row["source_links"] = deduped

    gh.flush_cache()
    return row, warnings


def run_enrich(
    scan: dict[str, Any],
    mode: str = "safe",
    max_concurrency: int = 3,
    compatibility_policy: str = "runtime-pinned",
    deep_repo_map: bool = False,
) -> dict[str, Any]:
    repo_context = scan["repo_context"]
    deps = scan["dependencies"]

    warnings: list[str] = list(scan.get("outdated", {}).get("warnings", []))
    command_traces = list(scan.get("outdated", {}).get("command_traces", []))

    enriched: list[dict[str, Any]] = []
    rate_limited: list[dict[str, Any]] = []

    def enrich_task(dep: dict[str, Any]) -> tuple[dict[str, Any], list[str]]:
        lookup = _build_outdated_lookup(scan, dep.get("ecosystem"))
        return _enrich_one_dependency(dep, repo_context, lookup, mode=mode, compatibility_policy=compatibility_policy)

    if mode == "fast" and max_concurrency > 1:
        with ThreadPoolExecutor(max_workers=max_concurrency) as pool:
            future_map = {pool.submit(enrich_task, dep): dep for dep in deps}
            for fut in as_completed(future_map):
                dep = future_map[fut]
                try:
                    row, w = fut.result()
                    enriched.append(row)
                    warnings.extend(w)
                    if any("rate limit" in x.lower() for x in w):
                        rate_limited.append(dep)
                except Exception as exc:
                    warnings.append(f"Fast mode error for {dep.get('name')}: {exc}")
                    rate_limited.append(dep)

        # Auto-fallback for dependencies that hit limits/errors in fast mode.
        if rate_limited:
            warnings.append(
                f"Fast mode fallback: re-running {len(rate_limited)} dependencies serially with safe mode due to rate-limit/errors."
            )
            # Remove any partial rows for those deps first.
            retry_keys = {(d.get("ecosystem"), d.get("name")) for d in rate_limited}
            enriched = [r for r in enriched if (r.get("ecosystem"), r.get("name")) not in retry_keys]
            for dep in rate_limited:
                lookup = _build_outdated_lookup(scan, dep.get("ecosystem"))
                row, w = _enrich_one_dependency(
                    dep,
                    repo_context,
                    lookup,
                    mode="safe",
                    compatibility_policy=compatibility_policy,
                )
                enriched.append(row)
                warnings.extend(w)
    else:
        for dep in deps:
            lookup = _build_outdated_lookup(scan, dep.get("ecosystem"))
            row, w = _enrich_one_dependency(
                dep,
                repo_context,
                lookup,
                mode="safe",
                compatibility_policy=compatibility_policy,
            )
            enriched.append(row)
            warnings.extend(w)

    enriched.sort(key=lambda x: (x.get("ecosystem") or "", x.get("name") or ""))

    return {
        "generated_at": now_iso(),
        "repo_root": scan["repo_root"],
        "repo_context": repo_context,
        "mode": mode,
        "targeted_dependencies": scan.get("targeted_dependencies", []),
        "deep_repo_map": deep_repo_map,
        "dependencies": enriched,
        "warnings": warnings,
        "command_traces": command_traces,
    }


def run_analyze(enriched: dict[str, Any]) -> dict[str, Any]:
    repo_root = Path(enriched["repo_root"])
    deep_repo_map = bool(enriched.get("deep_repo_map"))
    analyzed: list[dict[str, Any]] = []
    for dep in enriched["dependencies"]:
        impact = analyze_dependency_changes(dep)
        merged = dict(dep)
        merged.update(impact)
        if deep_repo_map:
            usage = _repo_usage_map_for_dependency(repo_root, merged)
            merged["repo_usage"] = usage
            if usage.get("files"):
                merged.setdefault("refactor_actions", [])
                merged["refactor_actions"].append(
                    f"Update all usage points in {len(usage.get('files') or [])} files identified by repo impact scan."
                )
                merged["refactor_actions"].append(
                    "Refactor imports/usages first, then run tests for modules listed in repo impact map."
                )
        analyzed.append(merged)

    analyzed.sort(key=lambda x: (x.get("ecosystem") or "", x.get("name") or ""))

    return {
        "generated_at": now_iso(),
        "repo_root": enriched["repo_root"],
        "repo_context": enriched["repo_context"],
        "mode": enriched.get("mode", "safe"),
        "targeted_dependencies": enriched.get("targeted_dependencies", []),
        "deep_repo_map": deep_repo_map,
        "dependencies": analyzed,
        "warnings": enriched.get("warnings", []),
        "command_traces": enriched.get("command_traces", []),
    }


def run_report(
    analyzed: dict[str, Any],
    out_dir: Path,
    compatibility_policy: str = "runtime-pinned",
) -> dict[str, Any]:
    paths = write_reports(
        out_dir=out_dir,
        repo_root=analyzed["repo_root"],
        repo_context=analyzed["repo_context"],
        dependencies=analyzed["dependencies"],
        mode=analyzed.get("mode", "safe"),
        compatibility_policy=compatibility_policy,
        command_traces=analyzed.get("command_traces", []),
        warnings=analyzed.get("warnings", []),
        targeted_dependencies=analyzed.get("targeted_dependencies", []),
        deep_repo_map=bool(analyzed.get("deep_repo_map")),
    )
    return {
        "generated_at": now_iso(),
        "report_paths": paths,
        "warnings": analyzed.get("warnings", []),
    }


def run_rate_limit_diag() -> dict[str, Any]:
    gh = GitHubClient(mode="safe")
    data = gh.get_rate_limit()
    gh.flush_cache()
    return data


def save_stage_json(out_dir: Path, name: str, payload: dict[str, Any]) -> str:
    ensure_dir(out_dir)
    path = out_dir / f"{name}.json"
    write_json(path, payload)
    return str(path)


def main() -> None:
    parser = argparse.ArgumentParser(description="GitHub dependency intelligence orchestrator")
    sub = parser.add_subparsers(dest="command", required=True)

    def add_common(p: argparse.ArgumentParser) -> None:
        p.add_argument("--repo", default=".", help="Target repository root (default: current directory)")
        p.add_argument("--out", default="reports", help="Output directory relative to target repo")
        p.add_argument("--mode", choices=["safe", "fast"], default="safe", help="Execution mode")
        p.add_argument("--max-concurrency", type=int, default=3, help="Fast mode worker cap")
        p.add_argument(
            "--dependency",
            action="append",
            default=[],
            help="Dependency selector (repeatable). Supports exact or partial name match.",
        )
        p.add_argument(
            "--deep-repo-map",
            action="store_true",
            help="Run repo-wide usage mapping with rg and include impacted files/usages in report.",
        )
        p.add_argument(
            "--compatibility-policy",
            default="runtime-pinned",
            choices=["runtime-pinned", "semver-only", "always-latest"],
            help="Target version selection policy",
        )

    add_common(sub.add_parser("scan", help="Detect repo and collect dependencies/outdated data"))
    add_common(sub.add_parser("enrich", help="Scan + enrich dependencies with registry and GitHub release metadata"))
    add_common(sub.add_parser("analyze", help="Scan + enrich + impact analysis"))

    p_report = sub.add_parser("report", help="Scan + enrich + analyze + report outputs")
    add_common(p_report)

    p_full = sub.add_parser("full", help="Same as report")
    add_common(p_full)
    p_package = sub.add_parser("package", help="Single-dependency comprehensive upgrade spec")
    add_common(p_package)

    sub.add_parser("rate-limit", help="Show current GitHub API rate-limit status")

    args = parser.parse_args()

    if args.command == "rate-limit":
        print(json.dumps(run_rate_limit_diag(), indent=2))
        return

    repo_root = Path(args.repo).resolve()
    out_dir = repo_root / args.out

    scan = run_scan(repo_root)
    selectors = list(args.dependency or [])
    if selectors:
        scan, filter_warnings = _filter_scan_dependencies(scan, selectors)
        scan.setdefault("outdated", {}).setdefault("warnings", []).extend(filter_warnings)
    if args.command == "package":
        if not selectors:
            raise SystemExit("`package` requires at least one --dependency selector")
        if not scan.get("dependencies"):
            raise SystemExit(f"No dependencies matched selector(s): {', '.join(selectors)}")

    if args.command == "scan":
        stage_path = save_stage_json(out_dir, "gh-deps-intel-scan", scan)
        print(json.dumps({"scan": stage_path, "summary": {"dependencies": len(scan['dependencies'])}}, indent=2))
        flush_registry_cache()
        return

    enriched = run_enrich(
        scan,
        mode=args.mode,
        max_concurrency=max(1, int(args.max_concurrency)),
        compatibility_policy=args.compatibility_policy,
        deep_repo_map=bool(args.deep_repo_map or args.command == "package"),
    )
    if args.command == "enrich":
        stage_path = save_stage_json(out_dir, "gh-deps-intel-enrich", enriched)
        print(json.dumps({"enrich": stage_path, "summary": {"dependencies": len(enriched['dependencies'])}}, indent=2))
        flush_registry_cache()
        return

    analyzed = run_analyze(enriched)
    if args.command == "analyze":
        stage_path = save_stage_json(out_dir, "gh-deps-intel-analyze", analyzed)
        print(json.dumps({"analyze": stage_path, "summary": {"dependencies": len(analyzed['dependencies'])}}, indent=2))
        flush_registry_cache()
        return

    if args.command in {"report", "full", "package"}:
        report = run_report(analyzed, out_dir, compatibility_policy=args.compatibility_policy)
        print(json.dumps(report, indent=2))
        flush_registry_cache()
        return


if __name__ == "__main__":
    main()
