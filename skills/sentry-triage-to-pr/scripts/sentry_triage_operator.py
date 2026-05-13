#!/usr/bin/env python3
"""Portable Sentry triage operator.

The operator is intentionally read-only against external systems. It may call
the `sentry` CLI for capture and writes local redacted artifacts, but GitHub,
git, worktree, and Sentry mutations are rendered as command plans.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import math
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any


BUNDLE_SCHEMA = "sentry-triage-to-pr.bundle.v1"
MAX_STRING = 700
DEFAULT_FIELDS = (
    "id,shortId,title,culprit,count,userCount,firstSeen,lastSeen,level,status,"
    "substatus,priority,platform,permalink,project,metadata,assignedTo,"
    "isUnhandled,seerFixabilityScore"
)
VIEW_FIELDS = (
    "id,shortId,title,culprit,count,userCount,firstSeen,lastSeen,level,status,"
    "substatus,priority,platform,permalink,project,metadata,assignedTo,"
    "isUnhandled,event,trace,replayIds,seerFixabilityScore"
)
EVENT_FIELDS = (
    "id,eventID,groupID,projectID,message,title,location,culprit,user,tags,"
    "platform,dateCreated,crashFile,metadata"
)

SENSITIVE_KEYS = {
    "authorization",
    "cookie",
    "cookies",
    "headers",
    "password",
    "passwd",
    "secret",
    "session",
    "token",
    "api_key",
    "apikey",
    "dsn",
    "jwt",
    "credential",
    "credentials",
    "private_key",
    "request",
    "body",
    "user",
    "email",
    "ip",
    "ip_address",
    "customer",
    "account",
    "prompt",
    "completion",
    "attachment",
}
SENSITIVE_KEY_PART_RE = re.compile(
    r"(authorization|cookie|secret|password|passwd|api[_-]?key|dsn|jwt|"
    r"bearer|credential|session|private[_-]?key)",
    re.IGNORECASE,
)
EMAIL_RE = re.compile(r"(?<![\w.+-])[\w.+-]+@[\w.-]+\.[A-Za-z]{2,}(?![\w.-])")
BEARER_RE = re.compile(r"\bBearer\s+[A-Za-z0-9._~+/=-]+", re.IGNORECASE)
DSN_RE = re.compile(r"https?://[^@\s]+@[^/\s]+/\d+")
IPV4_RE = re.compile(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
LONG_TOKEN_RE = re.compile(r"\b[A-Za-z0-9_-]{40,}\b")
UUID_RE = re.compile(
    r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-"
    r"[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b"
)
WORD_RE = re.compile(r"[a-z0-9]+")


class OperatorError(Exception):
    """User-facing CLI error without traceback noise."""


def utc_now() -> str:
    return dt.datetime.now(dt.UTC).replace(microsecond=0).isoformat()


def normalize_key(key: str) -> str:
    return key.replace("-", "_").lower()


def is_sensitive_key(key: str | None) -> bool:
    if key is None:
        return False
    normalized = normalize_key(key)
    if normalized in SENSITIVE_KEYS:
        return True
    return bool(SENSITIVE_KEY_PART_RE.search(normalized))


def redact_scalar(value: str, max_string: int = MAX_STRING) -> str:
    value = EMAIL_RE.sub("[REDACTED_EMAIL]", value)
    value = BEARER_RE.sub("Bearer [REDACTED]", value)
    value = DSN_RE.sub("[REDACTED_DSN]", value)
    value = IPV4_RE.sub("[REDACTED_IP]", value)
    value = UUID_RE.sub("[REDACTED_UUID]", value)
    value = LONG_TOKEN_RE.sub("[REDACTED_TOKEN]", value)
    if len(value) > max_string:
        value = value[:max_string] + "...[TRUNCATED]"
    return value


def redact(value: Any, max_string: int = MAX_STRING, key: str | None = None) -> Any:
    if is_sensitive_key(key):
        return "[REDACTED]"
    if isinstance(value, dict):
        return {str(k): redact(v, max_string, str(k)) for k, v in value.items()}
    if isinstance(value, list):
        return [redact(item, max_string) for item in value]
    if isinstance(value, str):
        return redact_scalar(value, max_string)
    return value


def command_record(command: list[str], returncode: int, stderr: str | None = None) -> dict[str, Any]:
    record: dict[str, Any] = {
        "args": command,
        "returncode": returncode,
    }
    if stderr:
        record["stderr"] = redact_scalar(stderr, 500)
    return record


def run_sentry(args: list[str], timeout: int) -> tuple[dict[str, Any], Any]:
    command = ["sentry", *args]
    try:
        proc = subprocess.run(
            command,
            check=False,
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except FileNotFoundError:
        return command_record(command, 127, "sentry CLI not found on PATH"), None
    except subprocess.TimeoutExpired as exc:
        stderr = exc.stderr if isinstance(exc.stderr, str) else ""
        stdout = exc.stdout if isinstance(exc.stdout, str) else ""
        parsed = {"stdout": stdout} if stdout else None
        return command_record(command, 124, f"timed out after {timeout}s\n{stderr}"), parsed

    parsed: Any = None
    if proc.stdout.strip():
        try:
            parsed = json.loads(proc.stdout)
        except json.JSONDecodeError as exc:
            parsed = {"parse_error": str(exc), "raw": proc.stdout}
    return command_record(command, proc.returncode, proc.stderr.strip() or None), redact(parsed)


def load_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text())
    except FileNotFoundError as exc:
        raise OperatorError(f"{path} does not exist") from exc
    except json.JSONDecodeError as exc:
        raise OperatorError(f"{path} is not valid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise OperatorError(f"{path} must contain a JSON object")
    return data


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def emit_json(data: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(data, indent=2, sort_keys=True) + "\n")


def bundle_base(target: str | None, query: str | None, period: str) -> dict[str, Any]:
    return {
        "schema": BUNDLE_SCHEMA,
        "generated_at": utc_now(),
        "target": target,
        "query": query,
        "period": period,
        "commands": [],
        "issues": [],
        "issue_contexts": {},
    }


def as_issue_list(value: Any) -> list[dict[str, Any]]:
    if isinstance(value, list):
        return [item for item in value if isinstance(item, dict)]
    if isinstance(value, dict):
        for key in ("issues", "data", "results"):
            child = value.get(key)
            if isinstance(child, list):
                return [item for item in child if isinstance(item, dict)]
    return []


def candidate_issues(bundle: dict[str, Any]) -> list[dict[str, Any]]:
    issues = as_issue_list(bundle.get("issues"))
    seen = {short_id(issue) for issue in issues}
    contexts = bundle.get("issue_contexts")
    if isinstance(contexts, dict):
        for context in contexts.values():
            if not isinstance(context, dict):
                continue
            view = context.get("view")
            if isinstance(view, dict) and short_id(view) not in seen:
                issues.append(view)
                seen.add(short_id(view))
    return issues


def short_id(issue: dict[str, Any]) -> str:
    value = issue.get("shortId") or issue.get("short_id") or issue.get("id") or "unknown"
    return str(value)


def project_slug(issue: dict[str, Any]) -> str:
    project = issue.get("project")
    if isinstance(project, dict):
        return str(project.get("slug") or project.get("name") or project.get("id") or "unknown")
    if isinstance(project, str):
        return project
    return "unknown"


def parse_count(value: Any) -> float:
    if value is None:
        return 0.0
    if isinstance(value, (int, float)):
        return float(value)
    text = str(value).strip().lower().replace(",", "")
    multiplier = 1.0
    if text.endswith("k"):
        multiplier = 1_000.0
        text = text[:-1]
    elif text.endswith("m"):
        multiplier = 1_000_000.0
        text = text[:-1]
    try:
        return float(text) * multiplier
    except ValueError:
        return 0.0


def parse_time(value: Any) -> dt.datetime | None:
    if not value:
        return None
    text = str(value).replace("Z", "+00:00")
    try:
        parsed = dt.datetime.fromisoformat(text)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=dt.UTC)
    return parsed.astimezone(dt.UTC)


def bounded_log_score(value: float) -> float:
    if value <= 0:
        return 0.0
    return min(10.0, math.log10(value + 1.0) * 2.5)


def recency_score(last_seen: Any) -> float:
    parsed = parse_time(last_seen)
    if parsed is None:
        return 2.0
    age = dt.datetime.now(dt.UTC) - parsed
    if age <= dt.timedelta(hours=24):
        return 10.0
    if age <= dt.timedelta(days=7):
        return 7.0
    if age <= dt.timedelta(days=30):
        return 4.0
    return 1.0


def field_text(issue: dict[str, Any], key: str) -> str:
    value = issue.get(key)
    return "" if value is None else str(value).lower()


def score_issue(issue: dict[str, Any]) -> dict[str, Any]:
    count = parse_count(issue.get("count"))
    users = parse_count(issue.get("userCount"))
    priority = field_text(issue, "priority")
    level = field_text(issue, "level")
    substatus = field_text(issue, "substatus")

    priority_score = {
        "critical": 10.0,
        "high": 9.0,
        "medium": 6.0,
        "low": 3.0,
    }.get(priority, 2.0)
    level_score = {
        "fatal": 10.0,
        "error": 8.0,
        "warning": 4.0,
        "info": 1.0,
    }.get(level, 2.0)
    unhandled_score = 10.0 if issue.get("isUnhandled") is True else 4.0
    substatus_score = 9.0 if substatus in {"regressed", "escalating", "for_review", "new"} else 4.0

    impact = (bounded_log_score(count) * 0.45) + (bounded_log_score(users) * 0.45) + (
        recency_score(issue.get("lastSeen")) * 0.10
    )
    urgency = (
        priority_score * 0.35
        + level_score * 0.25
        + unhandled_score * 0.25
        + substatus_score * 0.15
    )
    seer_score = issue.get("seerFixabilityScore")
    if isinstance(seer_score, (int, float)):
        fixability = max(0.0, min(10.0, float(seer_score) * 10.0))
    else:
        fixability = 3.0
    if issue.get("culprit"):
        fixability += 1.5
    if project_slug(issue) != "unknown":
        fixability += 1.0
    fixability = min(10.0, fixability)

    total = (impact * 0.40 + urgency * 0.35 + fixability * 0.25) * 10.0
    return {
        "issue": issue,
        "score": round(total, 2),
        "components": {
            "impact": round(impact, 2),
            "urgency": round(urgency, 2),
            "fixability": round(fixability, 2),
            "priority": priority or None,
            "users": users,
            "events": count,
            "recency": round(recency_score(issue.get("lastSeen")), 2),
        },
    }


def normalize_title(title: str) -> str:
    title = title.lower()
    title = UUID_RE.sub(" id ", title)
    title = re.sub(r"\b[0-9a-f]{8,}\b", " id ", title)
    title = re.sub(r"\b\d+\b", " n ", title)
    words = WORD_RE.findall(title)
    stop = {"the", "a", "an", "in", "on", "for", "with", "and", "or", "at", "of"}
    words = [word for word in words if word not in stop]
    return "-".join(words[:8]) or "sentry-issue"


def slugify(text: str, limit: int = 48) -> str:
    words = WORD_RE.findall(text.lower())
    slug = "-".join(words)[:limit].strip("-")
    return slug or "sentry-fix"


def group_key(issue: dict[str, Any]) -> str:
    title = normalize_title(str(issue.get("title") or short_id(issue)))
    culprit = slugify(str(issue.get("culprit") or "unknown"), 40)
    return f"{project_slug(issue)}::{title}::{culprit}"


def issue_ids_for_group(group: dict[str, Any]) -> list[str]:
    return [short_id(item["issue"] if "issue" in item else item) for item in group.get("ranked_issues", [])]


def branch_for_group(group: dict[str, Any]) -> str:
    ids = "-".join(issue_ids_for_group(group)[:3]).lower()
    title = group.get("title_slug") or "sentry-fix"
    return f"fix/sentry-{ids}-{title}"[:96].rstrip("-")


def collect_values(value: Any, wanted: set[str]) -> list[str]:
    out: list[str] = []
    if isinstance(value, dict):
        for key, child in value.items():
            normalized = normalize_key(str(key))
            if normalized in wanted:
                if isinstance(child, str):
                    out.append(child)
                elif isinstance(child, list):
                    out.extend(str(item) for item in child if isinstance(item, (str, int, float)))
            out.extend(collect_values(child, wanted))
    elif isinstance(value, list):
        for child in value:
            out.extend(collect_values(child, wanted))
    return out


def tag_values(value: Any, wanted: set[str]) -> list[str]:
    out: list[str] = []
    if isinstance(value, dict):
        tags = value.get("tags")
        if isinstance(tags, list):
            for tag in tags:
                if not isinstance(tag, dict):
                    continue
                key = normalize_key(str(tag.get("key") or tag.get("name") or ""))
                if key in wanted and tag.get("value") is not None:
                    out.append(str(tag["value"]))
        for child in value.values():
            out.extend(tag_values(child, wanted))
    elif isinstance(value, list):
        for child in value:
            out.extend(tag_values(child, wanted))
    return out


def unique(values: list[str], limit: int) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        if value and value not in seen:
            seen.add(value)
            out.append(value)
        if len(out) >= limit:
            break
    return out


def add_context(bundle: dict[str, Any], issue_id: str, args: argparse.Namespace) -> None:
    context: dict[str, Any] = {}
    fresh = ["--fresh"] if args.fresh else []
    commands = [
        (
            "view",
            ["issue", "view", issue_id, "--spans", str(args.spans), *fresh, "--json", "--fields", VIEW_FIELDS],
        ),
        (
            "events",
            [
                "issue",
                "events",
                issue_id,
                "--full",
                "--period",
                args.period,
                "--limit",
                str(args.event_limit),
                *fresh,
                "--json",
                "--fields",
                EVENT_FIELDS,
            ],
        ),
    ]
    if args.include_seer:
        commands.append(("explain", ["issue", "explain", issue_id, *fresh, "--json"]))
    if args.include_plan:
        commands.append(("plan", ["issue", "plan", issue_id, *fresh, "--json"]))

    for label, command in commands:
        record, parsed = run_sentry(command, args.timeout)
        bundle["commands"].append(record)
        context[label] = parsed

    if not args.no_traces:
        trace_ids = unique(
            collect_values(context, {"traceid", "trace_id", "trace"})
            + tag_values(context, {"traceid", "trace_id", "trace"})
            + args.trace_id,
            args.max_related_ids,
        )
        for trace_id in trace_ids:
            record, parsed = run_sentry(
                [
                    "trace",
                    "logs",
                    trace_id,
                    "--period",
                    args.period,
                    "--limit",
                    str(args.trace_log_limit),
                    *fresh,
                    "--json",
                ],
                args.timeout,
            )
            bundle["commands"].append(record)
            context[f"trace logs {trace_id}"] = parsed

    if not args.no_replays:
        replay_ids = unique(
            collect_values(context, {"replayid", "replay_id", "replayids", "replay"})
            + tag_values(context, {"replayid", "replay_id", "replayids", "replay"})
            + args.replay_id,
            args.max_related_ids,
        )
        for replay_id in replay_ids:
            record, parsed = run_sentry(["replay", "view", replay_id, *fresh, "--json"], args.timeout)
            bundle["commands"].append(record)
            context[f"replay {replay_id}"] = parsed

    bundle.setdefault("issue_contexts", {})[issue_id] = context


def command_capture(args: argparse.Namespace) -> int:
    issue_ids = list(args.issue)
    list_query = args.query or ("is:unresolved" if args.target or not issue_ids else None)
    bundle = bundle_base(args.target, list_query, args.period)

    if list_query:
        command = [
            "issue",
            "list",
            *( [args.target] if args.target else [] ),
            "--query",
            list_query,
            "--period",
            args.period,
            "--limit",
            str(args.limit),
            "--sort",
            args.sort,
            *( ["--fresh"] if args.fresh else [] ),
            "--json",
            "--fields",
            DEFAULT_FIELDS,
        ]
        record, parsed = run_sentry(command, args.timeout)
        bundle["commands"].append(record)
        bundle["issues"] = as_issue_list(parsed)
        if args.hydrate_top:
            issue_ids.extend(short_id(issue) for issue in bundle["issues"][: args.hydrate_top])

    for issue_id in unique(issue_ids, args.max_issues):
        add_context(bundle, issue_id, args)

    write_json(args.out, bundle)
    return 0 if all(command.get("returncode") == 0 for command in bundle["commands"]) else 1


def command_triage(args: argparse.Namespace) -> int:
    bundle = load_json(args.bundle)
    ranked = [score_issue(issue) for issue in candidate_issues(bundle)]
    ranked.sort(key=lambda item: item["score"], reverse=True)
    for index, item in enumerate(ranked, start=1):
        item["rank"] = index
    bundle["ranked_issues"] = ranked
    bundle["triage"] = {
        "objective": "impact_plus_fixability",
        "scored_at": utc_now(),
        "issue_count": len(ranked),
    }
    write_json(args.out, bundle)
    return 0


def command_group(args: argparse.Namespace) -> int:
    bundle = load_json(args.bundle)
    ranked = bundle.get("ranked_issues") or [score_issue(issue) for issue in candidate_issues(bundle)]
    buckets: dict[str, list[dict[str, Any]]] = {}
    for item in ranked:
        issue = item["issue"] if isinstance(item, dict) and "issue" in item else item
        buckets.setdefault(group_key(issue), []).append(item)

    groups: list[dict[str, Any]] = []
    for key, items in sorted(buckets.items()):
        issues = [item["issue"] for item in items]
        top_issue = issues[0]
        title_slug = normalize_title(str(top_issue.get("title") or short_id(top_issue)))
        group = {
            "id": "",
            "key": key,
            "title": str(top_issue.get("title") or short_id(top_issue)),
            "title_slug": title_slug,
            "project": project_slug(top_issue),
            "ranked_issues": items,
            "score": round(max(float(item.get("score", 0.0)) for item in items), 2),
            "aggregate_score": round(sum(float(item.get("score", 0.0)) for item in items), 2),
            "suspected_surface": str(top_issue.get("culprit") or "UNVERIFIED"),
            "branch": "",
            "parallel_safe": len(items) == 1,
            "notes": [],
        }
        if project_slug(top_issue) == "unknown" or not top_issue.get("culprit"):
            group["notes"].append("needs_manual_triage: ownership or culprit is incomplete")
            group["parallel_safe"] = False
        groups.append(group)

    groups.sort(key=lambda group: (group["score"], group["aggregate_score"]), reverse=True)
    for index, group in enumerate(groups, start=1):
        group["id"] = f"sentry-group-{index:03d}"
        group["branch"] = branch_for_group(group)
    bundle["groups"] = groups
    bundle["grouped_at"] = utc_now()
    write_json(args.out, bundle)
    return 0


def github_body(group: dict[str, Any]) -> str:
    ids = issue_ids_for_group(group)
    lines = [
        f"<!-- sentry-triage-to-pr:v1 group={group['id']} issues={','.join(ids)} -->",
        f"# Fix Sentry group {group['id']}",
        "",
        "## Sentry Issues",
        "",
    ]
    for item in group.get("ranked_issues", []):
        issue = item["issue"]
        link = issue.get("permalink") or "UNVERIFIED"
        lines.append(
            f"- `{short_id(issue)}` score `{item.get('score')}` priority `{issue.get('priority') or 'unknown'}` "
            f"users `{issue.get('userCount') or 0}` events `{issue.get('count') or 0}`: {link}"
        )
    lines.extend(
        [
            "",
            "## Suspected Surface",
            "",
            f"- Project: `{group.get('project', 'unknown')}`",
            f"- Surface: `{group.get('suspected_surface', 'UNVERIFIED')}`",
            "",
            "## Resolution Plan",
            "",
            "- Verify the stack frames and latest representative event in Sentry.",
            "- Reproduce or unit-test the suspected failure path with synthetic data.",
            "- Patch the smallest owning implementation surface.",
            "- Run focused tests and repo-native validation.",
            "- Open a PR with Sentry links and verification evidence.",
            "",
            "## Privacy",
            "",
            "This issue intentionally excludes raw event payloads, user data, request bodies, headers, breadcrumbs, prompts, completions, and replay content.",
        ]
    )
    return "\n".join(lines) + "\n"


def shell_join(parts: list[str]) -> str:
    return shlex.join(parts)


def command_render_github(args: argparse.Namespace) -> int:
    bundle = load_json(args.bundle)
    groups = bundle.get("groups", [])
    if not isinstance(groups, list):
        raise SystemExit("bundle groups must be a list")
    args.out_dir.mkdir(parents=True, exist_ok=True)

    plan: list[dict[str, Any]] = []
    for group in groups[: args.limit]:
        ids = issue_ids_for_group(group)
        title = f"fix(sentry): {group.get('title_slug', group['id']).replace('-', ' ')}"
        body_path = args.out_dir / f"{group['id']}.md"
        body = github_body(group)
        body_path.write_text(body)
        labels = list(args.label)
        marker_search = f"sentry-triage-to-pr:v1 {' '.join(ids)}"
        create_cmd = [
            "gh",
            "issue",
            "create",
            "--repo",
            args.repo,
            "--title",
            title,
            "--body-file",
            str(body_path),
        ]
        for label in labels:
            create_cmd.extend(["--label", label])
        plan.append(
            {
                "group_id": group["id"],
                "issue_ids": ids,
                "title": title,
                "body_file": str(body_path),
                "dedupe_command": shell_join(
                    ["gh", "issue", "list", "--repo", args.repo, "--search", marker_search]
                ),
                "create_command": shell_join(create_cmd),
                "create_args": create_cmd,
            }
        )
    output = {
        "schema": BUNDLE_SCHEMA,
        "generated_at": utc_now(),
        "repo": args.repo,
        "github_plan": plan,
    }
    write_json(args.out_dir / "github-plan.json", output)
    return 0


def command_plan_worktrees(args: argparse.Namespace) -> int:
    bundle = load_json(args.bundle)
    groups = bundle.get("groups", [])
    repo_root = args.repo_root.resolve()
    worktree_root = (args.worktree_root or repo_root.parent / f"{repo_root.name}-sentry-worktrees").resolve()
    assignments: list[dict[str, Any]] = []
    for group in groups[: args.limit]:
        branch = group.get("branch") or branch_for_group(group)
        worktree_path = worktree_root / slugify(branch.replace("/", "-"), 96)
        ids = issue_ids_for_group(group)
        prompt = (
            f"Task: Fix Sentry group {group['id']} ({', '.join(ids)}) on branch {branch}.\n"
            f"Scope: worktree {worktree_path}; suspected surface {group.get('suspected_surface', 'UNVERIFIED')}.\n"
            "Mode: edit only files needed for this issue group. Do not revert other work.\n"
            "Return: files changed, tests run, Sentry evidence used, residual risks, PR status."
        )
        assignments.append(
            {
                "group_id": group["id"],
                "issue_ids": ids,
                "branch": branch,
                "worktree_path": str(worktree_path),
                "parallel_safe": bool(group.get("parallel_safe")),
                "create_command": shell_join(
                    [
                        "git",
                        "worktree",
                        "add",
                        str(worktree_path),
                        "-b",
                        branch,
                        args.base_branch,
                    ]
                ),
                "subspawn_prompt": prompt,
            }
        )
    write_json(
        args.out,
        {
            "schema": BUNDLE_SCHEMA,
            "generated_at": utc_now(),
            "base_branch": args.base_branch,
            "max_parallel": args.max_parallel,
            "worktree_plan": assignments,
        },
    )
    return 0


def find_sensitive_strings(value: Any, path: str = "$") -> list[str]:
    findings: list[str] = []
    if isinstance(value, dict):
        for key, child in value.items():
            if is_sensitive_key(str(key)) and child not in (None, "", "[REDACTED]"):
                findings.append(f"{path}.{key}: sensitive key is not redacted")
            findings.extend(find_sensitive_strings(child, f"{path}.{key}"))
    elif isinstance(value, list):
        for index, child in enumerate(value):
            findings.extend(find_sensitive_strings(child, f"{path}[{index}]"))
    elif isinstance(value, str):
        if EMAIL_RE.search(value) or BEARER_RE.search(value) or DSN_RE.search(value) or IPV4_RE.search(value):
            findings.append(f"{path}: sensitive pattern found")
    return findings


def command_validate_bundle(args: argparse.Namespace) -> int:
    try:
        bundle = load_json(args.bundle)
    except OperatorError as exc:
        if args.json:
            emit_json(
                {
                    "schema": "sentry-triage-to-pr.validation.v1",
                    "ok": False,
                    "errors": [str(exc)],
                }
            )
            return 2
        print(str(exc), file=sys.stderr)
        return 2
    errors: list[str] = []
    if bundle.get("schema") != BUNDLE_SCHEMA:
        errors.append(f"schema must be {BUNDLE_SCHEMA}")
    if "generated_at" not in bundle:
        errors.append("generated_at is required")
    errors.extend(find_sensitive_strings(bundle))
    if errors:
        if args.json:
            emit_json(
                {
                    "schema": "sentry-triage-to-pr.validation.v1",
                    "ok": False,
                    "errors": errors,
                }
            )
            return 1
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    if args.json:
        emit_json({"schema": "sentry-triage-to-pr.validation.v1", "ok": True, "errors": []})
        return 0
    print("bundle valid")
    return 0


def probe_tool(name: str, version_args: list[str], timeout: int) -> dict[str, Any]:
    path = shutil.which(name)
    result: dict[str, Any] = {
        "name": name,
        "available": bool(path),
        "path": path,
    }
    if not path:
        return result
    try:
        proc = subprocess.run(
            [name, *version_args],
            capture_output=True,
            check=False,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        result.update({"ok": False, "returncode": 124, "version": "timeout"})
        return result
    output = (proc.stdout or proc.stderr).strip().splitlines()
    result.update(
        {
            "ok": proc.returncode == 0,
            "returncode": proc.returncode,
            "version": redact_scalar(output[0], 200) if output else None,
        }
    )
    return result


def command_doctor(args: argparse.Namespace) -> int:
    tools = [
        probe_tool("sentry", ["--version"], args.timeout),
        probe_tool("gh", ["--version"], args.timeout),
        probe_tool("git", ["--version"], args.timeout),
        probe_tool("codex", ["--version"], args.timeout),
    ]
    if args.include_auth:
        auth_checks = [
            ("sentry", ["auth", "whoami"]),
            ("gh", ["auth", "status"]),
        ]
        for name, command_args in auth_checks:
            if shutil.which(name):
                record, _ = run_sentry(command_args, args.timeout) if name == "sentry" else ({}, None)
                if name == "gh":
                    try:
                        proc = subprocess.run(
                            [name, *command_args],
                            capture_output=True,
                            check=False,
                            text=True,
                            timeout=args.timeout,
                        )
                        record = command_record([name, *command_args], proc.returncode, proc.stderr)
                    except subprocess.TimeoutExpired:
                        record = command_record([name, *command_args], 124, "timed out")
                tools.append({"name": f"{name} auth", "available": True, "ok": record.get("returncode") == 0})
    report = {
        "schema": "sentry-triage-to-pr.doctor.v1",
        "ok": all(tool.get("available") and tool.get("ok", True) for tool in tools if tool["name"] in {"sentry", "gh", "git"}),
        "generated_at": utc_now(),
        "tools": tools,
    }
    if args.json:
        emit_json(report)
    else:
        status = "ok" if report["ok"] else "needs setup"
        print(f"doctor: {status}")
        for tool in tools:
            available = "yes" if tool.get("available") else "no"
            version = f" ({tool.get('version')})" if tool.get("version") else ""
            print(f"- {tool['name']}: {available}{version}")
    return 0 if report["ok"] else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Read-only Sentry triage-to-PR operator.")
    sub = parser.add_subparsers(dest="command", required=True)

    doctor = sub.add_parser("doctor", help="Check local CLI dependencies without mutating state")
    doctor.add_argument("--json", action="store_true", help="Emit JSON report")
    doctor.add_argument("--include-auth", action="store_true", help="Also run Sentry and GitHub auth probes")
    doctor.add_argument("--timeout", type=int, default=15)
    doctor.set_defaults(func=command_doctor)

    capture = sub.add_parser("capture", help="Capture redacted Sentry issue data")
    capture.add_argument("--target", help="Sentry target such as ORG/ or ORG/PROJECT")
    capture.add_argument("--query", help="Sentry issue search query; defaults to is:unresolved when listing")
    capture.add_argument("--period", default="7d", help="Sentry time period")
    capture.add_argument("--limit", type=int, default=100, help="Issue list limit")
    capture.add_argument("--sort", default="user", choices=["date", "new", "freq", "user"])
    capture.add_argument("--issue", action="append", default=[], help="Issue ID to hydrate")
    capture.add_argument("--hydrate-top", type=int, default=0, help="Hydrate the first N listed issues")
    capture.add_argument("--event-limit", type=int, default=5, help="Representative event limit")
    capture.add_argument("--spans", default="3", help="Span depth for issue view")
    capture.add_argument("--include-seer", action="store_true", help="Run sentry issue explain")
    capture.add_argument("--include-plan", action="store_true", help="Run sentry issue plan")
    capture.add_argument("--trace-id", action="append", default=[], help="Trace ID to capture logs for")
    capture.add_argument("--replay-id", action="append", default=[], help="Replay ID to capture")
    capture.add_argument("--no-traces", action="store_true", help="Skip detected trace log capture")
    capture.add_argument("--no-replays", action="store_true", help="Skip detected replay capture")
    capture.add_argument("--trace-log-limit", type=int, default=50)
    capture.add_argument("--max-related-ids", type=int, default=3)
    capture.add_argument("--max-issues", type=int, default=25)
    capture.add_argument("--fresh", action="store_true")
    capture.add_argument("--timeout", type=int, default=120)
    capture.add_argument("--out", type=Path, required=True)
    capture.set_defaults(func=command_capture)

    triage = sub.add_parser("triage", help="Score captured issues")
    triage.add_argument("bundle", type=Path)
    triage.add_argument("--out", type=Path, required=True)
    triage.set_defaults(func=command_triage)

    group = sub.add_parser("group", help="Group ranked issues into fix units")
    group.add_argument("bundle", type=Path)
    group.add_argument("--out", type=Path, required=True)
    group.set_defaults(func=command_group)

    github = sub.add_parser("render-github", help="Render GitHub issue body and command plans")
    github.add_argument("bundle", type=Path)
    github.add_argument("--repo", required=True, help="GitHub repo in OWNER/REPO form")
    github.add_argument("--out-dir", type=Path, required=True)
    github.add_argument("--label", action="append", default=["sentry", "production"])
    github.add_argument("--limit", type=int, default=20)
    github.set_defaults(func=command_render_github)

    worktrees = sub.add_parser("plan-worktrees", help="Render branch/worktree/subspawn plans")
    worktrees.add_argument("bundle", type=Path)
    worktrees.add_argument("--repo-root", type=Path, default=Path.cwd())
    worktrees.add_argument("--worktree-root", type=Path)
    worktrees.add_argument("--base-branch", default="main")
    worktrees.add_argument("--max-parallel", type=int, default=3)
    worktrees.add_argument("--limit", type=int, default=20)
    worktrees.add_argument("--out", type=Path, required=True)
    worktrees.set_defaults(func=command_plan_worktrees)

    validate = sub.add_parser("validate-bundle", help="Validate schema and redaction")
    validate.add_argument("bundle", type=Path)
    validate.add_argument("--json", action="store_true", help="Emit JSON validation result")
    validate.set_defaults(func=command_validate_bundle)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return int(args.func(args))
    except OperatorError as exc:
        print(str(exc), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
