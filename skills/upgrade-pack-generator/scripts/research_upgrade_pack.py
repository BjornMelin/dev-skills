#!/usr/bin/env python3
"""Run read-only upstream and repo research for an upgrade-pack manifest."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import shlex
import subprocess
from pathlib import Path
from typing import Any
from urllib.parse import urlparse
from urllib.request import Request, urlopen

from common import (
    bundled_source_map_path,
    load_bundled_source_map,
    normalize_package_version_for_source,
    repo_path,
    root_manifest_record,
    source_map_entry,
    tool_available,
    unique_list,
    workspace_manifest_records,
)
from enrich_manifest import fetch_doc_metadata
from solve_constraints import parse_semver, satisfies, select_highest_stable
from validate_upgrade_pack import validate_manifest


URL_CATEGORIES = (
    "official_docs",
    "api_reference",
    "migration_guides",
    "release_history",
    "examples_cookbooks",
)

CONFIDENCE_SCORES = {"high": 0.95, "medium": 0.75, "low": 0.55}

CTX7_QUERIES = {
    "api_reference": "api reference public API types",
    "migration_guides": "migration guide upgrade breaking changes",
    "examples_cookbooks": "examples cookbook getting started",
}

SEMVER_PATTERN = re.compile(
    r"^v?(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$"
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", required=True, help="Path to upgrade-pack.yaml")
    parser.add_argument("--out", help="Optional alternate output path for research-snapshot.json")
    parser.add_argument("--bundle-out", help="Optional alternate output path for research-bundle.json")
    parser.add_argument("--web-findings", help="Optional path to web-research-findings.json")
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
    stdout = completed.stdout
    stderr = completed.stderr
    stdout_excerpt = clip_output(stdout)
    stderr_excerpt = clip_output(stderr)
    combined = stdout_excerpt + stderr_excerpt
    return {
        "command": command,
        "cwd": str(cwd),
        "exit_code": completed.returncode,
        "status": "ok" if completed.returncode == 0 else "failed",
        "stdout": stdout,
        "stderr": stderr,
        "stdout_excerpt": stdout_excerpt,
        "stderr_excerpt": stderr_excerpt,
        "summary": combined[:10],
    }


def run_json_command(command: str, cwd: Path) -> dict[str, Any]:
    """Run a shell command expected to return JSON."""
    result = run_shell(command, cwd)
    payload: Any = None
    if result["status"] == "ok":
        stdout = result.get("stdout")
        if not isinstance(stdout, str):
            stdout = "\n".join(result.get("stdout_excerpt") or [])
        try:
            payload = json.loads(stdout)
        except json.JSONDecodeError as exc:
            result["status"] = "failed"
            result["summary"] = [f"invalid JSON output: {exc}"]
    result["payload"] = payload
    return result


def probe_url(url: str) -> dict[str, Any]:
    """Fetch a URL and return a compact freshness/drift probe."""
    try:
        request = Request(
            url,
            headers={
                "User-Agent": "Mozilla/5.0 (compatible; upgrade-pack-generator/1.0; +https://openai.com)",
                "Accept": "text/html,application/xhtml+xml,application/xml",
            },
        )
        with urlopen(request, timeout=20) as response:
            html = response.read().decode("utf-8", errors="ignore")
            final_url = response.geturl()
            headers = {key.lower(): value for key, value in response.headers.items()}
    except Exception as exc:  # pragma: no cover - network failure handling
        return {
            "url": url,
            "status": "failed",
            "error": str(exc),
            "final_url": None,
            "title": url,
            "last_updated": "unavailable",
        }

    title_match = re.search(r"<title>(.*?)</title>", html, flags=re.IGNORECASE | re.DOTALL)
    title = re.sub(r"\s+", " ", title_match.group(1)).strip() if title_match else final_url
    title = title or url
    last_updated = headers.get("last-modified") or headers.get("date") or "unknown"
    return {
        "url": url,
        "status": "ok",
        "final_url": final_url,
        "title": title,
        "last_updated": last_updated,
    }


def freshness_from_verified_at(value: str | None) -> str:
    """Return a normalized freshness label for a YYYY-MM-DD verifiedAt field."""
    if not value:
        return "unknown"
    try:
        verified_at = dt.datetime.strptime(value, "%Y-%m-%d").date()
    except ValueError:
        return "unknown"
    age_days = (dt.datetime.now(dt.timezone.utc).date() - verified_at).days
    if age_days <= 30:
        return "fresh"
    if age_days <= 120:
        return "stale-but-validated"
    return "stale"


def normalize_repo_url(value: str | None) -> str | None:
    """Normalize repository-like URLs into a stable HTTPS form."""
    if not value or not isinstance(value, str):
        return None
    normalized = value.strip()
    if not normalized:
        return None
    normalized = normalized.replace("git+https://", "https://")
    normalized = normalized.replace("git+ssh://git@", "https://")
    normalized = normalized.replace("git://", "https://")
    if normalized.startswith("git@github.com:"):
        normalized = normalized.replace("git@github.com:", "https://github.com/", 1)
    normalized = normalized.removesuffix(".git").rstrip("/")
    return normalized


def github_slug_from_url(value: str | None) -> str | None:
    """Extract an owner/repo slug from a GitHub URL when possible."""
    normalized = normalize_repo_url(value)
    if not normalized:
        return None
    parsed = urlparse(normalized)
    if parsed.netloc.lower() not in {"github.com", "www.github.com"}:
        return None
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) < 2:
        return None
    return f"{parts[0]}/{parts[1]}"


def package_display_name(package_name: str) -> str:
    """Return the short package label used for fuzzy matching."""
    return package_name.rsplit("/", 1)[-1].lower()


def is_stable_version(version: str) -> bool:
    """Return whether a version string is stable."""
    parsed = parse_semver(version.strip())
    return bool(parsed and not parsed.is_prerelease)


def stable_versions(versions: list[str]) -> list[str]:
    """Return stable versions sorted newest-first."""
    stable = [version for version in versions if is_stable_version(version)]
    stable.sort(key=lambda version: parse_semver(version) or parse_semver("0.0.0"), reverse=True)
    return stable


def normalized_version(version: str) -> str:
    """Normalize a semver-ish string for comparison."""
    cleaned = normalize_package_version_for_source(version)
    return cleaned or version.strip()


def version_major(version: str) -> int | None:
    """Extract the major version from a semver-ish string."""
    match = SEMVER_PATTERN.match(version.strip())
    return int(match.group(1)) if match else None


def repo_dependency_versions(root: Path) -> dict[str, str]:
    """Return the first-seen dependency map across the repo's manifests."""
    versions: dict[str, str] = {}
    for record in workspace_manifest_records(root):
        for package, version in (record.get("dependencies") or {}).items():
            if package not in versions:
                versions[package] = str(version)
    return versions


def repo_node_version(root: Path) -> str | None:
    """Return the repo's preferred Node version when declared."""
    nvmrc = root / ".nvmrc"
    if nvmrc.exists():
        raw = nvmrc.read_text(encoding="utf-8", errors="ignore").strip()
        if raw:
            return raw.lstrip("v")
    root_record = root_manifest_record(root)
    engines = (root_record.get("data") or {}).get("engines") or {}
    node_engine = engines.get("node")
    if isinstance(node_engine, str) and node_engine.strip():
        return node_engine.strip()
    try:
        completed = subprocess.run(
            ["zsh", "-lc", "node --version"],
            cwd=str(root),
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return None
    if completed.returncode == 0:
        version = completed.stdout.strip().lstrip("v")
        if version:
            return version
    return None


def evaluate_constraint(range_spec: str, actual_version: str | None) -> dict[str, Any]:
    """Evaluate a semver-ish compatibility constraint."""
    if not range_spec:
        return {"status": "not-applicable", "range": range_spec, "actual_version": actual_version}
    if not actual_version:
        return {
            "status": "missing",
            "range": range_spec,
            "actual_version": actual_version,
            "reason": "repo does not declare the required dependency or runtime explicitly",
        }
    actual = parse_semver(normalized_version(actual_version))
    if actual is None:
        return {
            "status": "unknown",
            "range": range_spec,
            "actual_version": actual_version,
            "reason": "could not parse the repo version into a comparable semver",
        }
    result = satisfies(actual_version, range_spec)
    if result is True:
        return {
            "status": "compatible",
            "range": range_spec,
            "actual_version": actual_version,
        }
    if result is None:
        return {
            "status": "unknown",
            "range": range_spec,
            "actual_version": actual_version,
            "reason": "constraint could not be fully evaluated by the bundled solver",
        }
    return {
        "status": "incompatible",
        "range": range_spec,
        "actual_version": actual_version,
        "reason": "repo version major does not satisfy the package's declared constraint",
    }


def dedupe_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Dedupe entries while preserving first-seen order."""
    seen: set[str] = set()
    deduped: list[dict[str, Any]] = []
    for entry in entries:
        key = "||".join(
            str(entry.get(field) or "")
            for field in ("source_type", "url", "path", "library_id", "command", "label")
        )
        if key in seen:
            continue
        seen.add(key)
        deduped.append(entry)
    return deduped


def url_entries(urls: dict[str, str], *, source_type: str = "url") -> list[dict[str, Any]]:
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
                "source_type": source_type,
            }
        )
    return entries


def web_findings_entry_map(findings: dict[str, Any] | None) -> dict[tuple[str, str], dict[str, Any]]:
    """Index web findings by category and URL."""
    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    for item in (findings or {}).get("entries") or []:
        if not isinstance(item, dict):
            continue
        category = str(item.get("category") or "").strip()
        url = str(item.get("url") or "").strip()
        if not category or not url:
            continue
        indexed[(category, url)] = item
    return indexed


def merge_web_findings(
    entries: list[dict[str, Any]],
    *,
    category: str,
    indexed_findings: dict[tuple[str, str], dict[str, Any]],
) -> list[dict[str, Any]]:
    """Merge optional `web.run` confirmation findings into category entries."""
    merged: list[dict[str, Any]] = []
    for entry in entries:
        enriched = dict(entry)
        url = str(entry.get("url") or "").strip()
        finding = indexed_findings.get((category, url))
        if finding:
            enriched["web_confirmed"] = bool(finding.get("confirmed"))
            if finding.get("facts"):
                enriched["web_facts"] = list(finding.get("facts") or [])
            if finding.get("confirmed_at"):
                enriched["web_confirmed_at"] = str(finding.get("confirmed_at"))
        else:
            enriched.setdefault("web_confirmed", False)
        merged.append(enriched)
    return merged


def source_map_seed(
    script_path: str | Path,
    package_name: str,
) -> dict[str, Any]:
    """Resolve bundled source-map seed data for a package."""
    entry = source_map_entry(script_path, package_name)
    source_map_path = bundled_source_map_path(script_path)
    if not entry:
        return {
            "status": "missing",
            "match_status": "seed-missing",
            "source_map_path": str(source_map_path),
            "entry": None,
            "freshness": "unknown",
            "drift_findings": ["package not present in bundled source map"],
            "verified_urls": [],
        }

    verified_urls: list[dict[str, Any]] = []
    drift_findings: list[str] = []
    seed_urls = {
        "official_docs": entry.get("officialDocs"),
        "api_reference": entry.get("officialApiReference"),
        "release_history": entry.get("releaseNotesUrl") or entry.get("changelogUrl"),
        "migration_guides": entry.get("migrationGuideUrl"),
        "examples_cookbooks": entry.get("examplesUrl"),
    }
    for category, candidate in seed_urls.items():
        if not isinstance(candidate, str) or not candidate.strip():
            continue
        probe = probe_url(candidate)
        probe["category"] = category
        verified_urls.append(probe)
        if probe["status"] != "ok":
            drift_findings.append(f"{category} seed URL is unreachable: {candidate}")
        elif probe.get("final_url") and str(probe["final_url"]).rstrip("/") != candidate.rstrip("/"):
            drift_findings.append(f"{category} seed URL redirects to `{probe['final_url']}`")

    freshness = freshness_from_verified_at(str(entry.get("verifiedAt") or ""))
    if drift_findings:
        match_status = "drift-detected"
    elif freshness == "fresh":
        match_status = "fresh"
    elif freshness in {"stale", "stale-but-validated"}:
        match_status = freshness
    else:
        match_status = "matched"
    return {
        "status": "ok",
        "match_status": match_status,
        "source_map_path": str(source_map_path),
        "entry": entry,
        "freshness": freshness,
        "drift_findings": drift_findings,
        "verified_urls": verified_urls,
    }


def source_map_url_maps(seed: dict[str, Any]) -> dict[str, dict[str, str]]:
    """Translate a source-map seed into research-plan-compatible URL maps."""
    entry = seed.get("entry") or {}
    package_name = str(entry.get("packageName") or "").strip()
    release_url = entry.get("releaseNotesUrl") or entry.get("changelogUrl")
    return {
        "official_docs": {"docs home": str(entry.get("officialDocs") or "").strip()} if entry.get("officialDocs") else {},
        "api_reference": {"API reference": str(entry.get("officialApiReference") or "").strip()} if entry.get("officialApiReference") else {},
        "migration_guides": {"migration guide": str(entry.get("migrationGuideUrl") or "").strip()} if entry.get("migrationGuideUrl") else {},
        "release_history": {"release notes": str(release_url).strip()} if release_url else {},
        "examples_cookbooks": {"examples": str(entry.get("examplesUrl") or "").strip()} if entry.get("examplesUrl") else {},
        "source_specs": [package_name] if package_name else [],
    }


def collector_adapters() -> dict[str, dict[str, Any]]:
    """Report optional collector backends available in the environment."""
    deps_workbench_root = Path.home() / ".codex" / "deps-workbench"
    return {
        "bundled_source_map": {"status": "ok", "path": str(bundled_source_map_path(__file__))},
        "deps_workbench": {
            "status": "available" if deps_workbench_root.exists() else "missing",
            "path": str(deps_workbench_root),
        },
        "ctx7": {"status": "available" if tool_available("ctx7") else "missing"},
        "github": {"status": "available" if tool_available("gh") else "missing"},
        "opensrc": {"status": "available" if tool_available("opensrc") else "missing"},
    }


def provenance_score(category: str, entries: list[dict[str, Any]], seed: dict[str, Any]) -> dict[str, Any]:
    """Score provenance quality for one research category."""
    if not entries:
        return {
            "score": 0.0,
            "officiality": 0.0,
            "freshness": 0.0,
            "directness": 0.0,
            "package_specificity": 0.0,
            "source_kind": "missing",
        }

    officiality = 1.0 if any(str(entry.get("url") or "").startswith("https://") for entry in entries) else 0.6
    if any("github.com" in str(entry.get("url") or "") for entry in entries):
        officiality = max(officiality, 0.85)
    if any(entry.get("source_type", "").startswith("ctx7") for entry in entries):
        officiality = max(officiality, 0.75)
    source_confidence = CONFIDENCE_SCORES.get(
        str((seed.get("entry") or {}).get("sourceConfidence") or "").strip().lower(),
        0.6,
    )
    freshness_label = seed.get("freshness")
    freshness = {"fresh": 1.0, "stale-but-validated": 0.8, "stale": 0.55}.get(str(freshness_label), 0.65)
    directness = 1.0 if any(entry.get("web_confirmed") for entry in entries) else 0.8
    if category in {"source_evidence", "repo_usage_mapping"}:
        directness = 1.0
    package_specificity = 1.0 if any(entry.get("label") for entry in entries) else 0.7
    score = round((officiality * 0.35) + (freshness * 0.2) + (directness * 0.25) + (package_specificity * 0.2), 2)
    return {
        "score": score,
        "officiality": round(officiality, 2),
        "freshness": round(freshness, 2),
        "directness": round(directness, 2),
        "package_specificity": round(package_specificity, 2),
        "source_kind": entries[0].get("source_type", "unknown"),
        "seed_confidence": source_confidence,
    }


def build_web_research_queue(
    category_entries: dict[str, list[dict[str, Any]]],
    required_categories: list[str],
) -> list[dict[str, Any]]:
    """Build a structured `web.run` queue for official docs and API validation."""
    queue: list[dict[str, Any]] = []
    facts_map = {
        "official_docs": [
            "current upgrade guidance relevant to the repo",
            "latest compatible capability changes worth adopting",
            "breaking changes or migration constraints",
        ],
        "api_reference": [
            "current canonical API entrypoints",
            "new or deprecated API refs relevant to current repo usage",
            "exact API pages the implementation should follow",
        ],
        "migration_guides": [
            "upgrade steps and codemods",
            "breaking changes and required manual follow-ups",
        ],
        "release_history": [
            "release-note range from current to target version",
            "new capabilities and deprecations relevant to the repo",
        ],
    }
    for category in ("official_docs", "api_reference", "migration_guides", "release_history"):
        for entry in category_entries.get(category) or []:
            url = str(entry.get("url") or "").strip()
            if not url:
                continue
            queue.append(
                {
                    "category": category,
                    "label": str(entry.get("label") or category),
                    "seed_url": url,
                    "source": str(entry.get("source_type") or "discovered"),
                    "why_this_page": f"Validate the authoritative {category.replace('_', ' ')} surface for `{entry.get('label') or url}`.",
                    "facts_to_extract": facts_map.get(category, []),
                    "required_for_complete": category in required_categories,
                }
            )
    return dedupe_entries(queue)


def category_status(entries: list[dict[str, Any]]) -> str:
    """Return the normalized status for a category."""
    if not entries:
        return "missing"
    ok_count = sum(1 for entry in entries if entry.get("status") == "ok")
    if ok_count == len(entries):
        return "ok"
    if ok_count == 0:
        return "failed"
    return "partial"


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
    for spec in unique_list(specs):
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
                "source_type": "opensrc-path",
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
        elif result["exit_code"] == 2 and check["command"].lstrip().startswith("rg "):
            optional_path_errors = [
                line
                for line in stderr_excerpt
                if "No such file or directory" in line or "os error 2" in line
            ]
            if optional_path_errors and len(optional_path_errors) == len(stderr_excerpt):
                status = "ok"
                summary = stdout_excerpt[:10] or ["optional manifest paths absent"]
                stderr_excerpt = []
        entries.append(
            {
                "label": check["label"],
                "cwd": cwd_label,
                "command": check["command"],
                "exit_code": result["exit_code"],
                "status": status,
                "source_type": "repo-usage",
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


def npm_view_metadata(root: Path, package_spec: str, fields: list[str]) -> dict[str, Any]:
    """Collect npm registry metadata for a package or package@version spec."""
    field_args = " ".join(fields)
    command = f"npm view {shlex.quote(package_spec)} {field_args} --json"
    result = run_json_command(command, root)
    payload = result.get("payload")
    return {
        "package_spec": package_spec,
        "command": command,
        "status": result["status"],
        "summary": result["summary"],
        "payload": payload if isinstance(payload, dict) else {},
    }


def ctx7_library_candidates(root: Path, package_name: str) -> dict[str, Any]:
    """Resolve Context7 library candidates for a package."""
    command = f"ctx7 library {shlex.quote(package_name)} {shlex.quote('api docs')}"
    result = run_shell(command, root)
    candidates: list[dict[str, str]] = []
    if result["status"] == "ok":
        stdout = result.get("stdout")
        if not isinstance(stdout, str):
            stdout = "\n".join(result.get("stdout_excerpt") or [])
        lines = stdout.splitlines()
        current_title: str | None = None
        for line in lines:
            stripped = line.strip()
            if stripped.startswith(tuple(f"{index}." for index in range(1, 10))):
                current_title = stripped.split("Title:", 1)[-1].strip() if "Title:" in stripped else stripped
                continue
            if stripped.startswith("Title:"):
                current_title = stripped.split(":", 1)[1].strip()
                continue
            if "Context7-compatible library ID:" in stripped:
                library_id = stripped.split("Context7-compatible library ID:", 1)[1].strip()
                candidates.append(
                    {
                        "title": current_title or package_name,
                        "library_id": library_id,
                    }
                )
    return {
        "command": command,
        "status": result["status"],
        "summary": result["summary"],
        "candidates": candidates,
    }


def choose_ctx7_library(package_name: str, repo_slug: str | None, candidates: list[dict[str, str]]) -> dict[str, str] | None:
    """Pick the best Context7 library candidate for the package."""
    if not candidates:
        return None
    package_token = package_display_name(package_name)
    repo_token = (repo_slug or "").split("/", 1)[-1].lower()
    scored: list[tuple[int, dict[str, str]]] = []
    for candidate in candidates:
        library_id = candidate.get("library_id", "").lower()
        title = candidate.get("title", "").lower()
        score = 0
        if package_token and package_token in library_id:
            score += 3
        if package_token and package_token in title:
            score += 2
        if repo_token and repo_token in library_id:
            score += 2
        if library_id.startswith("/websites/"):
            score += 1
        scored.append((score, candidate))
    scored.sort(key=lambda item: item[0], reverse=True)
    return scored[0][1]


def ctx7_docs_entry(root: Path, library_id: str, category: str) -> dict[str, Any] | None:
    """Collect a single Context7 docs query for a category."""
    query = CTX7_QUERIES.get(category)
    if not query:
        return None
    command = f"ctx7 docs {shlex.quote(library_id)} {shlex.quote(query)}"
    result = run_shell(command, root)
    if result["status"] != "ok":
        return {
            "label": f"Context7 {category}",
            "library_id": library_id,
            "query": query,
            "status": "failed",
            "source_type": "ctx7-docs",
            "summary": result["summary"],
        }
    stdout = result.get("stdout")
    if not isinstance(stdout, str):
        stdout = "\n".join(result.get("stdout_excerpt") or [])
    excerpt = clip_output(stdout, limit=12)
    if not excerpt:
        return None
    return {
        "label": f"Context7 {category}",
        "library_id": library_id,
        "query": query,
        "status": "ok",
        "source_type": "ctx7-docs",
        "summary": excerpt[:6],
    }


def github_repo_metadata(root: Path, repo_slug: str | None) -> dict[str, Any]:
    """Collect GitHub repository metadata when a repo slug is known."""
    if not repo_slug:
        return {"status": "missing", "repo_slug": repo_slug, "payload": {}}
    command = f"gh api repos/{shlex.quote(repo_slug)}"
    result = run_json_command(command, root)
    payload = result.get("payload")
    return {
        "command": command,
        "repo_slug": repo_slug,
        "status": result["status"],
        "summary": result["summary"],
        "payload": payload if isinstance(payload, dict) else {},
    }


def github_release_metadata(root: Path, repo_slug: str | None) -> dict[str, Any]:
    """Collect GitHub release metadata when a repo slug is known."""
    if not repo_slug:
        return {"status": "missing", "repo_slug": repo_slug, "payload": []}
    endpoint = f"repos/{repo_slug}/releases?per_page=5"
    command = f"gh api {shlex.quote(endpoint)}"
    result = run_json_command(command, root)
    payload = result.get("payload")
    releases = payload if isinstance(payload, list) else []
    return {
        "command": command,
        "repo_slug": repo_slug,
        "status": result["status"],
        "summary": result["summary"],
        "payload": releases,
    }


def evaluate_target_version(
    manifest: dict[str, Any],
    root: Path,
    registry_metadata: dict[str, Any],
    repo_versions: dict[str, str],
    node_version: str | None,
) -> dict[str, Any]:
    """Pick the latest stable compatible target version and summarize constraints."""
    package_name = manifest["anchor_package"]
    current_version = str(manifest.get("current_version", "unknown"))
    current_normalized = normalized_version(current_version) if current_version != "unknown" else "unknown"
    payload = registry_metadata.get("payload") or {}
    dist_tags = payload.get("dist-tags") if isinstance(payload.get("dist-tags"), dict) else {}
    versions = payload.get("versions") if isinstance(payload.get("versions"), list) else []
    stable_candidates = stable_versions([str(version) for version in versions if isinstance(version, str)])
    latest_tag = str(dist_tags.get("latest")) if isinstance(dist_tags.get("latest"), str) else None
    latest_stable = latest_tag if latest_tag and is_stable_version(latest_tag) else (select_highest_stable(stable_candidates) or "unknown")
    current_major = version_major(current_normalized) if current_normalized != "unknown" else None

    candidate_pool: list[str] = []
    if latest_stable != "unknown":
        candidate_pool.append(latest_stable)
    if current_major is not None:
        candidate_pool.extend(
            version for version in stable_candidates if version_major(version) == current_major
        )
    candidate_pool.extend(stable_candidates[:8])
    if current_normalized != "unknown":
        candidate_pool.append(current_normalized)
    candidate_versions = unique_list([version for version in candidate_pool if version and version != "unknown"])

    selected_version = latest_stable
    selected_status = "compatible-with-caveats"
    compatibility_notes: list[str] = []
    peer_results: list[dict[str, Any]] = []
    engine_result: dict[str, Any] = {"status": "not-applicable"}
    inspected_candidates: list[dict[str, Any]] = []

    def compatibility_rank(status: str) -> int:
        return {"compatible": 3, "compatible-with-caveats": 2, "incompatible": 1}.get(status, 0)

    best_rank = 0
    for version in candidate_versions[:6]:
        target_metadata = npm_view_metadata(
            root,
            f"{package_name}@{version}",
            ["peerDependencies", "engines", "repository", "homepage", "bugs"],
        )
        payload = target_metadata.get("payload") or {}
        peer_dependencies = payload.get("peerDependencies") if isinstance(payload.get("peerDependencies"), dict) else {}
        peer_checks: list[dict[str, Any]] = []
        peer_notes: list[str] = []
        has_incompatible_peer = False
        for peer_name, range_spec in peer_dependencies.items():
            if not isinstance(peer_name, str) or not isinstance(range_spec, str):
                continue
            check = evaluate_constraint(range_spec, repo_versions.get(peer_name))
            check["package"] = peer_name
            peer_checks.append(check)
            if check["status"] == "incompatible":
                has_incompatible_peer = True
                peer_notes.append(f"{peer_name} expects `{range_spec}` but repo has `{repo_versions.get(peer_name)}`")
            elif check["status"] in {"missing", "unknown"}:
                peer_notes.append(f"{peer_name} compatibility needs manual confirmation (`{range_spec}`)")

        engines = payload.get("engines") if isinstance(payload.get("engines"), dict) else {}
        engine_check = evaluate_constraint(str(engines.get("node", "")), node_version)
        if target_metadata.get("status") != "ok":
            peer_notes.append("version-specific npm metadata could not be retrieved; compatibility needs manual confirmation")
        if engine_check["status"] == "incompatible":
            peer_notes.append(
                f"node engine expects `{engines.get('node')}` but repo declares `{node_version}`"
            )
        elif engine_check["status"] in {"missing", "unknown"} and engines.get("node"):
            peer_notes.append(f"node engine compatibility needs manual confirmation (`{engines.get('node')}`)")

        if has_incompatible_peer or engine_check["status"] == "incompatible":
            status = "incompatible"
        elif peer_notes:
            status = "compatible-with-caveats"
        else:
            status = "compatible"

        inspected_candidates.append(
            {
                "version": version,
                "status": status,
                "peer_checks": peer_checks,
                "engine_check": engine_check,
                "notes": peer_notes,
            }
        )
        rank = compatibility_rank(status)
        if rank > best_rank:
            best_rank = rank
            selected_version = version
            selected_status = status
            compatibility_notes = peer_notes
            peer_results = peer_checks
            engine_result = engine_check
        if status == "compatible":
            break

    recommended_related_packages = unique_list(
        [
            peer["package"]
            for peer in peer_results
            if peer.get("package") and (peer.get("actual_version") or peer.get("status") == "missing")
        ]
    )
    if selected_status == "compatible":
        compatibility_rationale = "Selected the highest stable candidate whose declared peer and engine constraints match the repo's observed posture."
    elif selected_status == "compatible-with-caveats":
        compatibility_rationale = (
            "Selected the highest stable candidate with no proven incompatibility, but some peer or engine constraints still require manual confirmation."
        )
    else:
        compatibility_rationale = (
            "Selected the latest stable candidate available from npm, but the repo posture appears incompatible with at least one declared peer or engine constraint."
        )

    release_range = f"{current_normalized} -> {selected_version}" if current_normalized != "unknown" else f"unknown -> {selected_version}"
    return {
        "current_version": current_normalized,
        "latest_stable": latest_stable,
        "selected_version": selected_version,
        "selected_status": selected_status,
        "compatibility_rationale": compatibility_rationale,
        "release_range": release_range,
        "peer_checks": peer_results,
        "engine_check": engine_result,
        "recommended_related_packages": recommended_related_packages,
        "candidate_versions": inspected_candidates,
    }


def identity_resolution(
    manifest: dict[str, Any],
    root: Path,
    source_evidence: list[dict[str, Any]],
) -> dict[str, Any]:
    """Resolve a canonical package identity and generic research sources."""
    package_name = manifest["anchor_package"]
    current_version = str(manifest.get("current_version", "unknown"))
    repo_versions = repo_dependency_versions(root)
    node_version = repo_node_version(root)
    source_seed = source_map_seed(__file__, package_name)
    seed_entry = source_seed.get("entry") or {}
    registry = npm_view_metadata(
        root,
        package_name,
        ["repository", "homepage", "bugs", "dist-tags", "versions", "peerDependencies", "engines", "name"],
    )
    registry_payload = registry.get("payload") or {}

    repository_raw = registry_payload.get("repository")
    repository_url = normalize_repo_url(str(seed_entry.get("officialGithubRepo") or "")) if seed_entry.get("officialGithubRepo") else None
    repository_directory = None
    if isinstance(repository_raw, dict):
        repository_url = normalize_repo_url(str(repository_raw.get("url") or "")) or repository_url
        repository_directory = str(repository_raw.get("directory") or "").strip() or None
    elif isinstance(repository_raw, str):
        repository_url = normalize_repo_url(repository_raw) or repository_url

    homepage_url = (
        normalize_repo_url(str(registry_payload.get("homepage") or ""))
        if registry_payload.get("homepage")
        else normalize_repo_url(str(seed_entry.get("officialDocs") or "")) if seed_entry.get("officialDocs") else None
    )
    bugs_payload = registry_payload.get("bugs")
    bugs_url = normalize_repo_url(str(bugs_payload.get("url") or "")) if isinstance(bugs_payload, dict) else normalize_repo_url(str(bugs_payload or "")) if bugs_payload else None
    repo_slug = github_slug_from_url(repository_url) or github_slug_from_url(homepage_url) or github_slug_from_url(bugs_url)

    github_repo = github_repo_metadata(root, repo_slug)
    github_releases = github_release_metadata(root, repo_slug)
    ctx7_candidates = ctx7_library_candidates(root, package_name)
    ctx7_library = choose_ctx7_library(package_name, repo_slug, ctx7_candidates.get("candidates") or [])
    ctx7_docs = {
        category: ctx7_docs_entry(root, ctx7_library["library_id"], category) if ctx7_library else None
        for category in CTX7_QUERIES
    }

    target_resolution = evaluate_target_version(manifest, root, registry, repo_versions, node_version)

    evidence: list[str] = []
    conflicts: list[str] = []
    unresolved: list[str] = []
    confidence = 0.0

    if registry["status"] == "ok":
        confidence += 0.3
        evidence.append("npm registry metadata resolved")
    else:
        unresolved.append("npm registry metadata")

    if source_seed.get("status") == "ok":
        confidence += 0.15
        evidence.append("bundled source-map seed resolved")
    else:
        unresolved.append("bundled source-map seed")

    if repository_url:
        confidence += 0.25
        evidence.append(f"repository URL: {repository_url}")
    else:
        unresolved.append("repository URL")

    if homepage_url:
        confidence += 0.15
        evidence.append(f"homepage/docs URL: {homepage_url}")
    else:
        unresolved.append("homepage/docs URL")

    if github_repo.get("status") == "ok":
        confidence += 0.1
        evidence.append(f"GitHub repo metadata resolved for `{repo_slug}`")
    elif repo_slug:
        unresolved.append("GitHub repo metadata")

    if github_releases.get("status") == "ok":
        confidence += 0.1
        evidence.append("GitHub releases surface resolved")
    elif repo_slug:
        unresolved.append("GitHub releases surface")

    if any(entry.get("status") == "ok" for entry in source_evidence):
        confidence += 0.1
        evidence.append("opensrc source path resolved")
    else:
        unresolved.append("opensrc source path")

    if ctx7_library:
        confidence += 0.05
        evidence.append(f"Context7 library candidate: {ctx7_library['library_id']}")
    else:
        unresolved.append("Context7 library candidate")

    repo_slug_candidates = {slug for slug in (github_slug_from_url(repository_url), github_slug_from_url(homepage_url), github_slug_from_url(bugs_url)) if slug}
    if len(repo_slug_candidates) > 1:
        conflicts.append(f"conflicting GitHub repo surfaces detected: {', '.join(sorted(repo_slug_candidates))}")
        confidence -= 0.2

    confidence = max(0.0, min(round(confidence, 2), 1.0))
    threshold = float((manifest.get("research_plan") or {}).get("identity_confidence_threshold", 0.75))
    if confidence >= threshold:
        identity_status = "high-confidence"
    elif confidence >= 0.5:
        identity_status = "medium-confidence"
    else:
        identity_status = "low-confidence"

    docs_root = homepage_url or repository_url
    official_docs_discovered: list[dict[str, Any]] = []
    seed_urls = source_map_url_maps(source_seed)
    if seed_urls["official_docs"]:
        official_docs_discovered.extend(url_entries(seed_urls["official_docs"], source_type="source-map-docs"))
    elif docs_root:
        official_docs_discovered.extend(url_entries({"docs home": docs_root}, source_type="discovered-docs"))

    api_reference_discovered: list[dict[str, Any]] = []
    if seed_urls["api_reference"]:
        api_reference_discovered.extend(url_entries(seed_urls["api_reference"], source_type="source-map-api"))
    elif docs_root:
        api_reference_discovered.extend(
            url_entries({"documentation surface": docs_root}, source_type="discovered-api-docs")
        )
    if ctx7_docs.get("api_reference"):
        api_reference_discovered.append(ctx7_docs["api_reference"])

    migration_discovered: list[dict[str, Any]] = []
    if seed_urls["migration_guides"]:
        migration_discovered.extend(url_entries(seed_urls["migration_guides"], source_type="source-map-migration"))
    if ctx7_docs.get("migration_guides"):
        migration_discovered.append(ctx7_docs["migration_guides"])

    release_discovered: list[dict[str, Any]] = []
    if seed_urls["release_history"]:
        release_discovered.extend(url_entries(seed_urls["release_history"], source_type="source-map-release"))
    if repo_slug:
        releases_url = f"https://github.com/{repo_slug}/releases"
        release_discovered.extend(url_entries({"github releases": releases_url}, source_type="github-releases"))
        if github_releases.get("payload"):
            release_discovered.append(
                {
                    "label": "GitHub release tags",
                    "url": releases_url,
                    "status": "ok",
                    "source_type": "github-release-metadata",
                    "summary": [
                        str(release.get("tag_name"))
                        for release in (github_releases.get("payload") or [])[:5]
                        if isinstance(release, dict) and release.get("tag_name")
                    ],
                }
            )

    example_discovered: list[dict[str, Any]] = []
    if seed_urls["examples_cookbooks"]:
        example_discovered.extend(url_entries(seed_urls["examples_cookbooks"], source_type="source-map-examples"))
    if ctx7_docs.get("examples_cookbooks"):
        example_discovered.append(ctx7_docs["examples_cookbooks"])

    source_release_entries: list[dict[str, Any]] = []
    source_migration_entries: list[dict[str, Any]] = []
    source_example_entries: list[dict[str, Any]] = []
    for entry in source_evidence:
        resolved_path = entry.get("resolved_path")
        if entry.get("status") != "ok" or not resolved_path:
            continue
        for relative in entry.get("release_note_files") or []:
            source_release_entries.append(
                {
                    "label": f"{entry['spec']} release history",
                    "path": f"{resolved_path}:{relative}",
                    "status": "ok",
                    "source_type": "opensrc-release-file",
                }
            )
        for relative in entry.get("migration_files") or []:
            source_migration_entries.append(
                {
                    "label": f"{entry['spec']} migration guide",
                    "path": f"{resolved_path}:{relative}",
                    "status": "ok",
                    "source_type": "opensrc-migration-file",
                }
            )
        for relative in entry.get("example_paths") or []:
            source_example_entries.append(
                {
                    "label": f"{entry['spec']} examples",
                    "path": f"{resolved_path}:{relative}",
                    "status": "ok",
                    "source_type": "opensrc-example-file",
                }
            )

    repository_slug = repo_slug or github_slug_from_url(str((github_repo.get("payload") or {}).get("html_url") or ""))
    return {
        "threshold": threshold,
        "identity": {
            "package_name": package_name,
            "current_version": current_version,
            "docs_url": docs_root,
            "repository_url": repository_url,
            "repository_slug": repository_slug,
            "repository_directory": repository_directory,
            "bugs_url": bugs_url,
            "ctx7_library_id": (ctx7_library or {}).get("library_id"),
            "status": identity_status,
            "confidence": confidence,
            "evidence": evidence,
            "conflicts": conflicts,
            "unresolved_surfaces": unresolved,
        },
        "target_resolution": target_resolution,
        "registry": {
            "status": registry["status"],
            "command": registry["command"],
            "summary": registry["summary"],
            "metadata": {
                "repository": repository_raw,
                "homepage": registry_payload.get("homepage"),
                "bugs": registry_payload.get("bugs"),
                "dist_tags": registry_payload.get("dist-tags"),
                "versions_total": len(registry_payload.get("versions") or []),
                "versions_tail": stable_versions(
                    [str(version) for version in (registry_payload.get("versions") or []) if isinstance(version, str)]
                )[:10],
            },
        },
        "github": {
            "repo": {
                "status": github_repo.get("status"),
                "repo_slug": repository_slug,
                "summary": github_repo.get("summary"),
                "default_branch": (github_repo.get("payload") or {}).get("default_branch"),
                "html_url": (github_repo.get("payload") or {}).get("html_url"),
            },
            "releases": {
                "status": github_releases.get("status"),
                "repo_slug": repository_slug,
                "summary": github_releases.get("summary"),
                "tags": [
                    str(release.get("tag_name"))
                    for release in (github_releases.get("payload") or [])[:10]
                    if isinstance(release, dict) and release.get("tag_name")
                ],
            },
        },
        "ctx7": {
            "resolver_status": ctx7_candidates.get("status"),
            "resolver_summary": ctx7_candidates.get("summary"),
            "candidates": ctx7_candidates.get("candidates") or [],
            "selected_library": ctx7_library,
            "docs_queries": {key: value for key, value in ctx7_docs.items() if value},
        },
        "source_map_seed": source_seed,
        "collectors": collector_adapters(),
        "repo_runtime": {
            "node_version": node_version,
            "dependency_versions": {key: repo_versions[key] for key in sorted(repo_versions) if key in unique_list([package_name] + target_resolution["recommended_related_packages"])},
        },
        "discovered_sources": {
            "official_docs": dedupe_entries(official_docs_discovered),
            "api_reference": dedupe_entries(api_reference_discovered),
            "migration_guides": dedupe_entries(migration_discovered + source_migration_entries),
            "release_history": dedupe_entries(release_discovered + source_release_entries),
            "examples_cookbooks": dedupe_entries(example_discovered + source_example_entries),
            "recommended_related_packages": target_resolution["recommended_related_packages"],
        },
    }


def research_status(
    required_categories: list[str],
    category_statuses: dict[str, str],
    identity: dict[str, Any],
    target_resolution: dict[str, Any],
    web_queue: list[dict[str, Any]],
    web_findings: dict[str, Any] | None,
) -> tuple[str, list[str]]:
    """Return the normalized overall research status and caveats."""
    caveats: list[str] = []
    statuses = [category_statuses.get(category, "missing") for category in required_categories]
    missing_categories = [category for category in required_categories if category_statuses.get(category) == "missing"]
    failed_categories = [category for category in required_categories if category_statuses.get(category) == "failed"]
    partial_categories = [category for category in required_categories if category_statuses.get(category) == "partial"]

    if missing_categories:
        caveats.append(f"missing research categories: {', '.join(missing_categories)}")
    if failed_categories:
        caveats.append(f"failed research categories: {', '.join(failed_categories)}")
    if partial_categories:
        caveats.append(f"partial research categories: {', '.join(partial_categories)}")

    identity_status = str(identity.get("status") or "unknown")
    if identity_status != "high-confidence":
        caveats.append(
            f"package identity is `{identity_status}` at confidence `{identity.get('confidence', 'unknown')}`"
        )
    target_status = str(target_resolution.get("selected_status") or "unknown")
    if target_status == "incompatible":
        caveats.append("selected target version appears incompatible with at least one peer or engine constraint")
    elif target_status == "compatible-with-caveats":
        caveats.append("selected target version still has peer or engine constraints requiring manual confirmation")

    required_web = [item for item in web_queue if item.get("required_for_complete")]
    confirmed_categories = {
        str(item.get("category") or "")
        for item in (web_findings or {}).get("entries") or []
        if isinstance(item, dict) and item.get("confirmed") and str(item.get("category") or "").strip()
    }
    missing_web = [
        item
        for item in required_web
        if str(item.get("category") or "") not in confirmed_categories
    ]
    if missing_web:
        categories = ", ".join(sorted({str(item.get("category") or "") for item in missing_web if item.get("category")}))
        caveats.append(f"required web.run confirmation is still missing for: {categories}")

    if (
        statuses
        and all(status == "ok" for status in statuses)
        and identity_status == "high-confidence"
        and target_status != "incompatible"
        and not missing_web
    ):
        return "complete", caveats

    if identity_status == "low-confidence" and not any(status in {"ok", "partial"} for status in statuses):
        return "insufficient-evidence", caveats
    if target_status == "incompatible" and not any(status == "ok" for status in statuses):
        return "insufficient-evidence", caveats
    return "partial", caveats


def snapshot_path(manifest_path: Path, manifest: dict[str, Any], explicit_out: str | None) -> Path:
    """Return the output path for the research snapshot."""
    if explicit_out:
        return Path(explicit_out).expanduser().resolve()
    research_plan = manifest.get("research_plan") or {}
    filename = str(research_plan.get("snapshot_filename") or "research-snapshot.json")
    return manifest_path.parent / filename


def bundle_path(manifest_path: Path, manifest: dict[str, Any], explicit_out: str | None) -> Path:
    """Return the output path for the raw research bundle."""
    if explicit_out:
        return Path(explicit_out).expanduser().resolve()
    research_plan = manifest.get("research_plan") or {}
    filename = str(research_plan.get("bundle_filename") or "research-bundle.json")
    return manifest_path.parent / filename


def web_findings_path(manifest_path: Path, manifest: dict[str, Any], explicit_out: str | None) -> Path:
    """Return the output path for `web.run` findings."""
    if explicit_out:
        return Path(explicit_out).expanduser().resolve()
    research_plan = manifest.get("research_plan") or {}
    filename = str(research_plan.get("web_findings_filename") or "web-research-findings.json")
    return manifest_path.parent / filename


def generate_snapshot(
    manifest: dict[str, Any],
    root: Path,
    web_findings: dict[str, Any] | None = None,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Generate the in-memory research snapshot and raw evidence bundle."""
    research_plan = manifest.get("research_plan") or {}
    current_version = str(manifest.get("current_version", "unknown"))
    source_specs = unique_list(
        list(research_plan.get("source_specs") or [])
        + ([f"{manifest['anchor_package']}@{normalize_package_version_for_source(current_version)}"] if current_version != "unknown" else [manifest["anchor_package"]])
    )

    source_evidence = source_entries(root, source_specs)
    repo_usage_mapping = repo_usage_entries(root, research_plan.get("repo_usage_queries") or [])
    identity_bundle = identity_resolution(manifest, root, source_evidence)
    discovered_sources = identity_bundle["discovered_sources"]
    target_resolution = identity_bundle["target_resolution"]
    indexed_web_findings = web_findings_entry_map(web_findings)

    category_entries: dict[str, list[dict[str, Any]]] = {
        "official_docs": merge_web_findings(dedupe_entries(
            url_entries(research_plan.get("official_docs") or {}) + list(discovered_sources.get("official_docs") or [])
        ), category="official_docs", indexed_findings=indexed_web_findings),
        "api_reference": merge_web_findings(dedupe_entries(
            url_entries(research_plan.get("api_reference") or {}) + list(discovered_sources.get("api_reference") or [])
        ), category="api_reference", indexed_findings=indexed_web_findings),
        "migration_guides": merge_web_findings(dedupe_entries(
            url_entries(research_plan.get("migration_guides") or {}) + list(discovered_sources.get("migration_guides") or [])
        ), category="migration_guides", indexed_findings=indexed_web_findings),
        "release_history": merge_web_findings(dedupe_entries(
            url_entries(research_plan.get("release_history") or {}) + list(discovered_sources.get("release_history") or [])
        ), category="release_history", indexed_findings=indexed_web_findings),
        "examples_cookbooks": merge_web_findings(dedupe_entries(
            url_entries(research_plan.get("examples_cookbooks") or {}) + list(discovered_sources.get("examples_cookbooks") or [])
        ), category="examples_cookbooks", indexed_findings=indexed_web_findings),
        "source_evidence": source_evidence,
        "repo_usage_mapping": repo_usage_mapping,
    }

    required_categories = research_plan.get("required_categories") or []
    required_web_confirmation_categories = research_plan.get("required_web_confirmation_categories") or []
    web_queue = build_web_research_queue(category_entries, required_web_confirmation_categories)
    category_statuses = {
        category: category_status(category_entries.get(category) or [])
        for category in required_categories
    }
    category_provenance = {
        category: provenance_score(category, category_entries.get(category) or [], identity_bundle.get("source_map_seed") or {})
        for category in required_categories
    }
    overall_status, caveats = research_status(
        required_categories,
        category_statuses,
        identity_bundle["identity"],
        target_resolution,
        web_queue,
        web_findings,
    )
    summary_counts = summarize_categories(category_statuses)
    confirmed_web_categories = sorted(
        {
            str(entry.get("category") or "")
            for entry in (web_findings or {}).get("entries") or []
            if isinstance(entry, dict) and entry.get("confirmed") and str(entry.get("category") or "").strip()
        }
    )

    snapshot = {
        "schema_version": 1,
        "generated_at": iso_now(),
        "family_slug": manifest["family_slug"],
        "anchor_package": manifest["anchor_package"],
        "repo_root": manifest["repo_context"]["repo_root"],
        "snapshot_filename": research_plan.get("snapshot_filename", "research-snapshot.json"),
        "bundle_filename": research_plan.get("bundle_filename", "research-bundle.json"),
        "web_findings_filename": research_plan.get("web_findings_filename", "web-research-findings.json"),
        "research_status": overall_status,
        "current_version": current_version,
        "target_version": target_resolution["selected_version"],
        "target_version_policy": research_plan.get("target_version_policy", "latest-compatible-stable"),
        "compatibility_rationale": target_resolution["compatibility_rationale"],
        "release_range": target_resolution["release_range"],
        "required_categories": required_categories,
        "required_web_confirmation_categories": required_web_confirmation_categories,
        "category_status": category_statuses,
        "category_provenance": category_provenance,
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
            "identity_status": identity_bundle["identity"]["status"],
            "identity_confidence": identity_bundle["identity"]["confidence"],
            "required_web_queue": len(required_web_confirmation_categories),
            "confirmed_web_queue": len(confirmed_web_categories),
        },
        "identity": identity_bundle["identity"],
        "source_map_seed": identity_bundle["source_map_seed"],
        "target_resolution": target_resolution,
        "recommended_related_packages": target_resolution["recommended_related_packages"],
        "official_docs": category_entries["official_docs"],
        "api_reference": category_entries["api_reference"],
        "migration_guides": category_entries["migration_guides"],
        "release_history": category_entries["release_history"],
        "examples_cookbooks": category_entries["examples_cookbooks"],
        "source_evidence": category_entries["source_evidence"],
        "repo_usage_mapping": category_entries["repo_usage_mapping"],
        "web_research_queue": web_queue,
        "web_research_findings": web_findings or {"entries": []},
        "caveats": caveats,
    }

    bundle = {
        "schema_version": 1,
        "generated_at": snapshot["generated_at"],
        "family_slug": manifest["family_slug"],
        "anchor_package": manifest["anchor_package"],
        "repo_root": manifest["repo_context"]["repo_root"],
        "snapshot_filename": snapshot["snapshot_filename"],
        "bundle_filename": snapshot["bundle_filename"],
        "web_findings_filename": snapshot["web_findings_filename"],
        "identity": identity_bundle["identity"],
        "source_map_seed": identity_bundle["source_map_seed"],
        "target_resolution": target_resolution,
        "registry": identity_bundle["registry"],
        "github": identity_bundle["github"],
        "ctx7": identity_bundle["ctx7"],
        "collectors": identity_bundle["collectors"],
        "repo_runtime": identity_bundle["repo_runtime"],
        "discovered_sources": discovered_sources,
        "source_evidence": source_evidence,
        "repo_usage_mapping": repo_usage_mapping,
        "category_entries": category_entries,
        "category_status": category_statuses,
        "category_provenance": category_provenance,
        "web_research_queue": web_queue,
        "web_research_findings": web_findings or {"entries": []},
        "caveats": caveats,
    }
    return snapshot, bundle


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
    findings_path = web_findings_path(manifest_path, manifest, args.web_findings)
    web_findings = None
    if findings_path.exists():
        web_findings = json.loads(findings_path.read_text(encoding="utf-8"))
    snapshot, bundle = generate_snapshot(manifest, root, web_findings)

    out_path = snapshot_path(manifest_path, manifest, args.out)
    out_path.write_text(json.dumps(snapshot, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    raw_bundle_path = bundle_path(manifest_path, manifest, args.bundle_out)
    raw_bundle_path.write_text(json.dumps(bundle, indent=2, sort_keys=False) + "\n", encoding="utf-8")
    print(out_path)


if __name__ == "__main__":
    main()
