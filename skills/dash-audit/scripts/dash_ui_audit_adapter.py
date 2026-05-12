#!/usr/bin/env python3
"""Convert Dash UI preflight callback maps into the ui_audit.v1 contract."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path, PureWindowsPath
from typing import Any


SCHEMA = "ui_audit.v1"
PRODUCER_VERSION = "2026-05-12"
JsonDict = dict[str, Any]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse command-line arguments.

    Args:
        argv: Optional argument vector for tests. When omitted, argparse reads
            from process arguments.

    Returns:
        Parsed command-line options.
    """
    parser = argparse.ArgumentParser(
        description=(
            "Adapt ui-audit-preflight dash-callback-map JSON into ui_audit.v1."
        )
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Path to dash-callback-map JSON produced by ui-audit-preflight.",
    )
    parser.add_argument(
        "--output",
        default="",
        help="Write adapted JSON to this file instead of stdout.",
    )
    parser.add_argument(
        "--pretty",
        action="store_true",
        help="Pretty-print JSON output.",
    )
    return parser.parse_args(argv)


def read_json(path: Path) -> JsonDict:
    """Read a JSON object from disk.

    Args:
        path: JSON file path.

    Returns:
        Decoded JSON object.

    Raises:
        ValueError: If the file does not contain a JSON object.
        OSError: If the file cannot be read.
        json.JSONDecodeError: If the file is not valid JSON.
    """
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data, dict):
        raise ValueError(f"Expected JSON object in {path}")
    return data


def is_windows_absolute(path: str) -> bool:
    """Return whether a string looks like an absolute Windows path.

    Args:
        path: Path string from the preflight payload.

    Returns:
        True when the path has a Windows drive or UNC root.
    """
    windows = PureWindowsPath(path)
    return bool(windows.drive and windows.root)


def redact_path(path: str, repo_root: str) -> str:
    """Return a root-redacted path suitable for shared audit evidence.

    Args:
        path: Path string from the preflight payload.
        repo_root: Repository root reported by the preflight payload.

    Returns:
        Repo-relative path when possible, otherwise a basename for absolute
        paths outside the root.
    """
    if is_windows_absolute(path):
        raw_windows = PureWindowsPath(path)
        if repo_root and is_windows_absolute(repo_root):
            try:
                return raw_windows.relative_to(
                    PureWindowsPath(repo_root)
                ).as_posix()
            except ValueError:
                pass
        return raw_windows.name or "<unknown>"

    raw = Path(path)
    if not raw.is_absolute():
        return path

    if repo_root:
        try:
            return str(raw.relative_to(Path(repo_root)))
        except ValueError:
            pass
    return raw.name or "<unknown>"


def ui_location(path: str, repo_root: str) -> JsonDict:
    """Build a ui_audit.v1 location object from a Dash preflight path.

    Args:
        path: Repo-relative or absolute path from the preflight payload.
        repo_root: Repository root reported by the preflight payload.

    Returns:
        Location object with a path field.
    """
    return {"path": redact_path(path, repo_root)}


def count_value(item: JsonDict, key: str) -> int:
    """Read a callback count field with a safe fallback.

    Args:
        item: Callback aggregate from ui-audit-preflight.
        key: Count field name.

    Returns:
        Integer count, or zero when the field is missing or malformed.
    """
    try:
        return int(item.get(key) or 0)
    except (TypeError, ValueError):
        return 0


def observation_for_callback_file(item: JsonDict, repo_root: str) -> JsonDict:
    """Render one Dash callback-map row as a non-actionable observation.

    Args:
        item: Callback aggregate from ui-audit-preflight.
        repo_root: Repository root reported by the preflight payload.

    Returns:
        ui_audit.v1 observation object.
    """
    file_path = str(item.get("file") or "<unknown>")
    callback_count = count_value(item, "callback_decorators")
    output_count = count_value(item, "output_calls")
    input_count = count_value(item, "input_calls")
    state_count = count_value(item, "state_calls")
    return {
        "id": "dash.callback_map",
        "category": "state",
        "title": "Dash callback map entry",
        "detail": (
            f"{callback_count} callback decorator(s), {output_count} "
            f"Output call(s), {input_count} Input call(s), and {state_count} "
            f"State call(s)."
        ),
        "locations": [ui_location(file_path, repo_root)],
        "data": {
            "callback_decorators": callback_count,
            "output_calls": output_count,
            "input_calls": input_count,
            "state_calls": state_count,
        },
    }


def findings_for_callback_file(
    item: JsonDict, repo_root: str
) -> list[JsonDict]:
    """Create actionable findings for suspicious callback-map rows.

    Args:
        item: Callback aggregate from ui-audit-preflight.
        repo_root: Repository root reported by the preflight payload.

    Returns:
        Finding objects for rows that need follow-up.
    """
    file_path = str(item.get("file") or "<unknown>")
    callback_count = count_value(item, "callback_decorators")
    output_count = count_value(item, "output_calls")
    if callback_count <= 0 or output_count > 0:
        return []
    return [
        {
            "id": "dash.callback_without_output",
            "severity": "warning",
            "category": "state",
            "title": "Callback decorator without detected Output",
            "detail": (
                "The Dash preflight found callback decorators but no Output "
                "calls in this file. Verify callback registration and imports."
            ),
            "locations": [ui_location(file_path, repo_root)],
            "recommendation": (
                "Inspect the callback decorators and confirm every callback "
                "declares at least one Output before runtime."
            ),
            "docs": [
                "https://dash.plotly.com/basic-callbacks",
            ],
        }
    ]


def summarize(findings: list[JsonDict]) -> JsonDict:
    """Summarize findings into ui_audit.v1 status and severity counts.

    Args:
        findings: ui_audit.v1 finding objects.

    Returns:
        Summary object with status, counts, and total_findings.
    """
    counts = {"error": 0, "warning": 0, "info": 0}
    for finding in findings:
        severity = str(finding.get("severity") or "info")
        if severity not in counts:
            severity = "info"
        counts[severity] += 1
    if counts["error"]:
        status = "fail"
    elif counts["warning"]:
        status = "warning"
    else:
        status = "pass"
    return {
        "status": status,
        "counts": counts,
        "total_findings": sum(counts.values()),
    }


def invalid_preflight_finding(detail: str) -> JsonDict:
    """Build a warning for malformed Dash preflight evidence.

    Args:
        detail: Specific invalid payload condition.

    Returns:
        ui_audit.v1 warning finding.
    """
    return {
        "id": "dash.invalid_preflight_payload",
        "severity": "warning",
        "category": "testing",
        "title": "Invalid Dash preflight callback payload",
        "detail": detail,
        "locations": [],
        "recommendation": (
            "Rerun ui-audit-preflight dash-callback-map and verify the "
            "generated JSON before relying on this audit."
        ),
        "docs": [],
    }


def adapt_dash_preflight(payload: JsonDict) -> JsonDict:
    """Adapt a Dash callback-map payload into ui_audit.v1.

    Args:
        payload: JSON object emitted by `ui-audit-preflight dash-callback-map`.

    Returns:
        ui_audit.v1 payload.
    """
    repo_root = str(payload.get("repo_root") or "")
    callbacks = payload.get("callbacks", [])
    invalid_shape = not isinstance(callbacks, list)
    if not isinstance(callbacks, list):
        callbacks = []

    observations: list[JsonDict] = []
    findings: list[JsonDict] = []
    invalid_rows = 0
    for raw in callbacks:
        if not isinstance(raw, dict):
            invalid_rows += 1
            continue
        observations.append(observation_for_callback_file(raw, repo_root))
        findings.extend(findings_for_callback_file(raw, repo_root))

    if invalid_shape:
        findings.append(
            invalid_preflight_finding(
                "The Dash preflight payload did not contain a callbacks array, "
                "so callback coverage could not be trusted."
            )
        )
    if invalid_rows:
        plural = "row" if invalid_rows == 1 else "rows"
        findings.append(
            invalid_preflight_finding(
                f"The Dash preflight payload contained {invalid_rows} "
                f"non-object callback {plural}, so callback coverage could "
                "not be fully trusted."
            )
        )

    return {
        "schema": SCHEMA,
        "producer": {
            "skill": "dash-audit",
            "tool": "dash_ui_audit_adapter.py",
            "version": PRODUCER_VERSION,
            "source": "ui-audit-preflight dash-callback-map",
        },
        "target": {
            "framework": "dash",
            "root": "<scan-root>",
        },
        "summary": summarize(findings),
        "findings": findings,
        "observations": observations,
        "metadata": {
            "privacy": {
                "root_redacted": True,
                "source_snippets_included": False,
            },
            "source_repo_root_present": bool(payload.get("repo_root")),
        },
    }


def main(argv: list[str] | None = None) -> int:
    """Run the adapter CLI.

    Args:
        argv: Optional argument vector for tests.

    Returns:
        Process exit code.
    """
    args = parse_args(argv)
    payload = adapt_dash_preflight(read_json(Path(args.input)))
    indent = 2 if args.pretty else None
    out = json.dumps(payload, indent=indent, sort_keys=True)
    if args.output:
        Path(args.output).write_text(out + "\n", encoding="utf-8")
    else:
        sys.stdout.write(out + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
