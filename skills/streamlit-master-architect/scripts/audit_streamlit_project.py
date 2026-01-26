from __future__ import annotations

import argparse
import ast
import json
import os
import re
import sys
import urllib.error
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable, Literal

try:
    import tomllib  # py311+
except ModuleNotFoundError:  # pragma: no cover
    tomllib = None  # type: ignore[assignment]

JsonDict = dict[str, Any]
Severity = Literal["low", "medium", "high"]


DEFAULT_EXCLUDES = {
    ".git",
    ".hg",
    ".svn",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    "__pycache__",
    "build",
    "dist",
    "node_modules",
    "opensrc",
    "venv",
    ".venv",
    "env",
    ".env",
    "site-packages",
}


WIDGET_FUNCS = {
    # Common input widgets (subset; used for "missing key in loop" heuristic).
    "button",
    "checkbox",
    "color_picker",
    "date_input",
    "datetime_input",
    "file_uploader",
    "camera_input",
    "multiselect",
    "number_input",
    "radio",
    "selectbox",
    "slider",
    "text_input",
    "text_area",
    "time_input",
    "toggle",
}


DEPRECATED_OR_RISKY = {
    "st.cache": ("high", "Deprecated caching API; migrate to st.cache_data / st.cache_resource."),
    "st.experimental_memo": ("high", "Deprecated; migrate to st.cache_data."),
    "st.experimental_singleton": ("high", "Deprecated; migrate to st.cache_resource."),
    "st.experimental_rerun": ("medium", "Prefer st.rerun (stable)."),
    "st.experimental_set_query_params": ("medium", "Prefer st.query_params."),
    "st.experimental_get_query_params": ("medium", "Prefer st.query_params."),
    "st.bokeh_chart": ("high", "Native Bokeh support removed in modern Streamlit; replace with Altair/Plotly/etc."),
}


RISKY_FLAGS = {
    "unsafe_allow_html=True": (
        "high",
        "Potential XSS sink (e.g., st.markdown). Avoid unless content is trusted and sanitized.",
    ),
    "unsafe_allow_javascript=True": (
        "high",
        "High-risk JS execution (st.html). Never use with untrusted input.",
    ),
}


@dataclass(frozen=True)
class Finding:
    severity: Severity
    code: str
    message: str
    locations: list[str]


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _pypi_latest_streamlit_version(timeout_s: float = 10.0) -> str | None:
    url = "https://pypi.org/pypi/streamlit/json"
    req = urllib.request.Request(url, headers={"User-Agent": "streamlit-master-architect/0.1"})
    try:
        with urllib.request.urlopen(req, timeout=timeout_s) as resp:
            payload = json.loads(resp.read().decode("utf-8", errors="replace"))
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError):
        return None
    info = payload.get("info", {})
    if isinstance(info, dict):
        v = info.get("version")
        return str(v) if v else None
    return None


def _installed_streamlit_version() -> str | None:
    try:
        from importlib.metadata import PackageNotFoundError, version
    except Exception:  # pragma: no cover
        return None
    try:
        return version("streamlit")
    except PackageNotFoundError:
        return None
    except Exception:
        return None


def _locked_streamlit_version(root: Path) -> str | None:
    if tomllib is None:
        return None

    uv_lock = root / "uv.lock"
    if uv_lock.exists():
        try:
            data = tomllib.loads(_read_text(uv_lock))
        except Exception:
            data = None
        if isinstance(data, dict):
            pkgs = data.get("package", [])
            if isinstance(pkgs, list):
                for p in pkgs:
                    if isinstance(p, dict) and p.get("name") == "streamlit":
                        v = p.get("version")
                        return str(v) if v else None

    poetry_lock = root / "poetry.lock"
    if poetry_lock.exists():
        try:
            data = tomllib.loads(_read_text(poetry_lock))
        except Exception:
            data = None
        if isinstance(data, dict):
            pkgs = data.get("package", [])
            if isinstance(pkgs, list):
                for p in pkgs:
                    if isinstance(p, dict) and p.get("name") == "streamlit":
                        v = p.get("version")
                        return str(v) if v else None

    return None


def _iter_python_files(root: Path, *, excludes: set[str]) -> Iterable[Path]:
    for path in root.rglob("*.py"):
        rel_parts = path.relative_to(root).parts
        if any(p in excludes for p in rel_parts):
            continue
        yield path


def _parse_requirements(path: Path) -> list[str]:
    specs: list[str] = []
    for raw in _read_text(path).splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        # Remove inline comments.
        line = line.split("#", 1)[0].strip()
        # Ignore pip options and URL installs.
        if line.startswith("-"):
            continue
        if "://" in line:
            continue
        if re.match(r"(?i)^streamlit(\[.*\])?([<>=!~]=?.*)?$", line):
            specs.append(line)
    return specs


def _scan_pyproject(path: Path) -> list[str]:
    if tomllib is None:
        return []
    try:
        data = tomllib.loads(_read_text(path))
    except Exception:
        return []

    found: list[str] = []

    project = data.get("project", {})
    if isinstance(project, dict):
        deps = project.get("dependencies", [])
        if isinstance(deps, list):
            for d in deps:
                if isinstance(d, str) and d.lower().startswith("streamlit"):
                    found.append(d)
        opt = project.get("optional-dependencies", {})
        if isinstance(opt, dict):
            for _group, group_deps in opt.items():
                if isinstance(group_deps, list):
                    for d in group_deps:
                        if isinstance(d, str) and d.lower().startswith("streamlit"):
                            found.append(d)

    tool = data.get("tool", {})
    if isinstance(tool, dict):
        poetry = tool.get("poetry", {})
        if isinstance(poetry, dict):
            deps = poetry.get("dependencies", {})
            if isinstance(deps, dict):
                v = deps.get("streamlit")
                if isinstance(v, str):
                    found.append(f"streamlit {v}")
                elif isinstance(v, dict):
                    found.append(f"streamlit {json.dumps(v, sort_keys=True)}")

            groups = poetry.get("group", {})
            if isinstance(groups, dict):
                for _gname, gdata in groups.items():
                    if not isinstance(gdata, dict):
                        continue
                    gdeps = gdata.get("dependencies", {})
                    if isinstance(gdeps, dict):
                        v = gdeps.get("streamlit")
                        if isinstance(v, str):
                            found.append(f"streamlit {v}")
                        elif isinstance(v, dict):
                            found.append(f"streamlit {json.dumps(v, sort_keys=True)}")

    # Deduplicate while keeping stable order.
    out: list[str] = []
    for x in found:
        if x not in out:
            out.append(x)
    return out


def _collect_dependency_specs(root: Path) -> list[JsonDict]:
    specs: list[JsonDict] = []

    req = root / "requirements.txt"
    if req.exists():
        for s in _parse_requirements(req):
            specs.append({"file": str(req), "spec": s})

    pyproject = root / "pyproject.toml"
    if pyproject.exists():
        for s in _scan_pyproject(pyproject):
            specs.append({"file": str(pyproject), "spec": s})

    return specs


def _get_attr_chain(expr: ast.AST) -> list[str] | None:
    parts: list[str] = []
    cur: ast.AST | None = expr
    while isinstance(cur, ast.Attribute):
        parts.append(cur.attr)
        cur = cur.value
    if isinstance(cur, ast.Name):
        parts.append(cur.id)
        return list(reversed(parts))
    return None


def _scan_streamlit_usage(py_file: Path) -> tuple[dict[str, int], list[Finding]]:
    text = _read_text(py_file)
    try:
        tree = ast.parse(text, filename=str(py_file))
    except SyntaxError:
        return {}, [Finding(severity="low", code="parse_error", message="Failed to parse Python file.", locations=[str(py_file)])]

    module_aliases: set[str] = set()
    imported_names: dict[str, str] = {}

    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            for alias in node.names:
                if alias.name == "streamlit":
                    module_aliases.add(alias.asname or "streamlit")
        elif isinstance(node, ast.ImportFrom) and node.module == "streamlit":
            for alias in node.names:
                imported_names[alias.asname or alias.name] = alias.name

    usage: dict[str, int] = {}
    findings: list[Finding] = []

    # Simple text-level risky flag detection (fast, includes non-Streamlit sinks).
    for needle, (sev, msg) in RISKY_FLAGS.items():
        if needle in text:
            findings.append(
                Finding(
                    severity=sev,
                    code="security_flag",
                    message=f"{msg} (found `{needle}`)",
                    locations=[str(py_file)],
                )
            )

    # Deprecated API usage detection via AST calls.
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue

        # Case A: st.foo(...)
        chain = _get_attr_chain(node.func)
        call_name: str | None = None
        if chain and chain[0] in module_aliases:
            call_name = ".".join(chain)
        elif isinstance(node.func, ast.Name) and node.func.id in imported_names:
            call_name = f"streamlit.{imported_names[node.func.id]}"

        if not call_name:
            continue

        usage[call_name] = usage.get(call_name, 0) + 1

        # Map aliases like "st.experimental_memo" regardless of alias name.
        canonical = call_name
        for alias in module_aliases:
            if canonical.startswith(f"{alias}."):
                canonical = "st." + canonical[len(alias) + 1 :]
                break

        if canonical in DEPRECATED_OR_RISKY:
            sev, msg = DEPRECATED_OR_RISKY[canonical]
            loc = f"{py_file}:{getattr(node, 'lineno', 1)}"
            findings.append(Finding(severity=sev, code="deprecated_api", message=f"{canonical}: {msg}", locations=[loc]))

        # Heuristic: widget call in loop without key.
        # Only applies to direct st.<widget>(...) or st.sidebar.<widget>(...) forms.
        if chain and chain[0] in module_aliases:
            # last segment could be widget name
            widget_name = chain[-1]
            if widget_name in WIDGET_FUNCS:
                has_key = any(isinstance(k, ast.keyword) and k.arg == "key" for k in node.keywords)
                if not has_key:
                    # Walk parents via a second pass: easiest is to flag only if text contains "for" on same or previous line.
                    # This is conservative and avoids building a full parent map.
                    line_no = getattr(node, "lineno", 0)
                    if line_no:
                        lines = text.splitlines()
                        window = "\n".join(lines[max(0, line_no - 3) : line_no])
                        if re.search(r"\bfor\b", window):
                            loc = f"{py_file}:{line_no}"
                            findings.append(
                                Finding(
                                    severity="medium",
                                    code="missing_key_in_loop",
                                    message=f"Possible widget call inside a loop without `key=`: {call_name}",
                                    locations=[loc],
                                )
                            )

    return usage, findings


def _aggregate_findings(findings: Iterable[Finding]) -> list[JsonDict]:
    # Merge identical (severity, code, message) by accumulating locations.
    merged: dict[tuple[str, str, str], set[str]] = {}
    for f in findings:
        key = (f.severity, f.code, f.message)
        merged.setdefault(key, set()).update(f.locations)
    out: list[JsonDict] = []
    for (sev, code, msg), locs in sorted(merged.items(), key=lambda x: (x[0][0], x[0][1], x[0][2])):
        out.append({"severity": sev, "code": code, "message": msg, "locations": sorted(locs)})
    return out


def _to_markdown(report: JsonDict) -> str:
    lines: list[str] = []
    lines.append(f"# Streamlit project audit\n")
    lines.append(f"- Root: `{report['project_root']}`")

    st_info = report.get("streamlit", {})
    if isinstance(st_info, dict):
        lines.append(f"- Installed Streamlit: `{st_info.get('installed_version')}`")
        lines.append(f"- Locked Streamlit (lockfile): `{st_info.get('locked_version')}`")
        lines.append(f"- Latest Streamlit (PyPI): `{st_info.get('latest_version')}`")

    specs = report.get("dependency_specs", [])
    lines.append("\n## Dependency specs\n")
    if not specs:
        lines.append("- (none found)")
    else:
        for s in specs:
            lines.append(f"- `{s.get('file')}`: `{s.get('spec')}`")

    usage = report.get("top_calls", [])
    lines.append("\n## Streamlit usage (top calls)\n")
    if not usage:
        lines.append("- (no Streamlit calls detected)")
    else:
        for item in usage:
            lines.append(f"- `{item['call']}`: {item['count']}")

    issues = report.get("issues", [])
    lines.append("\n## Findings\n")
    if not issues:
        lines.append("- (no issues detected)")
    else:
        for i in issues:
            sev = i.get("severity", "medium")
            msg = i.get("message", "")
            lines.append(f"- **{sev}**: {msg}")
            locs = i.get("locations", [])
            if isinstance(locs, list) and locs:
                for loc in locs[:20]:
                    lines.append(f"  - `{loc}`")
                if len(locs) > 20:
                    lines.append(f"  - … (+{len(locs) - 20} more)")

    recs = report.get("recommendations", [])
    lines.append("\n## Recommendations\n")
    if not recs:
        lines.append("- (none)")
    else:
        for r in recs:
            lines.append(f"- {r}")

    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="Audit a Streamlit project for version, deps, and risky/deprecated APIs.")
    parser.add_argument("--root", type=str, default=".", help="Project root to scan.")
    parser.add_argument("--format", type=str, default="json", choices=["json", "md"], help="Output format.")
    parser.add_argument("--output", type=str, default="", help="Write report to a file instead of stdout.")
    parser.add_argument(
        "--check-latest",
        action="store_true",
        help="(deprecated) Enable latest Streamlit version check (default on).",
    )
    parser.add_argument("--no-check-latest", action="store_true", help="Disable PyPI latest-version check.")
    parser.add_argument("--top", type=int, default=30, help="Top N Streamlit calls to include.")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    if not root.exists():
        raise FileNotFoundError(f"Root not found: {root}")

    check_latest = not args.no_check_latest

    installed = _installed_streamlit_version()
    locked = _locked_streamlit_version(root)
    latest = _pypi_latest_streamlit_version() if check_latest else None
    dep_specs = _collect_dependency_specs(root)

    all_usage: dict[str, int] = {}
    findings: list[Finding] = []

    for py in _iter_python_files(root, excludes=DEFAULT_EXCLUDES):
        usage, file_findings = _scan_streamlit_usage(py)
        for k, v in usage.items():
            all_usage[k] = all_usage.get(k, 0) + v
        findings.extend(file_findings)

    # Detect beta APIs by name prefix (works regardless of version).
    beta_calls = [k for k in all_usage.keys() if ".beta_" in k]
    if beta_calls:
        findings.append(
            Finding(
                severity="high",
                code="deprecated_api",
                message="Detected st.beta_* APIs; these are legacy and should be migrated to stable equivalents.",
                locations=beta_calls[:50],
            )
        )

    top_calls = sorted(all_usage.items(), key=lambda kv: kv[1], reverse=True)[: max(1, args.top)]

    recommendations: list[str] = []
    current_for_compare = installed or locked
    if latest and current_for_compare and latest != current_for_compare:
        recommendations.append(
            f"Consider upgrading Streamlit from {current_for_compare} to {latest} after reading release notes and running tests."
        )
    if not dep_specs:
        recommendations.append("No Streamlit dependency spec found (requirements.txt/pyproject.toml). Ensure Streamlit is pinned or constrained for reproducible deploys.")
    if any(f.code == "security_flag" for f in findings):
        recommendations.append("Review all unsafe HTML/JS flags; ensure inputs are trusted/sanitized and usage is isolated.")
    if any(f.code == "deprecated_api" for f in findings):
        recommendations.append("Migrate deprecated Streamlit APIs to their stable equivalents (see Findings).")

    report: JsonDict = {
        "project_root": str(root),
        "streamlit": {"installed_version": installed, "locked_version": locked, "latest_version": latest},
        "dependency_specs": dep_specs,
        "top_calls": [{"call": k, "count": v} for k, v in top_calls],
        "issues": _aggregate_findings(findings),
        "recommendations": recommendations,
    }

    out: str
    if args.format == "md":
        out = _to_markdown(report)
    else:
        out = json.dumps(report, indent=2, sort_keys=True)

    if args.output:
        Path(args.output).write_text(out + ("\n" if not out.endswith("\n") else ""), encoding="utf-8")
    else:
        sys.stdout.write(out + ("\n" if not out.endswith("\n") else ""))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
