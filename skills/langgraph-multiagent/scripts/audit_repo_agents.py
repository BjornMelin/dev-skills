#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import platform
import re
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime, timezone
from importlib import metadata
from pathlib import Path
from typing import Iterable


EXCLUDE_GLOBS: tuple[str, ...] = (
    "!.git/*",
    "!.venv/*",
    "!venv/*",
    "!__pycache__/*",
    "!node_modules/*",
    "!dist/*",
    "!build/*",
    "!.mypy_cache/*",
    "!.ruff_cache/*",
    "!*.min.*",
)


@dataclass(frozen=True)
class Finding:
    key: str
    title: str
    why: str
    recommendation: str
    patterns: tuple[str, ...]


FINDINGS: tuple[Finding, ...] = (
    Finding(
        key="deprecated-create-react-agent",
        title="Deprecated `langgraph.prebuilt.create_react_agent` usage",
        why="LangGraph v1 deprecates `create_react_agent` in favor of LangChain v1 `create_agent` (middleware, runtime context, streamlined API).",
        recommendation="Migrate to `langchain.agents.create_agent` and re-map pre/post hooks to middleware.",
        patterns=(
            r"\bfrom\s+langgraph\.prebuilt\s+import\s+create_react_agent\b",
            r"\bcreate_react_agent\s*\(",
        ),
    ),
    Finding(
        key="langgraph-supervisor",
        title="`langgraph-supervisor` / `langgraph-supervisor-py` detected",
        why="Supervisor libs are maintained to help upgrade older code, but current guidance generally prefers building the supervisor pattern directly via tools/subagents.",
        recommendation="Prefer tool-calling supervisor/subagents; keep the library only if it materially reduces complexity and is compatible with your pinned versions.",
        patterns=(
            r"\blanggraph[_-]supervisor\b",
            r"\bfrom\s+langgraph_supervisor\b",
            r"\bcreate_supervisor\s*\(",
        ),
    ),
    Finding(
        key="llamaindex-agents",
        title="LlamaIndex agents detected",
        why="Mixed agent frameworks often duplicate orchestration, memory, and tool abstractions, increasing maintenance and complicating observability.",
        recommendation="If migrating to LangGraph/LangChain, keep LlamaIndex only where it’s strictly needed (e.g., specific index/query features), otherwise port agent orchestration.",
        patterns=(
            r"\bllama[_-]?index\b",
            r"\bfrom\s+llama_index\b",
        ),
    ),
    Finding(
        key="crewai",
        title="CrewAI detected",
        why="Crew/task abstractions can be mapped to supervisor/subagents or orchestrator-worker graphs for better durability and observability.",
        recommendation="Plan a staged migration: isolate tool layer first, then port roles/tasks into LangChain tools + LangGraph control flow.",
        patterns=(r"\bcrewai\b",),
    ),
    Finding(
        key="agno",
        title="Agno detected",
        why="Agno-style agent composition often overlaps with LangGraph’s orchestration primitives.",
        recommendation="Port agent wiring into LangGraph nodes/edges; preserve tools and prompts with tests to keep behavior stable.",
        patterns=(r"\bagno\b",),
    ),
    Finding(
        key="openai-agents",
        title="OpenAI Agents / Responses tool-calling patterns detected",
        why="OpenAI-first agent stacks can usually be mapped to LangChain models + tools + middleware while keeping provider flexibility.",
        recommendation="Extract tools into LangChain `@tool`/BaseTool, then rebuild orchestration in LangChain/LangGraph; add provider-agnostic tracing.",
        patterns=(
            r"\bopenai\.agents\b",
            r"\bfrom\s+openai\s+import\s+OpenAI\b",
            r"\bresponses\.create\b",
        ),
    ),
)


KEY_DISTS: tuple[str, ...] = (
    "langgraph",
    "langchain",
    "langchain-core",
    "langgraph-supervisor",
    "llama-index",
    "crewai",
    "agno",
    "openai",
    "opentelemetry-api",
    "opentelemetry-sdk",
)


def _installed_version(dist: str) -> str | None:
    try:
        return metadata.version(dist)
    except metadata.PackageNotFoundError:
        return None


def _read_text_if_exists(path: Path, *, max_bytes: int = 512_000) -> str | None:
    if not path.exists() or not path.is_file():
        return None
    data = path.read_bytes()
    if len(data) > max_bytes:
        data = data[:max_bytes]
    return data.decode("utf-8", errors="replace")


def _parse_pyproject_deps(pyproject_text: str) -> dict[str, str]:
    deps: dict[str, str] = {}
    try:
        import tomllib  # py311+
    except Exception:
        return deps

    try:
        parsed = tomllib.loads(pyproject_text)
    except Exception:
        return deps

    project = parsed.get("project") or {}
    for dep in project.get("dependencies") or []:
        if not isinstance(dep, str):
            continue
        # Best-effort: "name[extra] >=1.2" -> "name"
        name = re.split(r"\s|;|==|~=|!=|<=|>=|<|>|=", dep, maxsplit=1)[0]
        name = name.split("[", 1)[0].strip()
        if name:
            deps[name] = dep
    return deps


def _rg_available() -> bool:
    try:
        subprocess.run(["rg", "--version"], check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        return True
    except Exception:
        return False


def _run_rg(pattern: str, root: Path) -> list[str]:
    cmd = ["rg", "-n", "--no-heading", "--color", "never"]
    for glob in EXCLUDE_GLOBS:
        cmd.extend(["--glob", glob])
    cmd.extend([pattern, str(root)])
    proc = subprocess.run(cmd, check=False, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if proc.returncode not in (0, 1):  # 1 means no matches
        return [f"[rg error] {proc.stderr.strip()}".strip()]
    lines = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    return lines


def _iter_code_files(root: Path) -> Iterable[Path]:
    ex_dirs = {".git", ".venv", "venv", "__pycache__", "node_modules", "dist", "build", ".mypy_cache", ".ruff_cache"}
    ex_suffixes = {".min.js", ".min.css"}
    for path in root.rglob("*"):
        if any(part in ex_dirs for part in path.parts):
            continue
        if not path.is_file():
            continue
        if path.suffix not in {".py", ".pyi", ".md", ".txt", ".toml", ".yaml", ".yml", ".json", ".ts", ".tsx", ".js", ".jsx"}:
            continue
        if any(path.name.endswith(suf) for suf in ex_suffixes):
            continue
        yield path


def _search_fallback(pattern: re.Pattern[str], root: Path) -> list[str]:
    hits: list[str] = []
    for file_path in _iter_code_files(root):
        try:
            text = file_path.read_text(encoding="utf-8", errors="replace")
        except Exception:
            continue
        for idx, line in enumerate(text.splitlines(), start=1):
            if pattern.search(line):
                rel = file_path.relative_to(root)
                hits.append(f"{rel}:{idx}:{line.strip()}")
                if len(hits) >= 200:
                    return hits + ["[truncated]"]
    return hits


def _collect_matches(root: Path) -> dict[str, list[str]]:
    use_rg = _rg_available()
    results: dict[str, list[str]] = {}
    for finding in FINDINGS:
        finding_hits: list[str] = []
        for pat in finding.patterns:
            if use_rg:
                finding_hits.extend(_run_rg(pat, root))
            else:
                finding_hits.extend(_search_fallback(re.compile(pat), root))
        # Deduplicate while preserving order
        seen: set[str] = set()
        deduped: list[str] = []
        for h in finding_hits:
            if h in seen:
                continue
            seen.add(h)
            deduped.append(h)
        results[finding.key] = deduped
    return results


def _md_escape(s: str) -> str:
    return s.replace("|", "\\|")


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit a repository for agent frameworks and deprecated LangGraph/LangChain patterns.")
    parser.add_argument("--root", default=".", help="Repository root to scan (default: .)")
    parser.add_argument("--out", default="agent_audit_report.md", help="Output markdown file path (default: agent_audit_report.md)")
    parser.add_argument("--json", dest="json_out", default=None, help="Optional JSON output file for machine parsing.")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    out = Path(args.out).resolve()
    now = datetime.now(timezone.utc).isoformat()

    versions = {dist: _installed_version(dist) for dist in KEY_DISTS}
    pyproject_path = root / "pyproject.toml"
    pyproject_text = _read_text_if_exists(pyproject_path)
    pyproject_deps = _parse_pyproject_deps(pyproject_text) if pyproject_text else {}

    matches = _collect_matches(root)

    detected_frameworks: list[str] = []
    if versions.get("langgraph"):
        detected_frameworks.append(f"langgraph=={versions['langgraph']}")
    if versions.get("langchain"):
        detected_frameworks.append(f"langchain=={versions['langchain']}")
    if versions.get("llama-index"):
        detected_frameworks.append(f"llama-index=={versions['llama-index']}")
    if versions.get("crewai"):
        detected_frameworks.append(f"crewai=={versions['crewai']}")
    if versions.get("agno"):
        detected_frameworks.append(f"agno=={versions['agno']}")

    findings_present = [f for f in FINDINGS if matches.get(f.key)]

    report: list[str] = []
    report.append("# Agent stack audit report")
    report.append("")
    report.append(f"- Generated: `{now}`")
    report.append(f"- Root: `{root}`")
    report.append(f"- Host: `{platform.platform()}`")
    report.append(f"- Python: `{sys.version.split()[0]}`")
    report.append("")
    report.append("## Executive summary")
    report.append("")
    if detected_frameworks:
        report.append(f"- Detected frameworks (installed): {', '.join(f'`{x}`' for x in detected_frameworks)}")
    else:
        report.append("- Detected frameworks (installed): `none detected via importlib.metadata` (may still be present via lockfiles or vendoring)")
    report.append(f"- Deprecated / migration-relevant findings: `{len(findings_present)}`")
    report.append("")

    report.append("## Dependency signals (pyproject.toml)")
    report.append("")
    if pyproject_text is None:
        report.append("- `pyproject.toml` not found.")
    elif not pyproject_deps:
        report.append("- No `[project.dependencies]` parsed (dynamic/other tooling may be used).")
    else:
        for name, spec in sorted(pyproject_deps.items(), key=lambda kv: kv[0].lower()):
            if any(k in name.lower() for k in ("langgraph", "langchain", "llama", "crewai", "agno", "openai", "opentelemetry")):
                report.append(f"- `{_md_escape(spec)}`")
    report.append("")

    report.append("## Findings")
    report.append("")
    if not findings_present:
        report.append("- No matching deprecated patterns or alternate agent frameworks were detected by this script.")
    else:
        report.append("| Finding | Why it matters | Recommendation |")
        report.append("|---|---|---|")
        for f in findings_present:
            report.append(
                f"| **{_md_escape(f.title)}** | {_md_escape(f.why)} | {_md_escape(f.recommendation)} |"
            )
    report.append("")

    for f in findings_present:
        report.append(f"### {f.title}")
        report.append("")
        report.append(f"- Why: {f.why}")
        report.append(f"- Recommended action: {f.recommendation}")
        report.append("")
        report.append("Matches:")
        report.append("")
        for line in matches.get(f.key, [])[:200]:
            report.append(f"- `{_md_escape(line)}`")
        if len(matches.get(f.key, [])) > 200:
            report.append("- `[truncated]`")
        report.append("")

    report.append("## Suggested next steps")
    report.append("")
    report.append("- Use the LangGraph v1 + LangChain v1 migration guides to confirm the correct replacements for your pinned versions.")
    report.append("- If migrating multi-agent supervisors: prefer supervisor/subagents via tool-calling (context isolation) unless you have a strong reason to keep `langgraph-supervisor`.")
    report.append("- Add regression tests around tool calls and state transitions before changing orchestration.")
    report.append("- If behavior is ambiguous, snapshot dependency source via `opensrc/` using `scripts/opensrc_snapshot.py`.")
    report.append("")

    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(report).rstrip() + "\n", encoding="utf-8")

    if args.json_out:
        json_path = Path(args.json_out).resolve()
        payload = {
            "generated_at": now,
            "root": str(root),
            "versions": versions,
            "pyproject_dependencies": pyproject_deps,
            "matches": matches,
        }
        json_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"Wrote: {out}")
    if args.json_out:
        print(f"Wrote: {args.json_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

