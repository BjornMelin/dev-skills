#!/usr/bin/env python3
"""Run Kimi Code CLI as a bounded UI advisor and emit structured JSON."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


MIN_KIMI_VERSION = (1, 44, 0)
LATEST_OBSERVED_VERSION = (1, 45, 0)
SCRIPT_DIR = Path(__file__).resolve().parent
SKILL_DIR = SCRIPT_DIR.parent
DEFAULT_AGENT_FILE = SKILL_DIR / "assets" / "kimi-agent" / "agent.yaml"
MODES = ("advise", "audit", "redesign", "component", "screenshot-review", "compare")
MODE_INSTRUCTIONS = {
    "advise": (
        "Provide targeted frontend/UI implementation advice and concrete code suggestions "
        "for the requested surface."
    ),
    "audit": (
        "Audit the current UI. Rank issues by user impact and implementation leverage. "
        "Prefer specific evidence, quick wins, and verification checks over broad redesigns."
    ),
    "redesign": (
        "Propose a cohesive professional redesign direction, then give concrete, repo-shaped "
        "code changes that improve hierarchy, density, state coverage, and responsive behavior."
    ),
    "component": (
        "Focus on one component or component family. Cover API shape, variants, states, "
        "composition, styling hooks, accessibility, and test/Storybook-style coverage."
    ),
    "screenshot-review": (
        "Review the provided screenshot or visual reference images. Identify composition, "
        "hierarchy, spacing, typography, color, responsive, and accessibility issues, then map "
        "the strongest findings to concrete code changes."
    ),
    "compare": (
        "Compare before and after images. Identify improvements, regressions, remaining gaps, "
        "and whether the after state meets the requested quality bar."
    ),
}


def positive_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("must be a positive integer") from exc
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be a positive integer")
    return parsed


def non_negative_int(value: str) -> int:
    try:
        parsed = int(value)
    except ValueError as exc:
        raise argparse.ArgumentTypeError("must be a non-negative integer") from exc
    if parsed < 0:
        raise argparse.ArgumentTypeError("must be a non-negative integer")
    return parsed


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Ask Kimi Code CLI for frontend/UI suggestions without letting it edit files.",
    )
    parser.add_argument("--prompt", help="UI/frontend task prompt. Reads stdin when omitted.")
    parser.add_argument("--prompt-file", type=Path, help="Read the task prompt from a file.")
    parser.add_argument(
        "--mode",
        choices=MODES,
        default="advise",
        help="Advisory mode for Kimi's response.",
    )
    parser.add_argument(
        "--compare",
        action="store_true",
        help="Shortcut for --mode compare.",
    )
    parser.add_argument(
        "--design-brief",
        help="Structured product/design constraints to include in the Kimi prompt.",
    )
    parser.add_argument(
        "--design-brief-file",
        type=Path,
        help="Read structured product/design constraints from a file.",
    )
    parser.add_argument(
        "--quality-bar",
        default="advanced professional product UI",
        help="Short description of the target UI quality bar.",
    )
    parser.add_argument("--work-dir", type=Path, default=Path.cwd(), help="Repo/work directory.")
    parser.add_argument(
        "--file",
        action="append",
        default=[],
        dest="files",
        help="Relevant file path to point Kimi at. Repeatable.",
    )
    parser.add_argument(
        "--image",
        "--screenshot",
        action="append",
        default=[],
        dest="images",
        help="Screenshot/reference image path for screenshot-review mode. Repeatable.",
    )
    parser.add_argument(
        "--before-image",
        action="append",
        default=[],
        dest="before_images",
        help="Before screenshot path for compare mode. Repeatable.",
    )
    parser.add_argument(
        "--after-image",
        action="append",
        default=[],
        dest="after_images",
        help="After screenshot path for compare mode. Repeatable.",
    )
    parser.add_argument("--agent-file", type=Path, default=DEFAULT_AGENT_FILE)
    parser.add_argument("--kimi-bin", default="kimi", help="Kimi executable name or path.")
    parser.add_argument("--max-steps", type=positive_int, default=8)
    parser.add_argument("--timeout", type=positive_int, default=480, help="Timeout in seconds.")
    parser.add_argument("--retries", type=non_negative_int, default=1, help="Retries for exit code 75.")
    parser.add_argument(
        "--thinking",
        action="store_true",
        help="Enable Kimi thinking mode. Default is off for deterministic JSON output.",
    )
    parser.add_argument("--save", action="store_true", help="Save output under .codex/kimi/.")
    parser.add_argument("--out", type=Path, help="Specific JSON output path.")
    args = parser.parse_args()
    if args.compare:
        if args.mode != "advise":
            raise SystemExit("Use either --compare or --mode, not both.")
        args.mode = "compare"
    return args


def read_prompt(args: argparse.Namespace) -> str:
    if args.prompt and args.prompt_file:
        raise SystemExit("Use either --prompt or --prompt-file, not both.")
    if args.prompt_file:
        with args.prompt_file.open("r", encoding="utf-8") as handle:
            return handle.read().strip()
    if args.prompt:
        return args.prompt.strip()
    if not sys.stdin.isatty():
        return sys.stdin.read().strip()
    raise SystemExit("Provide --prompt, --prompt-file, or stdin.")


def read_design_brief(args: argparse.Namespace) -> str:
    if args.design_brief and args.design_brief_file:
        raise SystemExit("Use either --design-brief or --design-brief-file, not both.")
    if args.design_brief_file:
        with args.design_brief_file.open("r", encoding="utf-8") as handle:
            return handle.read().strip()
    return (args.design_brief or "").strip()


def parse_version(text: str) -> tuple[int, int, int] | None:
    match = re.search(r"(\d+)\.(\d+)\.(\d+)", text)
    if not match:
        return None
    return tuple(int(part) for part in match.groups())


def kimi_info(kimi_bin: str) -> dict[str, Any]:
    if shutil.which(kimi_bin) is None and not Path(kimi_bin).exists():
        return {"available": False, "error": f"{kimi_bin!r} not found on PATH"}

    info = subprocess.run(
        [kimi_bin, "info"],
        text=True,
        capture_output=True,
        timeout=15,
        check=False,
    )
    version = parse_version(info.stdout)
    return {
        "available": info.returncode == 0,
        "version": ".".join(str(v) for v in version) if version else None,
        "version_tuple": version,
        "stdout": info.stdout.strip(),
        "stderr": info.stderr.strip(),
        "returncode": info.returncode,
        "minimum": ".".join(str(v) for v in MIN_KIMI_VERSION),
        "meets_minimum": version is not None and version >= MIN_KIMI_VERSION,
    }


def normalize_paths(paths: list[str], work_dir: Path) -> list[str]:
    normalized: list[str] = []
    for item in paths:
        path = Path(item)
        if not path.is_absolute():
            path = work_dir / path
        try:
            normalized.append(str(path.resolve().relative_to(work_dir.resolve())))
        except ValueError:
            normalized.append(str(path.resolve()))
    return normalized


def validate_mode_inputs(
    mode: str,
    images: list[str],
    before_images: list[str],
    after_images: list[str],
    work_dir: Path,
) -> None:
    if mode == "screenshot-review" and not images:
        raise SystemExit("--mode screenshot-review requires at least one --image/--screenshot.")
    if mode == "compare" and (not before_images or not after_images):
        raise SystemExit("--mode compare requires at least one --before-image and one --after-image.")
    if mode != "compare" and (before_images or after_images):
        raise SystemExit("--before-image and --after-image are only valid with --mode compare.")
    for label, paths in (
        ("--image/--screenshot", images),
        ("--before-image", before_images),
        ("--after-image", after_images),
    ):
        for item in paths:
            path = Path(item)
            if not path.is_absolute():
                path = work_dir / path
            if not path.exists():
                raise SystemExit(f"{label} path does not exist: {item}")


def format_list(items: list[str]) -> str:
    return "\n".join(f"- {path}" for path in items) if items else "- none provided"


def build_kimi_prompt(
    user_prompt: str,
    files: list[str],
    work_dir: Path,
    mode: str,
    design_brief: str,
    images: list[str],
    before_images: list[str],
    after_images: list[str],
    quality_bar: str,
) -> str:
    design_brief_block = design_brief if design_brief else "- none provided"
    return f"""You are being called by Codex through the explicit $kimi-ui-advisor skill.

Goal: produce high-quality frontend/UI implementation guidance and code suggestions.

Mode: {mode}
Mode instructions: {MODE_INSTRUCTIONS[mode]}
Target quality bar: {quality_bar}
Working directory: {work_dir}

Relevant files:
{format_list(files)}

Screenshot/reference images:
{format_list(images)}

Before images for comparison:
{format_list(before_images)}

After images for comparison:
{format_list(after_images)}

Design brief:
{design_brief_block}

User request:
{user_prompt}

Operating rules:
- If the request is generic or no relevant files are provided, answer directly
  instead of exploring the repository.
- If file context is provided, inspect only the smallest useful set of files.
- Inspect relevant project files with read-only tools as needed.
- If image paths are provided, inspect them with ReadMediaFile before final advice.
- Use web/fetch only for public framework docs, UI library docs, accessibility references, or design inspiration.
- Do not put proprietary source snippets, secrets, private tokens, or private requirements in web searches.
- Do not ask the user questions; make conservative assumptions and list them under risks.
- Do not claim you changed files. Codex will decide what to apply.
- Finish with the JSON object as soon as you have enough context. Do not keep
  searching for optional improvements.

Return exactly one JSON object and no Markdown fence. Use this schema:
{{
  "mode": "{mode}",
  "summary": "one or two sentences",
  "approach": "implementation approach",
  "ranked_issues": [
    {{
      "severity": "high|medium|low",
      "area": "layout|hierarchy|component|styling|responsive|accessibility|motion|copy",
      "evidence": "specific observation from files or images",
      "recommendation": "what Codex should do"
    }}
  ],
  "design_direction": {{
    "principles": ["short principles for the UI direction"],
    "tokens": ["spacing, color, type, radius, shadow, or density guidance"],
    "interaction_model": "how interaction states and feedback should behave"
  }},
  "files": [
    {{"path": "relative/path", "reason": "why this file matters"}}
  ],
  "image_findings": [
    {{"path": "relative/or/absolute/image", "finding": "visual finding", "recommendation": "actionable fix"}}
  ],
  "patch_suggestions": [
    {{
      "path": "relative/path",
      "intent": "what to change",
      "content": "code, diff, or precise replacement guidance"
    }}
  ],
  "component_notes": ["component structure, states, props, composition"],
  "styling_notes": ["layout, spacing, typography, responsive, theme notes"],
  "accessibility_notes": ["keyboard, semantics, focus, contrast, labels"],
  "responsive_notes": ["breakpoints, wrapping, density, touch target notes"],
  "motion_notes": ["microinteraction, transition, reduced-motion notes"],
  "verification": ["commands or visual checks Codex should run"],
  "acceptance_criteria": ["observable criteria for the improved UI"],
  "risks": ["assumptions, uncertainty, or rejected alternatives"],
  "rejected_suggestions": ["ideas intentionally not recommended and why"]
}}

Use empty arrays when a section has no items."""


def run_kimi(args: argparse.Namespace, prompt: str) -> tuple[int, str, str, int]:
    command = [
        args.kimi_bin,
        "--print",
        "--output-format=stream-json",
        "--final-message-only",
        "--agent-file",
        str(args.agent_file),
        "--work-dir",
        str(args.work_dir),
        "--thinking" if args.thinking else "--no-thinking",
        "--max-steps-per-turn",
        str(args.max_steps),
        "-p",
        prompt,
    ]

    attempts = max(1, args.retries + 1)
    last: subprocess.CompletedProcess[str] | None = None
    for attempt in range(1, attempts + 1):
        last = subprocess.run(
            command,
            text=True,
            capture_output=True,
            timeout=args.timeout,
            check=False,
        )
        if last.returncode != 75 or attempt == attempts:
            return last.returncode, last.stdout, last.stderr, attempt
        time.sleep(min(10, 2 * attempt))

    return 1, "", "Kimi did not run because retry attempts could not be initialized.", attempts


def parse_kimi_stdout(stdout: str) -> tuple[str, list[dict[str, Any]], list[str]]:
    messages: list[dict[str, Any]] = []
    non_json: list[str] = []
    content_parts: list[str] = []

    for line in stdout.splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        try:
            message = json.loads(stripped)
        except json.JSONDecodeError:
            non_json.append(stripped)
            continue
        if isinstance(message, dict):
            messages.append(message)
            if message.get("role") == "assistant" and isinstance(message.get("content"), str):
                content_parts.append(message["content"])

    return "\n".join(content_parts).strip(), messages, non_json


def extract_json_object(text: str) -> tuple[dict[str, Any] | None, str | None]:
    candidate = text.strip()
    if candidate.startswith("```"):
        candidate = re.sub(r"^```(?:json)?\s*", "", candidate)
        candidate = re.sub(r"\s*```$", "", candidate).strip()
    try:
        parsed = json.loads(candidate)
        if isinstance(parsed, dict):
            return parsed, None
    except json.JSONDecodeError as exc:
        error = str(exc)

    start = candidate.find("{")
    end = candidate.rfind("}")
    if start != -1 and end != -1 and end > start:
        try:
            parsed = json.loads(candidate[start : end + 1])
            if isinstance(parsed, dict):
                return parsed, None
        except json.JSONDecodeError as exc:
            error = str(exc)
    return None, error if "error" in locals() else "No JSON object found"


def write_output(payload: dict[str, Any], args: argparse.Namespace) -> None:
    output_path = args.out
    if args.save and output_path is None:
        stamp = dt.datetime.now(dt.UTC).strftime("%Y%m%d-%H%M%S")
        output_path = args.work_dir / ".codex" / "kimi" / f"kimi-ui-advisor-{stamp}.json"
    if output_path is not None:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        payload.setdefault("metadata", {})["saved_to"] = str(output_path)
        with output_path.open("w", encoding="utf-8") as handle:
            handle.write(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def main() -> int:
    args = parse_args()
    args.work_dir = args.work_dir.resolve()
    args.agent_file = args.agent_file.resolve()
    user_prompt = read_prompt(args)
    design_brief = read_design_brief(args)
    files = normalize_paths(args.files, args.work_dir)
    images = normalize_paths(args.images, args.work_dir)
    before_images = normalize_paths(args.before_images, args.work_dir)
    after_images = normalize_paths(args.after_images, args.work_dir)
    validate_mode_inputs(args.mode, images, before_images, after_images, args.work_dir)
    info = kimi_info(args.kimi_bin)

    prompt = build_kimi_prompt(
        user_prompt,
        files,
        args.work_dir,
        args.mode,
        design_brief,
        images,
        before_images,
        after_images,
        args.quality_bar,
    )
    payload: dict[str, Any] = {
        "ok": False,
        "metadata": {
            "skill": "kimi-ui-advisor",
            "mode": args.mode,
            "work_dir": str(args.work_dir),
            "files": files,
            "images": images,
            "before_images": before_images,
            "after_images": after_images,
            "design_brief_present": bool(design_brief),
            "quality_bar": args.quality_bar,
            "kimi": {k: v for k, v in info.items() if k != "version_tuple"},
        },
    }

    if not info.get("available"):
        payload["error"] = info.get("error") or "kimi info failed"
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 1
    if not info.get("meets_minimum"):
        payload["metadata"]["warning"] = (
            f"Kimi CLI {info.get('version') or 'unknown'} is below tested minimum "
            f"{info.get('minimum')}."
        )
    version_tuple = info.get("version_tuple")
    if version_tuple and version_tuple < LATEST_OBSERVED_VERSION:
        observed = ".".join(str(v) for v in LATEST_OBSERVED_VERSION)
        payload["metadata"]["latest_observed_warning"] = (
            f"Installed Kimi CLI {info.get('version')} is older than the latest "
            f"source/docs version observed during skill authoring ({observed})."
        )

    try:
        returncode, stdout, stderr, attempts = run_kimi(args, prompt)
    except subprocess.TimeoutExpired as exc:
        payload["error"] = f"Kimi timed out after {args.timeout}s"
        payload["stdout"] = exc.stdout or ""
        payload["stderr"] = exc.stderr or ""
        write_output(payload, args)
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 75

    content, messages, non_json = parse_kimi_stdout(stdout)
    result, parse_error = extract_json_object(content)
    payload["metadata"]["attempts"] = attempts
    payload["metadata"]["kimi_returncode"] = returncode
    payload["metadata"]["non_json_stdout"] = non_json
    payload["messages"] = messages
    payload["raw_response"] = content
    payload["stderr"] = stderr.strip()

    if returncode == 0 and result is not None:
        payload["ok"] = True
        payload["result"] = result
    else:
        if parse_error:
            payload["parse_error"] = parse_error
        payload["error"] = (
            f"Kimi exited with code {returncode}"
            if returncode != 0
            else parse_error or "Kimi response did not match the JSON contract"
        )

    write_output(payload, args)
    print(json.dumps(payload, indent=2, sort_keys=True))
    if returncode != 0:
        return returncode
    return 0 if payload["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
