#!/usr/bin/env python3
"""Collect redacted Sentry issue context using the sentry CLI."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any
from urllib.parse import quote

JsonValue = (
    None | bool | int | float | str | list["JsonValue"] | dict[str, "JsonValue"]
)
JsonObject = dict[str, JsonValue]

ISSUE_VIEW_FIELDS = (
    "id,shortId,title,culprit,count,userCount,firstSeen,lastSeen,level,status,"
    "substatus,priority,platform,permalink,project,metadata,assignedTo,"
    "isUnhandled,event,trace,replayIds"
)
EVENT_FIELDS = (
    "id,eventID,groupID,projectID,message,title,location,culprit,user,tags,"
    "platform,dateCreated,crashFile,metadata"
)


SENSITIVE_KEY_RE = re.compile(
    r"(authorization|cookie|token|secret|password|passwd|api[_-]?key|dsn|"
    r"session|jwt|bearer|private|credential|auth)",
    re.IGNORECASE,
)
EMAIL_RE = re.compile(r"(?<![\w.+-])[\w.+-]+@[\w.-]+\.[A-Za-z]{2,}(?![\w.-])")
BEARER_RE = re.compile(r"\bBearer\s+[A-Za-z0-9._~+/=-]+", re.IGNORECASE)
LONG_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_-]{32,}\b")
SENTRY_DSN_RE = re.compile(r"https?://[^@\s]+@[^/\s]+/\d+")


def redact_scalar(value: str, max_string: int) -> str:
    """Redact sensitive patterns from a scalar string.

    Args:
        value: String to redact.
        max_string: Maximum retained length before truncation.

    Returns:
        Redacted and possibly truncated string.
    """
    value = EMAIL_RE.sub("[REDACTED_EMAIL]", value)
    value = BEARER_RE.sub("Bearer [REDACTED]", value)
    value = SENTRY_DSN_RE.sub("[REDACTED_DSN]", value)
    value = LONG_TOKEN_RE.sub("[REDACTED_TOKEN]", value)
    if len(value) > max_string:
        value = value[:max_string] + "...[TRUNCATED]"
    return value


def redact(
    value: JsonValue, max_string: int = 500, key: str | None = None
) -> JsonValue:
    """Redact sensitive keys and nested scalar values.

    Args:
        value: JSON-compatible value to redact.
        max_string: Maximum retained string length.
        key: Optional parent key used for key-based redaction.

    Returns:
        Redacted JSON-compatible value.
    """
    if key and SENSITIVE_KEY_RE.search(key):
        return "[REDACTED]"
    if isinstance(value, dict):
        return {k: redact(v, max_string, str(k)) for k, v in value.items()}
    if isinstance(value, list):
        return [redact(v, max_string) for v in value]
    if isinstance(value, str):
        return redact_scalar(value, max_string)
    return value


def run_sentry(args: list[str], timeout: int) -> JsonObject:
    """Run a Sentry CLI command and capture raw output.

    Args:
        args: Arguments passed after the `sentry` executable.
        timeout: Maximum command runtime in seconds.

    Returns:
        Command record with args, return code, and captured output or error.
    """
    command = ["sentry", *args]
    try:
        # No shell is used; argv is constrained to the sentry executable.
        proc = subprocess.run(  # noqa: S603
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except FileNotFoundError:
        return {
            "args": command,
            "returncode": 127,
            "error": "sentry CLI not found on PATH",
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "args": command,
            "returncode": 124,
            "error": f"timed out after {timeout}s",
            "stdout": exc.stdout or "",
            "stderr": exc.stderr or "",
        }

    result: dict[str, Any] = {
        "args": command,
        "returncode": proc.returncode,
    }
    if proc.stdout.strip():
        result["stdout"] = proc.stdout
    if proc.stderr.strip():
        result["stderr"] = proc.stderr
    return result


def positive_int(value: str) -> int:
    """Parse a positive integer for argparse.

    Args:
        value: Raw command-line value.

    Returns:
        Parsed integer greater than zero.

    Raises:
        argparse.ArgumentTypeError: When the value is not a positive integer.
    """
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("must be a positive integer") from exc
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be a positive integer")
    return parsed


def parse_json_result(result: JsonObject) -> JsonValue:
    """Parse a Sentry command JSON result.

    Args:
        result: Command record returned by `run_sentry`.

    Returns:
        Parsed JSON value, `None` when stdout is empty, or a parse error record.
    """
    stdout = result.get("stdout", "")
    if not isinstance(stdout, str) or not stdout:
        return None
    try:
        return json.loads(stdout)
    except json.JSONDecodeError as exc:
        return {"parse_error": str(exc), "raw": stdout}


def add_json_command(
    bundle: dict[str, Any],
    label: str,
    args: list[str],
    timeout: int,
    max_string: int,
) -> JsonValue:
    """Run a JSON Sentry command and add it to the bundle.

    Args:
        bundle: Mutable output bundle.
        label: Section label for the parsed command output.
        args: Sentry CLI arguments.
        timeout: Maximum command runtime in seconds.
        max_string: Maximum retained string length.

    Returns:
        Parsed command JSON, or a parse/empty-output sentinel.
    """
    result = run_sentry(args, timeout)
    parsed = parse_json_result(result)
    bundle["commands"].append(
        redact(
            {
                "label": label,
                "args": result.get("args", []),
                "returncode": result.get("returncode"),
                "stderr": result.get("stderr"),
            },
            max_string,
        )
    )
    bundle["sections"][label] = redact(parsed, max_string)
    return parsed


def nested_values(value: JsonValue, wanted_keys: set[str]) -> list[str]:
    """Collect string values for matching nested keys.

    Args:
        value: JSON-compatible value to inspect.
        wanted_keys: Normalized key names to extract.

    Returns:
        Matching string values in traversal order.
    """
    found: list[str] = []
    if isinstance(value, dict):
        for key, child in value.items():
            normalized = key.replace("-", "_").lower()
            if normalized in wanted_keys and isinstance(child, str):
                found.append(child)
            else:
                found.extend(nested_values(child, wanted_keys))
    elif isinstance(value, list):
        for child in value:
            found.extend(nested_values(child, wanted_keys))
    return found


def tag_values(value: JsonValue, tag_name: str) -> list[str]:
    """Collect Sentry tag values by tag key.

    Args:
        value: JSON-compatible value to inspect.
        tag_name: Sentry tag key to match case-insensitively.

    Returns:
        Matching tag values in traversal order.
    """
    normalized_tag = tag_name.lower()
    values: list[str] = []
    if isinstance(value, dict):
        tags = value.get("tags")
        if isinstance(tags, list):
            for tag in tags:
                if not isinstance(tag, dict):
                    continue
                key = str(tag.get("key") or tag.get("name") or "").lower()
                if key == normalized_tag and tag.get("value") is not None:
                    values.append(str(tag["value"]))
        for child in value.values():
            values.extend(tag_values(child, tag_name))
    elif isinstance(value, list):
        for child in value:
            values.extend(tag_values(child, tag_name))
    return values


def unique(values: list[str], limit: int) -> list[str]:
    """Return unique values while preserving order.

    Args:
        values: Candidate values.
        limit: Maximum number of values to return.

    Returns:
        Ordered de-duplicated values.
    """
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        if value and value not in seen:
            seen.add(value)
            out.append(value)
        if len(out) >= limit:
            break
    return out


def extract_org(issue_view: JsonValue) -> str | None:
    """Extract an organization slug from issue view output.

    Args:
        issue_view: Parsed `sentry issue view` output.

    Returns:
        Organization slug when present, otherwise `None`.
    """
    if not isinstance(issue_view, dict):
        return None
    candidates = [
        issue_view.get("org"),
        issue_view.get("organization"),
        issue_view.get("project", {}).get("organization")
        if isinstance(issue_view.get("project"), dict)
        else None,
    ]
    for candidate in candidates:
        if isinstance(candidate, dict):
            slug = candidate.get("slug") or candidate.get("id")
            if slug:
                return str(slug)
        elif isinstance(candidate, str):
            return candidate
    return None


def extract_issue_id(issue_view: JsonValue) -> str | None:
    """Extract the numeric issue ID from issue view output.

    Args:
        issue_view: Parsed `sentry issue view` output.

    Returns:
        Numeric issue ID or group ID when present, otherwise `None`.
    """
    if isinstance(issue_view, dict):
        issue_id = issue_view.get("id") or issue_view.get("groupID")
        if issue_id:
            return str(issue_id)
    return None


def issue_tag_values_endpoint(org: str, issue_id: str, tag: str) -> str:
    """Build the relative Sentry API path for issue tag values.

    Args:
        org: Organization slug.
        issue_id: Numeric issue ID.
        tag: Tag key.

    Returns:
        Relative API endpoint path.
    """
    return (
        f"organizations/{quote(org, safe='')}/"
        f"issues/{quote(issue_id, safe='')}/"
        f"tags/{quote(tag, safe='')}/values/"
    )


def render_json(bundle: dict[str, Any]) -> str:
    """Render a context bundle as stable JSON.

    Args:
        bundle: Context bundle.

    Returns:
        Pretty-printed JSON string ending with a newline.
    """
    return json.dumps(bundle, indent=2, sort_keys=True) + "\n"


def render_markdown(bundle: dict[str, Any]) -> str:
    """Render a context bundle as Markdown.

    Args:
        bundle: Context bundle.

    Returns:
        Markdown report string ending with a newline.
    """
    lines = [
        "# Sentry Issue Context",
        "",
        f"- Issue: `{bundle['issue']}`",
        f"- Generated: `{bundle['generated_at']}`",
        f"- Period: `{bundle['period']}`",
        "",
        "## Commands",
        "",
    ]
    for command in bundle["commands"]:
        args = " ".join(command.get("args", []))
        lines.append(f"- `{args}` -> `{command.get('returncode')}`")
        if command.get("stderr"):
            lines.append(f"  - stderr: `{command['stderr']}`")
    for label, data in bundle["sections"].items():
        lines.extend(
            [
                "",
                f"## {label}",
                "",
                "```json",
                json.dumps(data, indent=2, sort_keys=True),
                "```",
            ]
        )
    return "\n".join(lines) + "\n"


def build_parser() -> argparse.ArgumentParser:
    """Build the command-line parser.

    Returns:
        Configured argument parser.
    """
    parser = argparse.ArgumentParser(
        description=(
            "Collect redacted Sentry issue context with the sentry CLI."
        ),
    )
    parser.add_argument(
        "issue",
        help="Issue short ID, numeric ID, URL, or selector",
    )
    parser.add_argument(
        "--period",
        default="24h",
        help="Time window for events/logs",
    )
    parser.add_argument(
        "--limit-events",
        type=positive_int,
        default=5,
        help="Event limit",
    )
    parser.add_argument("--fresh", action="store_true", help="Bypass CLI cache")
    parser.add_argument(
        "--include-seer",
        action="store_true",
        help="Run issue explain",
    )
    parser.add_argument(
        "--include-plan",
        action="store_true",
        help="Run issue plan",
    )
    parser.add_argument(
        "--trace-id",
        action="append",
        default=[],
        help="Trace ID",
    )
    parser.add_argument(
        "--replay-id",
        action="append",
        default=[],
        help="Replay ID",
    )
    parser.add_argument(
        "--tag",
        action="append",
        default=[],
        help="Issue tag values",
    )
    parser.add_argument("--org", help="Org slug for API fallbacks")
    parser.add_argument(
        "--timeout",
        type=positive_int,
        default=120,
        help="Command timeout",
    )
    parser.add_argument(
        "--max-string",
        type=positive_int,
        default=500,
        help="Max string length",
    )
    parser.add_argument(
        "--format",
        choices=["markdown", "json"],
        default="markdown",
    )
    parser.add_argument("--out", type=Path, help="Optional output path")
    return parser


def main() -> int:
    """Run the context collector.

    Returns:
        Process exit code.
    """
    args = build_parser().parse_args()
    fresh = ["--fresh"] if args.fresh else []
    bundle: dict[str, Any] = {
        "issue": args.issue,
        "period": args.period,
        "generated_at": dt.datetime.now(dt.UTC).isoformat(),
        "commands": [],
        "sections": {},
    }

    issue_view = add_json_command(
        bundle,
        "issue view",
        [
            "issue",
            "view",
            args.issue,
            *fresh,
            "--json",
            "--fields",
            ISSUE_VIEW_FIELDS,
        ],
        args.timeout,
        args.max_string,
    )

    events = add_json_command(
        bundle,
        "issue events",
        [
            "issue",
            "events",
            args.issue,
            "--full",
            "--period",
            args.period,
            "--limit",
            str(args.limit_events),
            *fresh,
            "--json",
            "--fields",
            EVENT_FIELDS,
        ],
        args.timeout,
        args.max_string,
    )

    if args.include_seer:
        add_json_command(
            bundle,
            "issue explain",
            ["issue", "explain", args.issue, *fresh, "--json"],
            args.timeout,
            args.max_string,
        )
    if args.include_plan:
        add_json_command(
            bundle,
            "issue plan",
            ["issue", "plan", args.issue, *fresh, "--json"],
            args.timeout,
            args.max_string,
        )

    trace_ids = unique(
        [
            *args.trace_id,
            *nested_values(issue_view, {"traceid", "trace_id"}),
            *nested_values(events, {"traceid", "trace_id"}),
            *tag_values(events, "trace"),
        ],
        3,
    )
    for trace_id in trace_ids:
        add_json_command(
            bundle,
            f"trace view {trace_id}",
            ["trace", "view", trace_id, "--full", *fresh, "--json"],
            args.timeout,
            args.max_string,
        )
        add_json_command(
            bundle,
            f"trace logs {trace_id}",
            [
                "trace",
                "logs",
                trace_id,
                "--period",
                args.period,
                "--limit",
                "50",
                *fresh,
                "--json",
            ],
            args.timeout,
            args.max_string,
        )

    replay_ids = unique(
        [
            *args.replay_id,
            *nested_values(issue_view, {"replayid", "replay_id"}),
            *nested_values(events, {"replayid", "replay_id"}),
            *tag_values(events, "replayId"),
        ],
        3,
    )
    for replay_id in replay_ids:
        add_json_command(
            bundle,
            f"replay view {replay_id}",
            ["replay", "view", replay_id, *fresh, "--json"],
            args.timeout,
            args.max_string,
        )

    org = args.org or extract_org(issue_view)
    issue_id = extract_issue_id(issue_view)
    for tag in args.tag:
        if org and issue_id:
            endpoint = issue_tag_values_endpoint(org, issue_id, tag)
            add_json_command(
                bundle,
                f"tag values {tag}",
                ["api", endpoint, "--json"],
                args.timeout,
                args.max_string,
            )
        else:
            bundle["sections"][f"tag values {tag}"] = {
                "skipped": "org slug or numeric issue id unavailable"
            }

    output = (
        render_json(bundle)
        if args.format == "json"
        else render_markdown(bundle)
    )
    if args.out:
        args.out.write_text(output)
    else:
        sys.stdout.write(output)

    issue_command = bundle["commands"][0]
    return 0 if issue_command.get("returncode") == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
