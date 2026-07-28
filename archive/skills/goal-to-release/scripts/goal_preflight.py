#!/usr/bin/env python3
"""Collect non-mutating preflight facts for a goal-to-release run."""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import time
from pathlib import Path
from typing import Any


def run(args: list[str], cwd: Path, *, timeout: int = 60) -> subprocess.CompletedProcess[str]:
    try:
        return subprocess.run(
            args,
            cwd=cwd,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout,
            check=False,
        )
    except FileNotFoundError as exc:
        return subprocess.CompletedProcess(args, 127, "", str(exc))
    except subprocess.TimeoutExpired as exc:
        stdout = exc.stdout if isinstance(exc.stdout, str) else ""
        stderr = exc.stderr if isinstance(exc.stderr, str) else ""
        return subprocess.CompletedProcess(args, 124, stdout, stderr or "command timed out")


def git(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return run(["git", *args], cwd)


def repo_root(start: Path) -> Path | None:
    result = git(["rev-parse", "--show-toplevel"], start)
    if result.returncode != 0:
        return None
    return Path(result.stdout.strip()).resolve()


def read_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}


def exists(root: Path, *names: str) -> list[str]:
    return [name for name in names if (root / name).exists()]


def check_ignore(root: Path, target: str) -> bool:
    result = git(["check-ignore", "-q", "--", target], root)
    return result.returncode == 0


def package_manager(root: Path, package_json: dict[str, Any]) -> str | None:
    declared = package_json.get("packageManager")
    if isinstance(declared, str) and declared:
        return declared.split("@", 1)[0]
    if (root / "pnpm-lock.yaml").exists():
        return "pnpm"
    if (root / "bun.lock").exists() or (root / "bun.lockb").exists():
        return "bun"
    if (root / "yarn.lock").exists():
        return "yarn"
    if (root / "package-lock.json").exists():
        return "npm"
    return None


def script_command(manager: str, script: str) -> list[str]:
    if manager == "bun":
        return ["bun", "run", script]
    if manager == "yarn":
        return ["yarn", script]
    if manager == "npm":
        return ["npm", "run", script]
    return [manager, script]


def discover_commands(root: Path, package_json: dict[str, Any]) -> list[dict[str, Any]]:
    commands: list[dict[str, Any]] = [
        {
            "command": ["git", "diff", "--check"],
            "reason": "detect whitespace/conflict-marker issues before review",
            "scope": "git",
            "run_by_default": True,
        }
    ]

    manager = package_manager(root, package_json)
    scripts = package_json.get("scripts") if isinstance(package_json.get("scripts"), dict) else {}
    if manager and scripts:
        if "check" in scripts:
            commands.append(
                {
                    "command": script_command(manager, "check"),
                    "reason": "repo canonical aggregate validation",
                    "scope": "javascript",
                    "run_by_default": False,
                }
            )
        else:
            for script in ("lint", "typecheck", "test", "build"):
                if script in scripts:
                    commands.append(
                        {
                            "command": script_command(manager, script),
                            "reason": f"repo {script} gate",
                            "scope": "javascript",
                            "run_by_default": False,
                        }
                    )

        if (root / "biome.json").exists():
            commands.append(
                {
                    "command": [manager, "exec", "biome", "ci", "--error-on-warnings", "."]
                    if manager not in {"npm", "yarn"}
                    else [manager, "exec", "biome", "ci", "--error-on-warnings", "."],
                    "reason": "format/lint contract when Biome is present",
                    "scope": "javascript",
                    "run_by_default": False,
                }
            )

    if (root / "Cargo.toml").exists():
        commands.extend(
            [
                {
                    "command": ["cargo", "fmt", "--all", "--", "--check"],
                    "reason": "Rust formatting gate",
                    "scope": "rust",
                    "run_by_default": False,
                },
                {
                    "command": [
                        "cargo",
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--all-features",
                        "--locked",
                        "--",
                        "-D",
                        "warnings",
                    ],
                    "reason": "Rust lint gate",
                    "scope": "rust",
                    "run_by_default": False,
                },
                {
                    "command": [
                        "cargo",
                        "test",
                        "--workspace",
                        "--all-targets",
                        "--all-features",
                        "--locked",
                    ],
                    "reason": "Rust test gate",
                    "scope": "rust",
                    "run_by_default": False,
                },
            ]
        )

    if (root / "pyproject.toml").exists():
        commands.append(
            {
                "command": ["uv", "run", "pytest"],
                "reason": "Python test gate when pyproject is present",
                "scope": "python",
                "run_by_default": False,
            }
        )

    return commands


def discover_surfaces(root: Path, package_json: dict[str, Any]) -> list[str]:
    surfaces: list[str] = []
    if package_json:
        surfaces.append("javascript/typescript workspace")
    if (root / "turbo.json").exists():
        surfaces.append("turborepo pipeline")
    if (root / "Cargo.toml").exists():
        surfaces.append("rust workspace")
    if (root / "pyproject.toml").exists():
        surfaces.append("python package")
    if (root / "apps").is_dir():
        surfaces.append("apps directory")
    if (root / "packages").is_dir():
        surfaces.append("packages directory")
    if (root / ".github" / "workflows").is_dir():
        surfaces.append("github actions")
    if (root / "docs").is_dir():
        surfaces.append("docs")
    if (root / "vercel.json").exists() or (root / ".vercel").exists():
        surfaces.append("vercel deployment")
    if (root / "convex").exists() or any(root.glob("**/convex/schema.ts")):
        surfaces.append("convex backend")
    return surfaces


def suggested_reviewers(surfaces: list[str], package_json: dict[str, Any]) -> list[str]:
    suggestions = ["reviewer"]
    joined = " ".join(surfaces)
    if "javascript/typescript" in joined or package_json:
        suggestions.append("bun_ts_reviewer")
    if "turborepo" in joined:
        suggestions.append("performance_reviewer")
    if "rust" in joined:
        suggestions.append("rust-focused reviewer")
    if "vercel" in joined:
        suggestions.append("vercel_reviewer")
    if "convex" in joined:
        suggestions.append("convex_reviewer")
    if "github actions" in joined:
        suggestions.append("ci_triager")
    suggestions.extend(["security_reviewer", "docs_aligner"])
    return list(dict.fromkeys(suggestions))


def gh_pr_snapshot(root: Path, pr: str | None) -> dict[str, Any]:
    if not pr:
        result = run(["gh", "pr", "view", "--json", "number,title,state,url,headRefName,baseRefName,isDraft"], root)
    else:
        result = run(
            ["gh", "pr", "view", pr, "--json", "number,title,state,url,headRefName,baseRefName,isDraft"],
            root,
        )
    if result.returncode != 0:
        return {"available": False, "error": (result.stderr or result.stdout).strip()}
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        return {"available": False, "error": f"invalid gh JSON: {exc}"}
    payload["available"] = True
    return payload


def run_recommended_commands(
    root: Path,
    commands: list[dict[str, Any]],
    *,
    run_all: bool,
) -> list[dict[str, Any]]:
    results: list[dict[str, Any]] = []
    selected = commands if run_all else [item for item in commands if item.get("run_by_default")]
    for item in selected:
        started = time.time()
        command = item["command"]
        result = run(command, root, timeout=900)
        output = "\n".join(part for part in (result.stdout.strip(), result.stderr.strip()) if part)
        results.append(
            {
                "command": command,
                "returncode": result.returncode,
                "duration_seconds": round(time.time() - started, 3),
                "output_tail": output[-4000:],
            }
        )
    return results


def build_preflight(args: argparse.Namespace) -> dict[str, Any]:
    start = Path(args.repo_root).expanduser().resolve()
    root = repo_root(start)
    blockers: list[str] = []
    warnings: list[str] = []

    if root is None:
        blockers.append(f"not a git repository: {start}")
        root = start

    branch_result = git(["branch", "--show-current"], root)
    branch = branch_result.stdout.strip() or "(detached)"
    status_result = git(["status", "--short"], root)
    dirty_files = [line for line in status_result.stdout.splitlines() if line.strip()]
    if dirty_files:
        warnings.append(f"{len(dirty_files)} dirty worktree entr{'y' if len(dirty_files) == 1 else 'ies'}")

    package_json = read_json(root / "package.json")
    surfaces = discover_surfaces(root, package_json)
    commands = discover_commands(root, package_json)

    goal_dirs = {
        ".agents/goals": check_ignore(root, ".agents/goals"),
        ".codex/goals": check_ignore(root, ".codex/goals"),
    }
    if not any(goal_dirs.values()):
        warnings.append("neither .agents/goals nor .codex/goals is git-ignored; use home fallback for ledger artifacts")

    docs = exists(root, "AGENTS.md", "CLAUDE.md", "README.md", "docs", "vercel.json", ".github")
    locks = exists(root, "pnpm-lock.yaml", "bun.lock", "bun.lockb", "yarn.lock", "package-lock.json", "Cargo.lock", "uv.lock")
    configs = exists(root, "package.json", "turbo.json", "tsconfig.json", "biome.json", "eslint.config.js", "Cargo.toml", "pyproject.toml")

    result: dict[str, Any] = {
        "repo_root": str(root),
        "branch": branch,
        "dirty_files": dirty_files[:200],
        "dirty_file_count": len(dirty_files),
        "goal_storage_gitignored": goal_dirs,
        "package_manager": package_manager(root, package_json),
        "surfaces": surfaces,
        "configs": configs,
        "locks": locks,
        "docs_and_ops": docs,
        "scripts": sorted(package_json.get("scripts", {}).keys()) if isinstance(package_json.get("scripts"), dict) else [],
        "recommended_commands": commands,
        "suggested_subagent_reviews": suggested_reviewers(surfaces, package_json),
        "blockers": blockers,
        "warnings": warnings,
    }

    if args.include_gh:
        result["github_pr"] = gh_pr_snapshot(root, args.pr)

    if args.run or args.run_all:
        run_results = run_recommended_commands(root, commands, run_all=args.run_all)
        result["command_run_scope"] = "all recommended commands" if args.run_all else "run_by_default commands"
        result["command_results"] = run_results
        failures = [
            shlex.join(item["command"])
            for item in run_results
            if item["returncode"] != 0
        ]
        if failures:
            blockers.extend(f"preflight command failed: {command}" for command in failures)

    return result


def render_markdown(payload: dict[str, Any]) -> str:
    lines = ["# Goal Preflight", ""]
    lines.append(f"- Repo: `{payload['repo_root']}`")
    lines.append(f"- Branch: `{payload['branch']}`")
    lines.append(f"- Dirty files: {payload['dirty_file_count']}")
    if payload.get("package_manager"):
        lines.append(f"- Package manager: `{payload['package_manager']}`")
    lines.append("")

    for heading, field in (
        ("Surfaces", "surfaces"),
        ("Configs", "configs"),
        ("Locks", "locks"),
        ("Docs and Ops", "docs_and_ops"),
        ("Suggested Subagent Reviews", "suggested_subagent_reviews"),
    ):
        values = payload.get(field, [])
        if not values:
            continue
        lines.append(f"## {heading}")
        lines.extend(f"- {value}" for value in values)
        lines.append("")

    lines.append("## Goal Storage")
    for path, ignored in payload["goal_storage_gitignored"].items():
        lines.append(f"- `{path}`: {'ignored' if ignored else 'not ignored'}")
    lines.append("")

    lines.append("## Recommended Commands")
    for item in payload["recommended_commands"]:
        default_marker = "default run" if item.get("run_by_default") else "manual/explicit"
        lines.append(f"- `{shlex.join(item['command'])}` - {item['reason']} ({default_marker})")
    lines.append("")

    if payload.get("command_results"):
        lines.append("## Command Results")
        for item in payload["command_results"]:
            status = "PASS" if item["returncode"] == 0 else f"FAIL {item['returncode']}"
            lines.append(f"- `{shlex.join(item['command'])}`: {status} ({item['duration_seconds']}s)")
        lines.append("")

    if payload.get("github_pr"):
        lines.append("## GitHub PR")
        gh = payload["github_pr"]
        if gh.get("available"):
            lines.append(f"- PR: #{gh.get('number')} {gh.get('title')} ({gh.get('state')})")
            lines.append(f"- URL: {gh.get('url')}")
        else:
            lines.append(f"- unavailable: {gh.get('error')}")
        lines.append("")

    if payload["blockers"]:
        lines.append("## Blockers")
        lines.extend(f"- {item}" for item in payload["blockers"])
        lines.append("")
    if payload["warnings"]:
        lines.append("## Warnings")
        lines.extend(f"- {item}" for item in payload["warnings"])
        lines.append("")

    return "\n".join(lines).rstrip() + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Collect goal-to-release preflight facts.")
    parser.add_argument("--repo-root", default=".", help="Repository path to inspect.")
    parser.add_argument("--include-gh", action="store_true", help="Include a best-effort GitHub PR snapshot.")
    parser.add_argument("--pr", help="PR number or URL for --include-gh.")
    parser.add_argument(
        "--run",
        action="store_true",
        help="Run only commands marked run_by_default, such as git diff --check.",
    )
    parser.add_argument(
        "--run-all",
        action="store_true",
        help="With --run, execute all recommended commands. Use only when broad validation is intended.",
    )
    parser.add_argument("--strict", action="store_true", help="Treat warnings as a nonzero result.")
    parser.add_argument(
        "--format",
        choices=("markdown", "json", "both"),
        default="markdown",
        help="Output format.",
    )
    parser.add_argument("--write-dir", help="Write preflight.md and/or preflight.json to this directory.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    payload = build_preflight(args)
    markdown = render_markdown(payload)

    if args.write_dir:
        out = Path(args.write_dir).expanduser()
        out.mkdir(parents=True, exist_ok=True)
        if args.format in {"markdown", "both"}:
            (out / "preflight.md").write_text(markdown, encoding="utf-8")
        if args.format in {"json", "both"}:
            (out / "preflight.json").write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")

    if args.format == "json":
        print(json.dumps(payload, indent=2, sort_keys=True))
    elif args.format == "both":
        print(markdown)
        print(json.dumps(payload, indent=2, sort_keys=True))
    else:
        print(markdown, end="")

    if payload["blockers"]:
        return 1
    if args.strict and payload["warnings"]:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
