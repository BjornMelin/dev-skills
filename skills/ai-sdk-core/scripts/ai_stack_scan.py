#!/usr/bin/env python3
"""Offline AI application stack scanner.

The scanner is intentionally dependency-free and conservative. It emits stable
JSON signals that should be verified against current docs before editing code.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import sys
from collections import Counter
from pathlib import Path
from typing import Any

SCHEMA = "ai_stack_scan.v1"
SCANNER_VERSION = "2026-05-12"

FAMILIES = {
    "ai-sdk-core",
    "ai-sdk-ui",
    "ai-sdk-agents",
    "streamdown",
    "zod-v4",
    "supabase-ts",
}

DOCS = {
    "ai-sdk-core": {
        "primary": "https://ai-sdk.dev/docs",
        "migration": "https://ai-sdk.dev/docs/migration-guides/migration-guide-5-0",
        "reference": "https://ai-sdk.dev/docs/reference/ai-sdk-core/step-count-is",
    },
    "ai-sdk-ui": {
        "primary": "https://ai-sdk.dev/docs/ai-sdk-ui",
        "migration": "https://ai-sdk.dev/docs/migration-guides/migration-guide-5-0",
        "reference": "https://ai-sdk.dev/docs/reference/ai-sdk-ui/use-chat",
    },
    "ai-sdk-agents": {
        "primary": "https://ai-sdk.dev/docs/agents",
        "reference": "https://ai-sdk.dev/docs/reference/ai-sdk-core/tool-loop-agent",
    },
    "streamdown": {
        "primary": "https://streamdown.ai",
        "source": "https://github.com/vercel/streamdown",
        "migration": "https://streamdown.ai/docs/migrate",
    },
    "zod-v4": {
        "primary": "https://zod.dev/v4",
        "migration": "https://zod.dev/v4/changelog",
        "reference": "https://zod.dev/api",
    },
    "supabase-ts": {
        "primary": "https://supabase.com/docs/guides/auth/server-side",
        "reference": "https://supabase.com/docs/reference/javascript/introduction",
        "source": "https://github.com/supabase/ssr",
    },
}

PACKAGE_FAMILIES = {
    "ai": {"ai-sdk-core", "ai-sdk-ui", "ai-sdk-agents"},
    "@ai-sdk/react": {"ai-sdk-ui"},
    "streamdown": {"streamdown"},
    "zod": {"zod-v4"},
    "@supabase/ssr": {"supabase-ts"},
    "@supabase/supabase-js": {"supabase-ts"},
}

EXCLUDED_DIRS = {
    ".git",
    ".next",
    ".nuxt",
    ".output",
    ".svelte-kit",
    ".turbo",
    ".vercel",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "out",
    "target",
}

SOURCE_SUFFIXES = {".css", ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mdx", ".sql"}
MANIFEST_SECTIONS = ("dependencies", "devDependencies", "peerDependencies", "optionalDependencies")
PUBLIC_ENV_RE = re.compile(r"\b(?:NEXT_PUBLIC|VITE|PUBLIC|EXPO_PUBLIC)_[A-Z0-9_]*SERVICE[_-]?ROLE[A-Z0-9_]*\b", re.I)
SERVICE_ROLE_NAME_RE = re.compile(r"\b[A-Z0-9_]*SUPABASE[A-Z0-9_]*SERVICE[_-]?ROLE[A-Z0-9_]*\b", re.I)
JWT_RE = re.compile(r"\beyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\b")
REDACTED = "[redacted]"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Scan an AI TypeScript stack and emit stable JSON.")
    parser.add_argument("--root", default=".", help="Repository root to scan.")
    parser.add_argument(
        "--family",
        action="append",
        choices=sorted(FAMILIES | {"all"}),
        help="Limit checks to one family. Repeatable. Defaults to all families.",
    )
    parser.add_argument("--max-files", type=int, default=3000, help="Maximum source files to inspect.")
    parser.add_argument("--max-dirs", type=int, default=5000, help="Maximum directories to traverse.")
    parser.add_argument("--max-manifests", type=int, default=200, help="Maximum package.json manifests to inspect.")
    parser.add_argument("--max-bytes", type=int, default=1_000_000, help="Maximum bytes per source file.")
    parser.add_argument("--include-evidence", action="store_true", help="Include sanitized source evidence snippets.")
    parser.add_argument("--include-absolute-root", action="store_true", help="Include the absolute scan root in JSON output.")
    parser.add_argument("--pretty", action="store_true", help="Pretty-print JSON output.")
    return parser.parse_args()


def selected_families(values: list[str] | None) -> set[str]:
    if values and "all" in values:
        return set(FAMILIES)
    if not values:
        for parent in Path(__file__).resolve().parents:
            if parent.name == "ai-sdk-core":
                return set(FAMILIES)
            if parent.name in FAMILIES:
                return {parent.name}
        return set(FAMILIES)
    return set(values)


def read_json(path: Path) -> dict[str, Any] | None:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None
    return data if isinstance(data, dict) else None


def rel(root: Path, path: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def skip_dir(path: Path) -> bool:
    return any(part in EXCLUDED_DIRS for part in path.parts)


def path_is_within_root(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
        return True
    except (OSError, ValueError):
        return False


def is_scannable_file(path: Path) -> bool:
    return (
        path.suffix in SOURCE_SUFFIXES
        or path.name == "package.json"
        or path.name.startswith(".env")
        or path.name in {
            "tailwind.config.js",
            "tailwind.config.ts",
            "tailwind.config.mjs",
            "tailwind.config.cjs",
            "next.config.js",
            "next.config.ts",
            "next.config.mjs",
            "vite.config.js",
            "vite.config.ts",
            "vite.config.mjs",
        }
    )


def iter_repo_files(root: Path, *, max_files: int, max_dirs: int) -> list[Path]:
    files: list[Path] = []
    dirs_seen = 0
    root = root.resolve()
    for current_root, dirnames, filenames in os.walk(root, followlinks=False):
        dirs_seen += 1
        if dirs_seen > max_dirs:
            break
        current = Path(current_root)
        dirnames[:] = sorted(
            [
                name
                for name in dirnames
                if name not in EXCLUDED_DIRS
                and not (current / name).is_symlink()
                and path_is_within_root(current / name, root)
            ]
        )
        for filename in sorted(filenames):
            if len(files) >= max_files:
                break
            path = current / filename
            if path.is_symlink() or not path_is_within_root(path, root) or not is_scannable_file(path):
                continue
            files.append(path)
        if len(files) >= max_files:
            break
    return files


def read_source(path: Path, max_bytes: int) -> str | None:
    try:
        if path.is_symlink():
            return None
        if path.stat().st_size > max_bytes:
            return None
        return path.read_text(encoding="utf-8")
    except Exception:
        return None


def line_for_offset(text: str, offset: int) -> int:
    return text.count("\n", 0, offset) + 1


def major_from_spec(spec: str) -> int | None:
    value = spec.strip()
    for prefix in ("workspace:", "catalog:", "npm:"):
        if value.startswith(prefix):
            value = value.split(":", 1)[1].strip()
    if value.startswith(("file:", "link:", "git+", "github:")) or value in {"*", "latest", "next"}:
        return None
    match = re.search(r"(\d+)(?:\.\d+)?(?:\.\d+)?", value)
    return int(match.group(1)) if match else None


def sanitize_text(value: str) -> str:
    value = re.sub(r"(?i)(://[^/\s:@]+:)[^@\s/]+@", rf"\1{REDACTED}@", value)
    value = re.sub(r"(?i)(token|key|secret|password|passwd)=([^&\s]+)", rf"\1={REDACTED}", value)
    value = JWT_RE.sub(REDACTED, value)
    return value[:240]


def public_spec(spec: str) -> str:
    return sanitize_text(spec)


def collect_manifests(
    root: Path,
    *,
    max_files: int,
    max_dirs: int,
    max_manifests: int,
) -> tuple[list[dict[str, Any]], dict[str, list[dict[str, str]]]]:
    manifests: list[dict[str, Any]] = []
    packages: dict[str, list[dict[str, str]]] = {}
    candidates = [path for path in iter_repo_files(root, max_files=max_files, max_dirs=max_dirs) if path.name == "package.json"]
    for path in candidates[:max_manifests]:
        if skip_dir(path) or path.is_symlink() or not path_is_within_root(path, root):
            continue
        data = read_json(path)
        if not data:
            continue
        manifest = {"path": rel(root, path), "packages": {}}
        for section in MANIFEST_SECTIONS:
            deps = data.get(section)
            if not isinstance(deps, dict):
                continue
            for name, spec in deps.items():
                if not isinstance(name, str) or not isinstance(spec, str):
                    continue
                safe_spec = public_spec(spec)
                manifest["packages"][name] = {"section": section, "spec": safe_spec}
                packages.setdefault(name, []).append({"path": rel(root, path), "section": section, "spec": safe_spec})
        manifests.append(manifest)
    manifests.sort(key=lambda item: item["path"])
    return manifests, packages


def has_package(packages: dict[str, list[dict[str, str]]], name: str) -> bool:
    return name in packages


def package_rows(packages: dict[str, list[dict[str, str]]], families: set[str]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for name, entries in sorted(packages.items()):
        matching_families = set(PACKAGE_FAMILIES.get(name, set()))
        if name.startswith("@ai-sdk/"):
            matching_families.add("ai-sdk-core")
        if not matching_families.intersection(families):
            continue
        for entry in entries:
            rows.append(
                {
                    "name": name,
                    "spec": entry["spec"],
                    "major": major_from_spec(entry["spec"]),
                    "manifest": entry["path"],
                    "section": entry["section"],
                    "families": sorted(matching_families),
                }
            )
    return rows


def add_signal(
    signals: list[dict[str, Any]],
    *,
    family: str,
    signal_id: str,
    severity: str,
    path: str,
    message: str,
    line: int | None = None,
    evidence: str | None = None,
    docs: str | None = None,
) -> None:
    signal = {
        "id": signal_id,
        "family": family,
        "severity": severity,
        "path": path,
        "message": message,
    }
    if line is not None:
        signal["line"] = line
    if evidence:
        signal["evidence"] = sanitize_text(evidence)
    if docs:
        signal["docs"] = docs
    signals.append(signal)


def add_regex_signals(
    signals: list[dict[str, Any]],
    *,
    family: str,
    signal_id: str,
    severity: str,
    path: str,
    text: str,
    pattern: str,
    message: str,
    docs: str,
) -> None:
    for match in re.finditer(pattern, text):
        add_signal(
            signals,
            family=family,
            signal_id=signal_id,
            severity=severity,
            path=path,
            line=line_for_offset(text, match.start()),
            evidence=match.group(0),
            message=message,
            docs=docs,
        )


def scan_ai_sdk(text: str, path: str, signals: list[dict[str, Any]], families: set[str]) -> None:
    if "ai-sdk-core" in families:
        add_regex_signals(
            signals,
            family="ai-sdk-core",
            signal_id="ai_sdk_max_steps_legacy",
            severity="warning",
            path=path,
            text=text,
            pattern=r"\bmaxSteps\s*:",
            message="AI SDK multi-step calls should use stopWhen with stepCountIs/hasToolCall instead of legacy maxSteps.",
            docs=DOCS["ai-sdk-core"]["migration"],
        )
        add_regex_signals(
            signals,
            family="ai-sdk-core",
            signal_id="ai_sdk_removed_stream_response_helper",
            severity="warning",
            path=path,
            text=text,
            pattern=r"\b(StreamingTextResponse|streamToResponse)\b",
            message="Legacy stream response helpers should be replaced with result.toUIMessageStreamResponse() or current stream helpers.",
            docs=DOCS["ai-sdk-ui"]["migration"],
        )
        add_regex_signals(
            signals,
            family="ai-sdk-core",
            signal_id="ai_sdk_tool_without_input_schema",
            severity="warning",
            path=path,
            text=text,
            pattern=r"tool\s*\(\s*\{(?![^}]{0,900}\binputSchema\s*:)",
            message="tool() definitions should provide inputSchema for typed tool inputs.",
            docs=DOCS["ai-sdk-core"]["primary"],
        )
        if "createMCPClient" in text and ".close(" not in text:
            add_signal(
                signals,
                family="ai-sdk-core",
                signal_id="ai_sdk_mcp_client_without_close",
                severity="warning",
                path=path,
                message="createMCPClient() appears without a close call in this file; verify lifecycle cleanup.",
                docs=DOCS["ai-sdk-core"]["primary"],
            )

    if "ai-sdk-ui" in families and ("useChat(" in text or "@ai-sdk/react" in text):
        add_regex_signals(
            signals,
            family="ai-sdk-ui",
            signal_id="ai_sdk_ui_message_content",
            severity="warning",
            path=path,
            text=text,
            pattern=r"\bmessage\.content\b|\bmessages\[[^\]]+\]\.content\b",
            message="Current UIMessage rendering should use message.parts instead of message.content.",
            docs=DOCS["ai-sdk-ui"]["migration"],
        )
        add_regex_signals(
            signals,
            family="ai-sdk-ui",
            signal_id="ai_sdk_ui_legacy_use_chat_helpers",
            severity="warning",
            path=path,
            text=text,
            pattern=r"const\s*\{[^}]*\b(input|handleInputChange|handleSubmit)\b[^}]*\}\s*=\s*useChat\s*\(",
            message="Current useChat examples favor sendMessage with DefaultChatTransport; verify legacy hook helper usage before migration.",
            docs=DOCS["ai-sdk-ui"]["reference"],
        )

    if "ai-sdk-agents" in families and ("ToolLoopAgent" in text or "stopWhen" in text):
        add_regex_signals(
            signals,
            family="ai-sdk-agents",
            signal_id="ai_sdk_agent_unbounded_loop",
            severity="warning",
            path=path,
            text=text,
            pattern=r"new\s+ToolLoopAgent\s*\(\s*\{(?![\s\S]{0,1600}\bstopWhen\s*:)",
            message="ToolLoopAgent should set explicit stopWhen when the task can run tools repeatedly.",
            docs=DOCS["ai-sdk-agents"]["reference"],
        )


def scan_streamdown(text: str, path: str, signals: list[dict[str, Any]], families: set[str]) -> None:
    if "streamdown" not in families:
        return
    if "react-markdown" in text and ("useChat(" in text or "streamText(" in text or "@ai-sdk/react" in text):
        add_signal(
            signals,
            family="streamdown",
            signal_id="streamdown_react_markdown_in_streaming_chat",
            severity="info",
            path=path,
            message="AI streaming markdown can usually use Streamdown instead of react-markdown.",
            docs=DOCS["streamdown"]["migration"],
        )
    if "Streamdown" in text and "isAnimating" not in text:
        add_signal(
            signals,
            family="streamdown",
            signal_id="streamdown_missing_is_animating",
            severity="warning",
            path=path,
            message="Streamdown in chat UIs should wire isAnimating from streaming status when content streams.",
            docs=DOCS["streamdown"]["primary"],
        )


def scan_zod(text: str, path: str, signals: list[dict[str, Any]], families: set[str]) -> None:
    if "zod-v4" not in families or ("zod" not in text and re.search(r"\bz\.", text) is None):
        return
    add_regex_signals(
        signals,
        family="zod-v4",
        signal_id="zod_v4_deprecated_string_format_method",
        severity="info",
        path=path,
        text=text,
        pattern=r"\bz\.string\(\)\.(email|uuid|url|emoji|base64|base64url|nanoid|cuid|cuid2|ulid|ip|ipv4|ipv6|datetime|date|time)\s*\(",
        message="Zod v4 prefers top-level string format helpers such as z.email(), z.uuid(), z.url(), and z.iso.datetime().",
        docs=DOCS["zod-v4"]["migration"],
    )
    add_regex_signals(
        signals,
        family="zod-v4",
        signal_id="zod_v4_legacy_error_params",
        severity="warning",
        path=path,
        text=text,
        pattern=r"\b(required_error|invalid_type_error)\s*:",
        message="Zod v4 removed required_error/invalid_type_error; use the unified error parameter.",
        docs=DOCS["zod-v4"]["migration"],
    )
    add_regex_signals(
        signals,
        family="zod-v4",
        signal_id="zod_v4_message_param",
        severity="info",
        path=path,
        text=text,
        pattern=r"\{\s*message\s*:\s*['\"]",
        message="Zod v4 prefers { error: ... } over { message: ... } for schema error customization.",
        docs=DOCS["zod-v4"]["migration"],
    )
    add_regex_signals(
        signals,
        family="zod-v4",
        signal_id="zod_v4_native_enum",
        severity="info",
        path=path,
        text=text,
        pattern=r"\bz\.nativeEnum\s*\(",
        message="Zod v4 supports enum-like inputs through z.enum(); verify nativeEnum usage during migration.",
        docs=DOCS["zod-v4"]["migration"],
    )
    add_regex_signals(
        signals,
        family="zod-v4",
        signal_id="zod_v4_error_errors_property",
        severity="warning",
        path=path,
        text=text,
        pattern=r"\berror\.errors\b",
        message="Prefer ZodError.issues in current Zod error handling.",
        docs=DOCS["zod-v4"]["reference"],
    )


def looks_client_path(path: str, text: str) -> bool:
    lowered = path.lower()
    name = Path(path).name.lower()
    if ".server." in name or "/server/" in lowered or "/route." in lowered or "/actions/" in lowered:
        return False
    return (
        "'use client'" in text[:300]
        or '"use client"' in text[:300]
        or ".client." in name
        or "/client" in lowered
        or "browser" in lowered
    )


def contains_service_role_jwt(text: str) -> bool:
    for match in JWT_RE.finditer(text):
        token = match.group(0)
        parts = token.split(".")
        if len(parts) != 3:
            continue
        payload = parts[1] + "=" * (-len(parts[1]) % 4)
        try:
            decoded = base64.urlsafe_b64decode(payload.encode("ascii"))
        except Exception:
            continue
        if b'"role":"service_role"' in decoded or b'"role": "service_role"' in decoded:
            return True
    return False


def scan_supabase(text: str, path: str, signals: list[dict[str, Any]], families: set[str]) -> None:
    if "supabase-ts" not in families:
        return
    if "@supabase/auth-helpers" in text:
        add_signal(
            signals,
            family="supabase-ts",
            signal_id="supabase_legacy_auth_helpers",
            severity="warning",
            path=path,
            message="Supabase auth helpers should be migrated to @supabase/ssr for current SSR auth.",
            docs=DOCS["supabase-ts"]["primary"],
        )
    if ".auth.getSession(" in text and re.search(r"\b(middleware|route|server|loader|action)\b", path):
        add_signal(
            signals,
            family="supabase-ts",
            signal_id="supabase_server_get_session_authz",
            severity="warning",
            path=path,
            message="Server authorization should use auth.getUser() because getSession() reads cookie state without server verification.",
            docs=DOCS["supabase-ts"]["primary"],
        )
    if PUBLIC_ENV_RE.search(text):
        add_signal(
            signals,
            family="supabase-ts",
            signal_id="supabase_service_role_public_env",
            severity="error",
            path=path,
            message="Public environment variable prefixes must not be used for Supabase service-role keys.",
            docs=DOCS["supabase-ts"]["primary"],
        )
    if contains_service_role_jwt(text):
        add_signal(
            signals,
            family="supabase-ts",
            signal_id="supabase_service_role_jwt_literal",
            severity="error",
            path=path,
            message="A Supabase service-role JWT literal appears in source; remove it and rotate the credential.",
            docs=DOCS["supabase-ts"]["primary"],
        )
    if SERVICE_ROLE_NAME_RE.search(text) and looks_client_path(path, text):
        add_signal(
            signals,
            family="supabase-ts",
            signal_id="supabase_service_role_client_exposure",
            severity="error",
            path=path,
            message="Service role keys must stay server-only and should never appear in client/browser code.",
            docs=DOCS["supabase-ts"]["primary"],
        )
    if path.endswith(".sql"):
        add_regex_signals(
            signals,
            family="supabase-ts",
            signal_id="supabase_rls_direct_auth_uid",
            severity="info",
            path=path,
            text=text,
            pattern=r"(?<!select\s)auth\.uid\(\)",
            message="RLS policies should usually wrap auth.uid() in (select auth.uid()) to avoid repeated calls.",
            docs=DOCS["supabase-ts"]["primary"],
        )


def add_package_signals(
    signals: list[dict[str, Any]],
    packages: dict[str, list[dict[str, str]]],
    package_rows_value: list[dict[str, Any]],
    families: set[str],
) -> None:
    ai_major = next((row["major"] for row in package_rows_value if row["name"] == "ai" and row["major"]), None)
    for row in package_rows_value:
        if row["name"] == "ai" and row["major"] is not None and row["major"] < 5:
            add_signal(
                signals,
                family="ai-sdk-core",
                signal_id="ai_sdk_old_major",
                severity="warning",
                path=row["manifest"],
                message="AI SDK package major is older than current skill coverage; verify migration before using v5/v6 patterns.",
                docs=DOCS["ai-sdk-core"]["migration"],
                evidence=f"{row['name']}@{row['spec']}",
            )
        if row["name"].startswith("@ai-sdk/") and ai_major and row["major"] and row["major"] != ai_major:
            add_signal(
                signals,
                family="ai-sdk-core",
                signal_id="ai_sdk_provider_major_mismatch",
                severity="warning",
                path=row["manifest"],
                message="AI SDK provider package major does not match the ai package major.",
                docs=DOCS["ai-sdk-core"]["primary"],
                evidence=f"ai major {ai_major}; {row['name']}@{row['spec']}",
            )
        if row["name"] == "zod" and row["major"] is not None and row["major"] < 4:
            add_signal(
                signals,
                family="zod-v4",
                signal_id="zod_pre_v4_dependency",
                severity="warning",
                path=row["manifest"],
                message="This skill targets Zod v4; migrate package.json before applying v4-only APIs.",
                docs=DOCS["zod-v4"]["migration"],
                evidence=f"zod@{row['spec']}",
            )

    if "streamdown" in families and has_package(packages, "react-markdown") and not has_package(packages, "streamdown"):
        for entry in packages["react-markdown"]:
            add_signal(
                signals,
                family="streamdown",
                signal_id="streamdown_missing_dependency",
                severity="info",
                path=entry["path"],
                message="Project depends on react-markdown but not streamdown; consider Streamdown for AI streaming markdown.",
                docs=DOCS["streamdown"]["migration"],
                evidence=f"react-markdown@{entry['spec']}",
            )

    if "supabase-ts" in families:
        for name, entries in packages.items():
            if not name.startswith("@supabase/auth-helpers"):
                continue
            for entry in entries:
                add_signal(
                    signals,
                    family="supabase-ts",
                    signal_id="supabase_legacy_auth_helpers_dependency",
                    severity="warning",
                    path=entry["path"],
                    message="Legacy @supabase/auth-helpers packages should be replaced with @supabase/ssr.",
                    docs=DOCS["supabase-ts"]["primary"],
                    evidence=f"{name}@{entry['spec']}",
                )


def add_cross_file_signals(
    signals: list[dict[str, Any]],
    root: Path,
    files: dict[str, str],
    packages: dict[str, list[dict[str, str]]],
    families: set[str],
) -> None:
    all_text = "\n".join(files.values())
    if "streamdown" in families and has_package(packages, "streamdown") and "streamdown/dist" not in all_text:
        first_manifest = packages["streamdown"][0]["path"]
        add_signal(
            signals,
            family="streamdown",
            signal_id="streamdown_tailwind_source_missing",
            severity="info",
            path=first_manifest,
            message="streamdown is installed but no Tailwind source/content entry for streamdown/dist was found.",
            docs=DOCS["streamdown"]["migration"],
        )

    if "ai-sdk-ui" in families and "@ai-sdk/react" in all_text and not has_package(packages, "@ai-sdk/react"):
        add_signal(
            signals,
            family="ai-sdk-ui",
            signal_id="ai_sdk_ui_missing_react_package",
            severity="warning",
            path=rel(root, root),
            message="Source imports @ai-sdk/react but no package.json dependency was found.",
            docs=DOCS["ai-sdk-ui"]["primary"],
        )

    if (
        "streamdown" in families
        and ("from 'streamdown'" in all_text or 'from "streamdown"' in all_text)
        and not has_package(packages, "streamdown")
    ):
        add_signal(
            signals,
            family="streamdown",
            signal_id="streamdown_import_missing_dependency",
            severity="warning",
            path=rel(root, root),
            message="Source imports streamdown but no package.json dependency was found.",
            docs=DOCS["streamdown"]["primary"],
        )


def scan_sources(
    root: Path,
    families: set[str],
    max_files: int,
    max_dirs: int,
    max_bytes: int,
) -> tuple[list[dict[str, Any]], dict[str, str]]:
    signals: list[dict[str, Any]] = []
    files: dict[str, str] = {}
    for path in iter_repo_files(root, max_files=max_files, max_dirs=max_dirs):
        text = read_source(path, max_bytes)
        if text is None:
            continue
        relative = rel(root, path)
        files[relative] = text
        scan_ai_sdk(text, relative, signals, families)
        scan_streamdown(text, relative, signals, families)
        scan_zod(text, relative, signals, families)
        scan_supabase(text, relative, signals, families)
    return signals, files


def summarize(signals: list[dict[str, Any]], packages: list[dict[str, Any]], families: set[str]) -> dict[str, Any]:
    by_severity = Counter(signal["severity"] for signal in signals)
    by_family = Counter(signal["family"] for signal in signals)
    present_families = sorted({family for row in packages for family in row["families"] if family in families})
    return {
        "signal_count": len(signals),
        "by_severity": dict(sorted(by_severity.items())),
        "by_family": dict(sorted(by_family.items())),
        "present_families": present_families,
    }


def main() -> int:
    args = parse_args()
    root = Path(args.root).resolve()
    families = selected_families(args.family)
    manifests, packages_by_name = collect_manifests(
        root,
        max_files=args.max_files,
        max_dirs=args.max_dirs,
        max_manifests=args.max_manifests,
    )
    packages = package_rows(packages_by_name, families)
    source_signals, files = scan_sources(root, families, args.max_files, args.max_dirs, args.max_bytes)
    signals: list[dict[str, Any]] = []
    add_package_signals(signals, packages_by_name, packages, families)
    signals.extend(source_signals)
    add_cross_file_signals(signals, root, files, packages_by_name, families)
    signals.sort(key=lambda item: (item["severity"], item["family"], item["path"], item.get("line", 0), item["id"]))
    if not args.include_evidence:
        for signal in signals:
            signal.pop("evidence", None)

    output = {
        "schema": SCHEMA,
        "scanner_version": SCANNER_VERSION,
        "root": str(root) if args.include_absolute_root else "<scan-root>",
        "families": sorted(families),
        "privacy": {
            "external_processing": "repo-confidential; do not paste full output into external services",
            "absolute_root_included": bool(args.include_absolute_root),
            "evidence_included": bool(args.include_evidence),
        },
        "docs": {family: DOCS[family] for family in sorted(families)},
        "package_manifests": manifests,
        "packages": packages,
        "signals": signals,
        "summary": summarize(signals, packages, families),
    }
    json.dump(output, sys.stdout, indent=2 if args.pretty else None, sort_keys=True)
    sys.stdout.write("\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
