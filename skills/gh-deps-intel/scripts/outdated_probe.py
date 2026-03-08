#!/usr/bin/env python3
"""Outdated dependency probing for JS and Python ecosystems."""

from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

from utils import run_cmd, which


class ProbeResult(dict):
    pass


def _to_json(text: str) -> Any:
    try:
        return json.loads(text)
    except Exception:
        return None


def _parse_bun_outdated_table(stdout: str) -> dict[str, dict[str, str]]:
    results: dict[str, dict[str, str]] = {}
    lines = [ln.rstrip() for ln in stdout.splitlines() if ln.strip()]
    header_seen = False
    for line in lines:
        lower = line.lower()
        if "package" in lower and "current" in lower and "latest" in lower:
            header_seen = True
            continue
        if not header_seen:
            continue
        # Typical shape: name  current  target  latest
        parts = re.split(r"\s{2,}|\t+", line.strip())
        if len(parts) < 4:
            continue
        name, current, wanted, latest = parts[0], parts[1], parts[2], parts[3]
        if not name or name.startswith("-"):
            continue
        results[name] = {
            "current": current,
            "wanted": wanted,
            "latest": latest,
            "source": "bun outdated",
        }
    return results


def _parse_npm_outdated_json(stdout: str) -> dict[str, dict[str, str]]:
    data = _to_json(stdout)
    if not isinstance(data, dict):
        return {}
    out: dict[str, dict[str, str]] = {}
    for name, details in data.items():
        if not isinstance(details, dict):
            continue
        out[name] = {
            "current": str(details.get("current") or ""),
            "wanted": str(details.get("wanted") or ""),
            "latest": str(details.get("latest") or ""),
            "source": "npm outdated",
        }
    return out


def _parse_pnpm_outdated_json(stdout: str) -> dict[str, dict[str, str]]:
    data = _to_json(stdout)
    out: dict[str, dict[str, str]] = {}

    if isinstance(data, list):
        iterable = data
    elif isinstance(data, dict):
        # pnpm may return object keyed by workspace path.
        iterable = []
        for v in data.values():
            if isinstance(v, list):
                iterable.extend(v)
    else:
        iterable = []

    for item in iterable:
        if not isinstance(item, dict):
            continue
        name = item.get("packageName") or item.get("name")
        if not isinstance(name, str):
            continue
        out[name] = {
            "current": str(item.get("current") or ""),
            "wanted": str(item.get("wanted") or ""),
            "latest": str(item.get("latest") or ""),
            "source": "pnpm outdated",
        }
    return out


def _parse_yarn_outdated(stdout: str) -> dict[str, dict[str, str]]:
    out: dict[str, dict[str, str]] = {}
    # Yarn classic can emit table rows: package current wanted latest type url
    for line in stdout.splitlines():
        s = line.strip()
        if not s or s.startswith("{"):
            continue
        if s.lower().startswith("package "):
            continue
        if s.startswith("Done") or s.startswith("✨"):
            continue
        parts = re.split(r"\s+", s)
        if len(parts) < 4:
            continue
        name, current, wanted, latest = parts[0], parts[1], parts[2], parts[3]
        if name in {"info", "warning", "error"}:
            continue
        out[name] = {
            "current": current,
            "wanted": wanted,
            "latest": latest,
            "source": "yarn outdated",
        }
    return out


def _parse_python_outdated_json(stdout: str, source: str) -> dict[str, dict[str, str]]:
    data = _to_json(stdout)
    out: dict[str, dict[str, str]] = {}
    if not isinstance(data, list):
        return out
    for item in data:
        if not isinstance(item, dict):
            continue
        name = item.get("name")
        if not isinstance(name, str):
            continue
        latest = item.get("latest_version") or item.get("latest") or ""
        out[name.lower().replace("_", "-")] = {
            "current": str(item.get("version") or item.get("current") or ""),
            "wanted": str(latest),
            "latest": str(latest),
            "source": source,
        }
    return out


def probe_js_outdated(repo_context: dict[str, Any], repo_root: Path) -> tuple[dict[str, dict[str, str]], list[dict[str, Any]], list[str]]:
    manager = repo_context.get("node_manager") or "npm"
    commands: list[list[str]] = []
    warnings: list[str] = []

    if manager == "bun":
        cmd = ["bun", "outdated"]
        if repo_context.get("is_monorepo"):
            cmd += ["--recursive", "--filter=*", "--no-progress"]
        commands.append(cmd)
    elif manager == "pnpm":
        commands.append(["pnpm", "outdated", "-r", "--format", "json"])
    elif manager == "yarn":
        commands.append(["yarn", "outdated"])
    else:
        commands.append(["npm", "outdated", "--json", "--all"])

    parsed: dict[str, dict[str, str]] = {}
    traces: list[dict[str, Any]] = []

    for cmd in commands:
        proc = run_cmd(cmd, cwd=repo_root, check=False)
        traces.append(
            {
                "command": " ".join(cmd),
                "returncode": proc.returncode,
                "stdout": proc.stdout[-8000:],
                "stderr": proc.stderr[-4000:],
            }
        )

        if manager == "bun":
            parsed = _parse_bun_outdated_table(proc.stdout)
        elif manager == "pnpm":
            parsed = _parse_pnpm_outdated_json(proc.stdout)
        elif manager == "yarn":
            parsed = _parse_yarn_outdated(proc.stdout)
        else:
            parsed = _parse_npm_outdated_json(proc.stdout)

        if parsed:
            break
        warnings.append(f"Unable to parse `{ ' '.join(cmd) }` output; falling back to registry metadata for missing versions.")

    return parsed, traces, warnings


def probe_python_outdated(repo_context: dict[str, Any], repo_root: Path) -> tuple[dict[str, dict[str, str]], list[dict[str, Any]], list[str]]:
    manager = repo_context.get("python_manager") or "uv"
    traces: list[dict[str, Any]] = []
    warnings: list[str] = []

    cmds: list[tuple[list[str], str]] = []
    if manager == "uv" and which("uv"):
        cmds.append(([
            "uv",
            "pip",
            "list",
            "--outdated",
            "--format",
            "json",
            "--project",
            str(repo_root),
        ], "uv pip list --outdated"))

    cmds.append((["python3", "-m", "pip", "list", "--outdated", "--format", "json"], "pip list --outdated"))

    for cmd, source in cmds:
        proc = run_cmd(cmd, cwd=repo_root, check=False)
        traces.append(
            {
                "command": " ".join(cmd),
                "returncode": proc.returncode,
                "stdout": proc.stdout[-8000:],
                "stderr": proc.stderr[-4000:],
            }
        )
        parsed = _parse_python_outdated_json(proc.stdout, source)
        if parsed:
            return parsed, traces, warnings

    warnings.append("Unable to gather Python outdated list from uv/pip; using index metadata fallback.")
    return {}, traces, warnings


def probe_outdated(repo_context: dict[str, Any]) -> ProbeResult:
    repo_root = Path(repo_context["repo_root"])
    data: ProbeResult = ProbeResult(
        js={},
        python={},
        command_traces=[],
        warnings=[],
    )

    if repo_context.get("has_node"):
        js, traces, warnings = probe_js_outdated(repo_context, repo_root)
        data["js"] = js
        data["command_traces"].extend(traces)
        data["warnings"].extend(warnings)

    if repo_context.get("has_python"):
        py, traces, warnings = probe_python_outdated(repo_context, repo_root)
        data["python"] = py
        data["command_traces"].extend(traces)
        data["warnings"].extend(warnings)

    return data


def main() -> None:
    import argparse
    from detect_repo import detect_repo_context

    parser = argparse.ArgumentParser(description="Probe outdated dependencies.")
    parser.add_argument("repo", nargs="?", default=".")
    args = parser.parse_args()

    ctx = detect_repo_context(Path(args.repo))
    print(json.dumps(probe_outdated(ctx), indent=2))


if __name__ == "__main__":
    main()
