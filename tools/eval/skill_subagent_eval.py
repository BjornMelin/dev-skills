#!/usr/bin/env python3
"""Offline eval lab for skill catalog and subagent contracts."""

from __future__ import annotations

import argparse
import json
import os
import py_compile
import re
import shutil
import subprocess
import sys
import tempfile
import time
import zipfile
from collections.abc import Callable
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path, PurePosixPath
from typing import Any
from urllib.parse import unquote

import yaml


SCHEMA = "skill_eval_report.v1"
TAIL_CHARS = 4000
CHECK_TIMEOUT_SECONDS = 120
GIT_LS_FILES_TIMEOUT_SECONDS = 30
SCRIPT_CHECK_TIMEOUT_SECONDS = 20

GENERATED_DIR_NAMES = {
    "__pycache__",
    ".codex",
    ".mypy_cache",
    ".pytest_cache",
    ".ruff_cache",
    ".venv",
    "node_modules",
    "target",
}
GENERATED_FILE_NAMES = {".DS_Store"}
GENERATED_SUFFIXES = {".pyc", ".pyo"}


@dataclass(frozen=True)
class SkillRecord:
    """Repository skill directory discovered from `skills/*/SKILL.md`."""

    name: str
    path: Path
    skill_md: Path
    frontmatter: dict[str, Any]


@dataclass(frozen=True)
class EvalCheck:
    """Definition for one offline eval check.

    Attributes:
        id: Stable machine-readable check identifier.
        name: Human-readable check label.
        command: Repo-relative command argv for subprocess-backed checks.
        runner: Native runner key for Python-backed aggregate checks.
        severity: Required checks fail the report on errors. Advisory checks are
            still surfaced but do not fail unless strict mode is enabled.
    """

    id: str
    name: str
    command: tuple[str, ...] | None = None
    runner: str | None = None
    severity: str = "required"

    def to_list_item(self) -> dict[str, Any]:
        """Render this check for the list-mode JSON contract."""
        item: dict[str, Any] = {
            "id": self.id,
            "name": self.name,
            "type": "command" if self.command else "native",
            "severity": self.severity,
        }
        if self.command:
            item["command"] = list(self.command)
        else:
            item["runner"] = self.runner
        return item


NativeRunner = Callable[[Path], tuple[list[dict[str, Any]], dict[str, Any]]]


def repo_root() -> Path:
    """Resolve the repository root from this script location."""
    return Path(__file__).resolve().parents[2]


def portable_repo_root() -> str:
    """Return the portable repository-root marker used in reports."""
    return "$REPO"


def default_checks() -> list[EvalCheck]:
    """Build the default offline eval check set."""
    return [
        EvalCheck(
            id="all-skill-frontmatter",
            name="All skill frontmatter validates through quick_validate",
            runner="all_skill_frontmatter",
        ),
        EvalCheck(
            id="readme-catalog-exposure",
            name="README catalog exposes every skill",
            runner="readme_catalog_exposure",
        ),
        EvalCheck(
            id="docs-reference-exposure",
            name="Expected reference docs are linked from docs/index.md",
            runner="docs_reference_exposure",
        ),
        EvalCheck(
            id="skill-local-links",
            name="Tracked skill Markdown local links resolve",
            runner="skill_local_links",
        ),
        EvalCheck(
            id="skill-script-syntax",
            name="Tracked skill helper scripts have parseable syntax",
            runner="skill_script_syntax",
        ),
        EvalCheck(
            id="generated-cache-exclusion",
            name="Tracked skill files exclude generated caches",
            runner="generated_cache_exclusion",
        ),
        EvalCheck(
            id="dist-package-metadata",
            name="Local .skill bundles have valid redistributable contents",
            runner="dist_package_metadata",
        ),
        EvalCheck(
            id="openai-agent-metadata",
            name="Skill agents/openai.yaml metadata validates",
            runner="openai_agent_metadata",
        ),
        EvalCheck(
            id="subagent-template-contracts",
            name="Subagent templates validate through subagent-creator",
            command=(
                "python3",
                "skills/subagent-creator/scripts/subagent_creator.py",
                "validate",
                "--json",
                "skills/deep-researcher/templates/agents",
                "skills/subagent-creator/templates/agents",
                "skills/subspawn/templates/agents",
                "subagents/codex/agents",
            ),
        ),
        EvalCheck(
            id="subspawn-role-contracts",
            name="Subspawn role contracts validate",
            command=(
                "python3",
                "skills/subspawn/scripts/subspawn_plan.py",
                "validate-roles",
                "--json",
            ),
        ),
        EvalCheck(
            id="subspawn-research-plan",
            name="Subspawn research preset plans deterministically",
            command=(
                "python3",
                "skills/subspawn/scripts/subspawn_plan.py",
                "plan",
                "--preset",
                "research",
                "--task",
                "validation smoke",
                "--scope",
                "docs and template metadata",
                "--json",
            ),
        ),
        EvalCheck(
            id="python-helper-compile",
            name="Skill, tool, and Codex subagent Python helpers compile",
            command=(
                "python3",
                "-m",
                "compileall",
                "-q",
                "skills",
                "tools",
                "subagents/codex/scripts",
            ),
        ),
    ]


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    """Parse eval-lab command-line arguments."""
    parser = argparse.ArgumentParser(
        description=(
            "Run offline skill/subagent eval checks with JSON evidence output."
        )
    )
    parser.add_argument("--json", action="store_true", help="Emit JSON report")
    parser.add_argument(
        "--list",
        action="store_true",
        help="List checks without running them",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Treat warning findings as report failures",
    )
    parser.add_argument(
        "--check",
        action="append",
        default=[],
        help="Run one check id; can be repeated. Defaults to all checks.",
    )
    return parser.parse_args(argv)


def selected_checks(checks: list[EvalCheck], ids: list[str]) -> list[EvalCheck]:
    """Filter checks to requested ids while preserving request order."""
    if not ids:
        return checks
    by_id = {check.id: check for check in checks}
    missing = sorted(set(ids) - set(by_id))
    if missing:
        available = ", ".join(sorted(by_id))
        missing_ids = ", ".join(missing)
        message = f"unknown check id(s): {missing_ids}. Available: {available}"
        raise SystemExit(message)
    return [by_id[check_id] for check_id in ids]


def run_check(check: EvalCheck, root: Path, strict: bool) -> dict[str, Any]:
    """Run one eval check and return a bounded evidence record."""
    if check.command:
        return run_command_check(check, root)
    return run_native_check(check, root, strict)


def run_command_check(check: EvalCheck, root: Path) -> dict[str, Any]:
    """Run one subprocess-backed eval check."""
    if check.command is None:
        raise ValueError(f"check {check.id} does not define a command")
    started = time.monotonic()
    env = os.environ.copy()
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    with tempfile.TemporaryDirectory(
        prefix="dev-skills-eval-pycache-",
    ) as pycache_dir:
        env["PYTHONPYCACHEPREFIX"] = pycache_dir
        try:
            completed = subprocess.run(  # noqa: S603
                check.command,
                cwd=root,
                env=env,
                text=True,
                capture_output=True,
                check=False,
                timeout=CHECK_TIMEOUT_SECONDS,
            )
        except subprocess.TimeoutExpired as error:
            duration_ms = elapsed_ms(started)
            return {
                "id": check.id,
                "name": check.name,
                "type": "command",
                "severity": check.severity,
                "command": list(check.command),
                "status": "timed_out",
                "exit_code": None,
                "duration_ms": duration_ms,
                "timeout_seconds": CHECK_TIMEOUT_SECONDS,
                "findings": [],
                "stdout_tail": tail(output_text(error.stdout), root),
                "stderr_tail": tail(output_text(error.stderr), root),
            }
        except OSError as error:
            duration_ms = elapsed_ms(started)
            return {
                "id": check.id,
                "name": check.name,
                "type": "command",
                "severity": check.severity,
                "command": list(check.command),
                "status": "failed",
                "exit_code": None,
                "duration_ms": duration_ms,
                "timeout_seconds": CHECK_TIMEOUT_SECONDS,
                "findings": [
                    finding(
                        "error",
                        None,
                        f"failed to start command: {error}",
                    )
                ],
                "stdout_tail": "",
                "stderr_tail": tail(str(error), root),
            }
    duration_ms = elapsed_ms(started)
    return {
        "id": check.id,
        "name": check.name,
        "type": "command",
        "severity": check.severity,
        "command": list(check.command),
        "status": "passed" if completed.returncode == 0 else "failed",
        "exit_code": completed.returncode,
        "duration_ms": duration_ms,
        "timeout_seconds": CHECK_TIMEOUT_SECONDS,
        "findings": [],
        "stdout_tail": tail(completed.stdout, root),
        "stderr_tail": tail(completed.stderr, root),
    }


def run_native_check(
    check: EvalCheck, root: Path, strict: bool
) -> dict[str, Any]:
    """Run one Python-backed aggregate eval check."""
    if check.runner is None:
        raise ValueError(f"check {check.id} does not define a runner")
    runner = native_runners()[check.runner]
    started = time.monotonic()
    try:
        findings, details = runner(root)
    except Exception as error:  # noqa: BLE001
        findings = [
            finding(
                "error", None, f"runner raised {type(error).__name__}: {error}"
            )
        ]
        details = {}
    duration_ms = elapsed_ms(started)
    error_count = count_findings(findings, "error")
    warning_count = count_findings(findings, "warning")
    status = "passed"
    if error_count and (check.severity == "required" or strict):
        status = "failed"
    elif error_count:
        status = "warning"
    elif warning_count:
        status = "failed" if strict else "warning"
    return {
        "id": check.id,
        "name": check.name,
        "type": "native",
        "severity": check.severity,
        "runner": check.runner,
        "status": status,
        "exit_code": None,
        "duration_ms": duration_ms,
        "findings": sanitize_findings(findings, root),
        "details": details,
    }


def native_runners() -> dict[str, NativeRunner]:
    """Return native runner registry."""
    return {
        "all_skill_frontmatter": check_all_skill_frontmatter,
        "readme_catalog_exposure": check_readme_catalog_exposure,
        "docs_reference_exposure": check_docs_reference_exposure,
        "skill_local_links": check_skill_local_links,
        "skill_script_syntax": check_skill_script_syntax,
        "generated_cache_exclusion": check_generated_cache_exclusion,
        "dist_package_metadata": check_dist_package_metadata,
        "openai_agent_metadata": check_openai_agent_metadata,
    }


def check_all_skill_frontmatter(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Validate every skill through the canonical quick validator."""
    skill_count = 0
    findings: list[dict[str, Any]] = []
    validator = load_quick_validator(root)
    for skill in discover_skills(root):
        skill_count += 1
        valid, message = validator(skill.path)
        if not valid:
            findings.append(finding("error", skill.skill_md, message))
    if skill_count == 0:
        findings.append(
            finding("error", root / "skills", "no skill directories found")
        )
    return findings, {"skills": skill_count}


def check_readme_catalog_exposure(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Ensure the top-level catalog names every skill."""
    readme_path = root / "README.md"
    text = read_text(readme_path)
    catalog = skill_catalog_section(text)
    findings: list[dict[str, Any]] = []
    skills = discover_skills(root)
    for skill in skills:
        expected_row_prefix = f"| `{skill.path.name}` |"
        expected_link = (
            f"[skills/{skill.path.name}/SKILL.md]"
            f"(skills/{skill.path.name}/SKILL.md)"
        )
        if expected_row_prefix not in catalog or expected_link not in catalog:
            findings.append(
                finding(
                    "error",
                    skill.skill_md,
                    "skill is missing an exact README catalog row/link: "
                    f"{skill.path.name}",
                )
            )
    return findings, {
        "skills": len(skills),
        "catalog": relative_path(readme_path, root),
    }


def check_docs_reference_exposure(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Ensure expected reference docs are linked from the docs portal."""
    docs_index = root / "docs" / "index.md"
    text = read_text(docs_index)
    findings: list[dict[str, Any]] = []
    reference_docs = sorted((root / "docs" / "reference").glob("*.md"))
    for doc in reference_docs:
        rel = relative_path(doc, root)
        link = doc.relative_to(root / "docs").as_posix()
        if doc.name not in text and link not in text and rel not in text:
            findings.append(
                finding(
                    "error",
                    doc,
                    "reference document is not linked from docs/index.md",
                )
            )
    documented_skills = [
        skill.path.name
        for skill in discover_skills(root)
        if any(skill.path.name in doc.stem for doc in reference_docs)
    ]
    return (
        findings,
        {
            "reference_docs": len(reference_docs),
            "documented_skills": documented_skills,
        },
    )


def check_skill_local_links(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Check tracked skill Markdown links against the local filesystem."""
    tracked = tracked_files(root, "skills")
    markdown_files = sorted(
        path for path in tracked if path.suffix.lower() == ".md"
    )
    findings: list[dict[str, Any]] = []
    checked_links = 0
    for markdown_file in markdown_files:
        text = strip_fenced_code(read_text(markdown_file))
        skill_root = owning_skill_root(root, markdown_file)
        for raw_target in markdown_link_targets(text):
            target = normalize_markdown_target(raw_target)
            if should_skip_link_target(target):
                continue
            resolved = resolve_markdown_target(root, markdown_file, target)
            checked_links += 1
            if not resolved.exists():
                findings.append(
                    finding(
                        "error",
                        markdown_file,
                        f"local Markdown link target does not exist: {target}",
                    )
                )
                continue
            if skill_root and not is_relative_to(resolved, skill_root):
                findings.append(
                    finding(
                        "error",
                        markdown_file,
                        "local Markdown link escapes the packaged skill root: "
                        f"{target}",
                    )
                )
    return (
        findings,
        {
            "markdown_files": len(markdown_files),
            "local_links": checked_links,
        },
    )


def check_skill_script_syntax(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Check tracked helper script syntax with local parsers."""
    tracked = tracked_files(root, "skills")
    findings: list[dict[str, Any]] = []
    python_files = sorted(path for path in tracked if path.suffix == ".py")
    js_files = sorted(
        path for path in tracked if path.suffix in {".js", ".mjs"}
    )
    shell_files = sorted(path for path in tracked if path.suffix == ".sh")

    with tempfile.TemporaryDirectory(
        prefix="dev-skills-eval-pycompile-"
    ) as pycache_dir:
        previous_pycache_prefix = sys.pycache_prefix
        sys.pycache_prefix = pycache_dir
        try:
            for script in python_files:
                try:
                    py_compile.compile(str(script), doraise=True)
                except py_compile.PyCompileError as error:
                    findings.append(
                        finding(
                            "error",
                            script,
                            f"Python syntax check failed: {error.msg}",
                        )
                    )
        finally:
            sys.pycache_prefix = previous_pycache_prefix

    node_path = shutil.which("node")
    if js_files and node_path is None:
        findings.append(
            finding(
                "warning",
                None,
                "node is unavailable; "
                "JavaScript helper syntax checks were skipped",
            )
        )
    elif node_path:
        for script in js_files:
            error = run_script_syntax_command(
                root,
                [node_path, "--check", str(script)],
                script,
                "JavaScript",
            )
            if error is not None:
                findings.append(error)

    bash_path = shutil.which("bash")
    if shell_files and bash_path is None:
        findings.append(
            finding(
                "warning",
                None,
                "bash is unavailable; shell helper syntax checks were skipped",
            )
        )
    elif bash_path:
        for script in shell_files:
            error = run_script_syntax_command(
                root,
                [bash_path, "-n", str(script)],
                script,
                "shell",
            )
            if error is not None:
                findings.append(error)

    return (
        findings,
        {
            "python_files": len(python_files),
            "javascript_files": len(js_files),
            "shell_files": len(shell_files),
        },
    )


def run_script_syntax_command(
    root: Path,
    command: list[str],
    script: Path,
    label: str,
) -> dict[str, Any] | None:
    """Run one script syntax command and return a finding on failure."""
    try:
        completed = subprocess.run(  # noqa: S603
            command,
            cwd=root,
            text=True,
            capture_output=True,
            check=False,
            timeout=SCRIPT_CHECK_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        output = decoded_output(error.stderr) or decoded_output(error.stdout)
        message = (
            f"{label} syntax check timed out after "
            f"{SCRIPT_CHECK_TIMEOUT_SECONDS}s"
        )
        if output:
            message += ": " + tail(output, root)
        return finding("error", script, message)
    if completed.returncode == 0:
        return None
    output = completed.stderr or completed.stdout
    return finding(
        "error",
        script,
        f"{label} syntax check failed: " + tail(output, root),
    )


def decoded_output(value: str | bytes | None) -> str:
    """Return subprocess output as text."""
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode("utf-8", errors="replace")
    return value


def check_generated_cache_exclusion(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Ensure generated cache artifacts are not tracked in skill folders."""
    tracked = tracked_files(root, "skills")
    findings: list[dict[str, Any]] = []
    for path in sorted(tracked):
        relative = path.relative_to(root)
        if (
            path.name in GENERATED_FILE_NAMES
            or path.suffix in GENERATED_SUFFIXES
        ):
            findings.append(
                finding("error", path, "generated cache file is tracked")
            )
            continue
        if any(part in GENERATED_DIR_NAMES for part in relative.parts):
            findings.append(
                finding("error", path, "generated cache directory is tracked")
            )
    return findings, {"tracked_skill_files": len(tracked)}


def check_dist_package_metadata(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Inspect local `.skill` bundles without requiring every skill bundled."""
    dist = root / "skills" / "dist"
    tracked_bundles = {
        path
        for path in tracked_files(root, "skills/dist")
        if path.suffix == ".skill"
    }
    bundles = sorted(dist.glob("*.skill")) if dist.exists() else []
    findings: list[dict[str, Any]] = []
    checked_entries = 0
    for bundle in bundles:
        bundle_severity = "error" if bundle in tracked_bundles else "warning"
        try:
            with zipfile.ZipFile(bundle) as archive:
                names = archive.namelist()
        except zipfile.BadZipFile:
            findings.append(
                finding(
                    bundle_severity, bundle, "bundle is not a valid zip archive"
                )
            )
            continue
        if not names:
            findings.append(finding(bundle_severity, bundle, "bundle is empty"))
            continue
        checked_entries += len(names)
        roots: set[str] = set()
        invalid_names = False
        for name in names:
            invalid_message = invalid_archive_name_message(name)
            if invalid_message:
                invalid_names = True
                findings.append(
                    finding(bundle_severity, bundle, invalid_message)
                )
                continue
            parts = PurePosixPath(name).parts
            roots.add(parts[0])
        if invalid_names:
            continue
        if len(roots) != 1:
            findings.append(
                finding(
                    bundle_severity,
                    bundle,
                    "bundle must contain exactly one top-level skill directory",
                )
            )
            continue
        root_name = next(iter(roots))
        skill_entry = f"{root_name}/SKILL.md"
        if skill_entry not in names:
            findings.append(
                finding(bundle_severity, bundle, "bundle is missing SKILL.md")
            )
        else:
            try:
                with zipfile.ZipFile(bundle) as archive:
                    skill_content = archive.read(skill_entry).decode(
                        "utf-8-sig"
                    )
            except (KeyError, UnicodeDecodeError, OSError) as error:
                findings.append(
                    finding(
                        bundle_severity,
                        bundle,
                        f"could not read SKILL.md: {error}",
                    )
                )
            else:
                findings.extend(
                    bundle_skill_metadata_findings(
                        bundle,
                        root_name,
                        skill_content,
                        bundle_severity,
                    )
                )
        for name in names:
            parts = tuple(part for part in Path(name).parts if part)
            suffix = Path(name).suffix
            if (
                Path(name).name in GENERATED_FILE_NAMES
                or suffix in GENERATED_SUFFIXES
                or any(part in GENERATED_DIR_NAMES for part in parts)
            ):
                findings.append(
                    finding(
                        bundle_severity,
                        bundle,
                        f"bundle includes generated/cache artifact: {name}",
                    )
                )
    return (
        findings,
        {
            "local_bundles": len(bundles),
            "tracked_bundles": len(tracked_bundles),
            "entries": checked_entries,
            "dist": relative_path(dist, root),
        },
    )


def check_openai_agent_metadata(
    root: Path,
) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    """Validate supported `agents/openai.yaml` metadata shapes."""
    paths = sorted((root / "skills").glob("*/agents/openai.yaml"))
    findings: list[dict[str, Any]] = []
    shape_counts = {"interface": 0, "direct": 0, "legacy": 0}
    for path in paths:
        try:
            data = yaml.safe_load(path.read_text(encoding="utf-8"))
        except yaml.YAMLError as error:
            findings.append(finding("error", path, f"invalid YAML: {error}"))
            continue
        if not isinstance(data, dict):
            findings.append(
                finding("error", path, "metadata must be a YAML mapping")
            )
            continue
        shape = openai_metadata_shape(data)
        if shape is None:
            findings.append(
                finding(
                    "error",
                    path,
                    "metadata must use interface, direct, "
                    "or legacy required fields",
                )
            )
        else:
            shape_counts[shape] += 1
        policy = data.get("policy")
        if policy is not None:
            if not isinstance(policy, dict):
                findings.append(
                    finding("error", path, "policy must be a mapping")
                )
            elif "allow_implicit_invocation" in policy and not isinstance(
                policy["allow_implicit_invocation"],
                bool,
            ):
                findings.append(
                    finding(
                        "error",
                        path,
                        "policy.allow_implicit_invocation must be boolean",
                    )
                )
        dependencies = data.get("dependencies")
        if dependencies is not None and not isinstance(dependencies, dict):
            findings.append(
                finding("error", path, "dependencies must be a mapping")
            )
        if (
            data.get("name") is not None
            and data.get("name") != path.parents[1].name
        ):
            findings.append(
                finding(
                    "warning",
                    path,
                    "legacy name does not match the skill directory name",
                )
            )
    return findings, {"files": len(paths), "shapes": shape_counts}


def openai_metadata_shape(data: dict[str, Any]) -> str | None:
    """Return the supported metadata shape name, if the payload is valid."""
    interface = data.get("interface")
    if isinstance(interface, dict) and required_string_fields(
        interface,
        ("display_name", "short_description", "default_prompt"),
    ):
        return "interface"
    if required_string_fields(
        data,
        ("display_name", "short_description", "default_prompt"),
    ):
        return "direct"
    if required_string_fields(data, ("name", "description", "instructions")):
        return "legacy"
    return None


def required_string_fields(
    data: dict[str, Any], fields: tuple[str, ...]
) -> bool:
    """Return whether all required fields exist as non-empty strings."""
    return all(
        isinstance(data.get(field), str) and data[field].strip()
        for field in fields
    )


def discover_skills(root: Path) -> list[SkillRecord]:
    """Discover regular skill directories from `skills/*/SKILL.md`."""
    skills_root = root / "skills"
    records: list[SkillRecord] = []
    for path in sorted(skills_root.iterdir()):
        if path.name == "dist" or not path.is_dir() or path.is_symlink():
            continue
        skill_md = path / "SKILL.md"
        if not skill_md.is_file() or skill_md.is_symlink():
            continue
        frontmatter = read_frontmatter(skill_md)
        name = (
            frontmatter.get("name")
            if isinstance(frontmatter.get("name"), str)
            else path.name
        )
        records.append(
            SkillRecord(
                name=name, path=path, skill_md=skill_md, frontmatter=frontmatter
            )
        )
    return records


def read_frontmatter(path: Path) -> dict[str, Any]:
    """Read YAML frontmatter from a skill file."""
    return parse_frontmatter(path.read_text(encoding="utf-8-sig"))


def parse_frontmatter(content: str) -> dict[str, Any]:
    """Parse skill YAML frontmatter from text."""
    match = re.match(r"^---\n(.*?)\n---", content, re.DOTALL)
    if not match:
        return {}
    loaded = yaml.safe_load(match.group(1))
    return loaded if isinstance(loaded, dict) else {}


def load_quick_validator(
    root: Path,
) -> Callable[[str | Path], tuple[bool, str]]:
    """Import the repo-owned quick validator."""
    skill_tools = str(root / "tools" / "skill")
    if skill_tools not in sys.path:
        sys.path.insert(0, skill_tools)
    from quick_validate import validate_skill  # noqa: PLC0415

    return validate_skill


def tracked_files(root: Path, prefix: str) -> set[Path]:
    """Return tracked files below a repo prefix."""
    command = ["git", "ls-files", "-z", "--", prefix]
    try:
        completed = subprocess.run(  # noqa: S603
            command,
            cwd=root,
            check=True,
            capture_output=True,
            timeout=GIT_LS_FILES_TIMEOUT_SECONDS,
        )
    except subprocess.TimeoutExpired as error:
        raise RuntimeError(
            f"git ls-files timed out for prefix {prefix!r} "
            f"after {GIT_LS_FILES_TIMEOUT_SECONDS}s"
        ) from error
    except (OSError, subprocess.CalledProcessError):
        return {path for path in (root / prefix).rglob("*") if path.is_file()}
    files = set()
    for raw_path in completed.stdout.split(b"\0"):
        if raw_path:
            path = root / raw_path.decode()
            if path.is_file():
                files.add(path)
    return files


def normalize_markdown_target(raw_target: str) -> str:
    """Normalize a Markdown link target for local path checks."""
    target = raw_target.strip()
    if target.startswith("<") and ">" in target:
        target = target[1 : target.index(">")]
    elif " " in target:
        target = target.split(" ", 1)[0]
    return unquote(target.strip("'\""))


def should_skip_link_target(target: str) -> bool:
    """Return whether a link target is outside local path validation."""
    if not target or target.startswith("#"):
        return True
    if re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", target):
        return True
    return False


def resolve_markdown_target(
    root: Path, markdown_file: Path, target: str
) -> Path:
    """Resolve a Markdown link target relative to its source file."""
    path_part = re.split(r"[?#]", target, 1)[0]
    absolute = Path(path_part)
    if absolute.is_absolute() and (
        path_part.startswith(str(root)) or absolute.exists()
    ):
        return absolute.resolve()
    if path_part.startswith("/"):
        return root / path_part.lstrip("/")
    return (markdown_file.parent / path_part).resolve()


def owning_skill_root(root: Path, path: Path) -> Path | None:
    """Return the containing skill root for a path under skills/<name>."""
    try:
        relative = path.relative_to(root)
    except ValueError:
        return None
    if len(relative.parts) < 3 or relative.parts[0] != "skills":
        return None
    if relative.parts[1] == "dist":
        return None
    return root / "skills" / relative.parts[1]


def is_relative_to(path: Path, parent: Path) -> bool:
    """Return whether path is at or below parent."""
    try:
        path.resolve().relative_to(parent.resolve())
    except ValueError:
        return False
    return True


def skill_catalog_section(text: str) -> str:
    """Extract the README skill catalog section."""
    match = re.search(
        r"^## Skill catalog\n(?P<body>.*?)(?=^## |\Z)", text, re.M | re.S
    )
    return match.group("body") if match else ""


def invalid_archive_name_message(name: str) -> str | None:
    """Return a finding message when a ZIP archive entry path is unsafe."""
    if not name:
        return "bundle includes an empty archive entry name"
    if "\\" in name:
        return f"bundle entry uses a non-portable separator: {name}"
    if name.startswith("/") or re.match(r"^[A-Za-z]:", name):
        return f"bundle entry uses an absolute path: {name}"
    parts = PurePosixPath(name).parts
    if not parts or any(part in {"", ".", ".."} for part in parts):
        return f"bundle entry uses a traversal or empty path component: {name}"
    if len(parts) < 2 and not name.endswith("/"):
        return (
            "bundle includes a root-level entry outside "
            f"the skill directory: {name}"
        )
    return None


def bundle_skill_metadata_findings(
    bundle: Path,
    root_name: str,
    skill_content: str,
    severity: str,
) -> list[dict[str, Any]]:
    """Validate metadata embedded inside a `.skill` bundle."""
    findings: list[dict[str, Any]] = []
    if root_name != bundle.stem:
        findings.append(
            finding(
                severity,
                bundle,
                "bundle filename does not match archive root: "
                f"{bundle.stem} != {root_name}",
            )
        )
    with tempfile.TemporaryDirectory(
        prefix="dev-skills-bundle-validate-"
    ) as tmp:
        skill_dir = Path(tmp) / root_name
        skill_dir.mkdir()
        (skill_dir / "SKILL.md").write_text(skill_content, encoding="utf-8")
        validator = load_quick_validator(repo_root())
        valid, message = validator(skill_dir)
    if not valid:
        findings.append(
            finding(severity, bundle, f"bundled SKILL.md invalid: {message}")
        )
    return findings


def read_text(path: Path) -> str:
    """Read UTF-8 text with replacement for uncommon legacy bytes."""
    return path.read_text(encoding="utf-8", errors="replace")


def strip_fenced_code(text: str) -> str:
    """Remove fenced-code bodies before scanning Markdown prose links."""
    lines: list[str] = []
    fence: str | None = None
    for line in text.splitlines():
        stripped = line.lstrip()
        marker = re.match(r"(`{3,}|~{3,})", stripped)
        if marker:
            token = marker.group(1)
            if fence is None:
                fence = token
                lines.append("")
                continue
            if stripped.startswith(fence):
                fence = None
                lines.append("")
                continue
        lines.append(line if fence is None else "")
    return "\n".join(lines)


def markdown_link_targets(text: str) -> list[str]:
    """Extract inline Markdown link destinations with balanced parentheses."""
    targets: list[str] = []
    index = 0
    while index < len(text):
        label_start = text.find("[", index)
        if label_start == -1:
            break
        if is_escaped(text, label_start):
            index = label_start + 1
            continue
        label_end = find_unescaped(text, "]", label_start + 1)
        if label_end == -1:
            break
        paren_start = label_end + 1
        if paren_start >= len(text) or text[paren_start] != "(":
            index = label_end + 1
            continue
        target, paren_end = parse_markdown_destination(text, paren_start + 1)
        if target:
            targets.append(target)
        index = paren_end + 1 if paren_end > paren_start else paren_start + 1
    return targets


def parse_markdown_destination(text: str, start: int) -> tuple[str, int]:
    """Parse a Markdown destination from inside an inline link."""
    index = start
    while index < len(text) and text[index].isspace():
        index += 1
    if index >= len(text):
        return "", index
    if text[index] == "<":
        target_start = index + 1
        index = target_start
        while index < len(text):
            if text[index] == ">" and not is_escaped(text, index):
                return text[target_start:index], find_link_close(
                    text, index + 1
                )
            index += 1
        return text[target_start:index], index

    target_chars: list[str] = []
    depth = 0
    while index < len(text):
        char = text[index]
        if char == "\\" and index + 1 < len(text):
            target_chars.append(text[index + 1])
            index += 2
            continue
        if char == "(":
            depth += 1
            target_chars.append(char)
            index += 1
            continue
        if char == ")":
            if depth == 0:
                return "".join(target_chars), index
            depth -= 1
            target_chars.append(char)
            index += 1
            continue
        if char.isspace() and depth == 0:
            return "".join(target_chars), find_link_close(text, index)
        target_chars.append(char)
        index += 1
    return "".join(target_chars), index


def find_link_close(text: str, start: int) -> int:
    """Find the closing parenthesis for a Markdown inline link."""
    index = start
    in_quote: str | None = None
    while index < len(text):
        char = text[index]
        if char in {"'", '"'} and not is_escaped(text, index):
            in_quote = None if in_quote == char else char
        elif char == ")" and in_quote is None and not is_escaped(text, index):
            return index
        index += 1
    return index


def find_unescaped(text: str, char: str, start: int) -> int:
    """Find an unescaped character in text."""
    index = start
    while index < len(text):
        if text[index] == char and not is_escaped(text, index):
            return index
        index += 1
    return -1


def is_escaped(text: str, index: int) -> bool:
    """Return whether the character at index is escaped by backslashes."""
    backslashes = 0
    cursor = index - 1
    while cursor >= 0 and text[cursor] == "\\":
        backslashes += 1
        cursor -= 1
    return bool(backslashes % 2)


def output_text(value: str | bytes | None) -> str:
    """Normalize subprocess exception output to text."""
    if value is None:
        return ""
    if isinstance(value, bytes):
        return value.decode(errors="replace")
    return value


def tail(text: str, root: Path) -> str:
    """Sanitize and bound command output for report embedding."""
    text = text.replace(str(root), "$REPO").strip()
    if len(text) <= TAIL_CHARS:
        return text
    return "[truncated]\n" + text[-TAIL_CHARS:]


def sanitize_findings(
    findings: list[dict[str, Any]], root: Path
) -> list[dict[str, Any]]:
    """Make finding paths portable."""
    sanitized: list[dict[str, Any]] = []
    for item in findings:
        clean = dict(item)
        if isinstance(clean.get("path"), str):
            clean["path"] = clean["path"].replace(str(root), "$REPO")
        if isinstance(clean.get("message"), str):
            clean["message"] = tail(clean["message"], root)
        sanitized.append(clean)
    return sanitized


def finding(severity: str, path: Path | None, message: str) -> dict[str, Any]:
    """Build a normalized finding."""
    payload = {"severity": severity, "message": message}
    if path is not None:
        payload["path"] = str(path)
    return payload


def count_findings(findings: list[dict[str, Any]], severity: str) -> int:
    """Count findings by severity."""
    return sum(1 for item in findings if item.get("severity") == severity)


def relative_path(path: Path, root: Path) -> str:
    """Return a repo-relative path when possible."""
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return str(path)


def elapsed_ms(started: float) -> int:
    """Return elapsed milliseconds from a monotonic timestamp."""
    return round((time.monotonic() - started) * 1000)


def report(checks: list[EvalCheck], root: Path, strict: bool) -> dict[str, Any]:
    """Run checks and assemble the eval-lab report."""
    results = [run_check(check, root, strict) for check in checks]
    summary = summarize_results(results)
    return {
        "schema": SCHEMA,
        "generated_at": (
            datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
        ),
        "repo_root": portable_repo_root(),
        "strict": strict,
        "ok": summary["failed"] == 0 and summary["timed_out"] == 0,
        "summary": summary,
        "checks": results,
    }


def summarize_results(results: list[dict[str, Any]]) -> dict[str, int]:
    """Build top-level report counts."""
    statuses = {"passed": 0, "warning": 0, "failed": 0, "timed_out": 0}
    errors = 0
    warnings = 0
    for result in results:
        status = result.get("status")
        if status in statuses:
            statuses[status] += 1
        findings = result.get("findings", [])
        if isinstance(findings, list):
            errors += count_findings(findings, "error")
            warnings += count_findings(findings, "warning")
    return {
        "checks": len(results),
        "passed": statuses["passed"],
        "warning": statuses["warning"],
        "failed": statuses["failed"],
        "timed_out": statuses["timed_out"],
        "errors": errors,
        "warnings": warnings,
    }


def render_human(result: dict[str, Any]) -> str:
    """Render a compact human-readable report summary."""
    summary = result["summary"]
    lines = [
        (
            f"{result['schema']} ok={str(result['ok']).lower()} "
            f"strict={str(result['strict']).lower()} "
            f"passed={summary['passed']} warning={summary['warning']} "
            f"failed={summary['failed']} timed_out={summary['timed_out']}"
        )
    ]
    for check in result["checks"]:
        finding_count = len(check.get("findings", []))
        finding_suffix = f", findings={finding_count}" if finding_count else ""
        exit_value = check.get("exit_code")
        exit_label = "native" if exit_value is None else f"exit={exit_value}"
        lines.append(
            (
                f"- {check['status']}: {check['id']} "
                f"({exit_label}, {check['duration_ms']}ms{finding_suffix})"
            )
        )
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    """Run the eval-lab CLI."""
    args = parse_args(argv)
    root = repo_root()
    checks = selected_checks(default_checks(), args.check)

    if args.list:
        payload = {
            "schema": SCHEMA,
            "repo_root": portable_repo_root(),
            "strict": args.strict,
            "checks": [check.to_list_item() for check in checks],
        }
        if args.json:
            print(json.dumps(payload, indent=2))
        else:
            for check in payload["checks"]:
                if check["type"] == "command":
                    print(f"{check['id']}: {' '.join(check['command'])}")
                else:
                    print(f"{check['id']}: native:{check['runner']}")
        return 0

    payload = report(checks, root, args.strict)
    if args.json:
        print(json.dumps(payload, indent=2))
    else:
        print(render_human(payload))
    return 0 if payload["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
