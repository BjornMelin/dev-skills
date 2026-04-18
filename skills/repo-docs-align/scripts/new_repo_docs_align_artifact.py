#!/usr/bin/env python3

import argparse
import datetime as dt
import fnmatch
from pathlib import Path


ARTIFACTS = {
    "drift-map": "drift-map.md",
    "reviewed-surfaces": "reviewed-surfaces.md",
    "exec-plan": "exec-plan.md",
    "retrospective": "retrospective.md",
}


def today_parts(date_override: str | None) -> tuple[str, str, str]:
    if date_override:
        day = dt.datetime.strptime(date_override, "%Y-%m-%d").date()
    else:
        day = dt.date.today()
    return day.strftime("%Y-%m"), day.strftime("%m-%d"), day.isoformat()


def normalize_run_bucket(raw: str) -> str:
    raw = raw.strip()
    if not raw:
        raise SystemExit("Run bucket cannot be empty.")
    if not raw.isdigit():
        raise SystemExit("Run bucket must be numeric, e.g. 01 or 2.")
    value = int(raw, 10)
    if value < 1:
        raise SystemExit("Run bucket must be >= 1.")
    return f"{value:02d}"


def next_run_bucket(day_root: Path) -> str:
    max_seen = 0
    if day_root.exists():
        for child in day_root.iterdir():
            if child.is_dir() and child.name.isdigit():
                max_seen = max(max_seen, int(child.name, 10))
    return f"{max_seen + 1:02d}"


def read_template(skill_root: Path, artifact_key: str) -> str:
    template_path = (skill_root / "templates" / ARTIFACTS[artifact_key]).resolve()
    try:
        return template_path.read_text(encoding="utf-8")
    except OSError as exc:
        raise SystemExit(
            "Failed to load template "
            f"(artifact_key={artifact_key}, skill_root={skill_root}, template_path={template_path}): "
            f"{exc}"
        ) from exc


def render_template(template: str, iso_date: str, repo_name: str, repo_root: Path) -> str:
    return (
        template.replace("{{DATE}}", iso_date)
        .replace("{{REPO_NAME}}", repo_name)
        .replace("{{REPO_ROOT}}", str(repo_root))
    )


def ignore_targets(skill_name: str) -> tuple[str, ...]:
    return (
        ".agents",
        ".agents/",
        f".agents/{skill_name}",
        f".agents/{skill_name}/",
    )


def ensure_gitignore(repo_root: Path, skill_name: str) -> str:
    gitignore_path = repo_root / ".gitignore"
    if gitignore_path.exists():
        current = gitignore_path.read_text(encoding="utf-8")
    else:
        current = ""

    existing = [
        line.strip()
        for line in current.splitlines()
        if line.strip() and not line.lstrip().startswith("#")
    ]
    targets = ignore_targets(skill_name)
    for pattern in existing:
        normalized = pattern.lstrip("/")
        for target in targets:
            if fnmatch.fnmatch(target, normalized) or fnmatch.fnmatch(target.rstrip("/"), normalized.rstrip("/")):
                return "already_ignored"
            if fnmatch.fnmatch(normalized, target) or fnmatch.fnmatch(normalized.rstrip("/"), target.rstrip("/")):
                return "already_ignored"

    rule = ".agents/\n"
    if current and not current.endswith("\n"):
        current += "\n"
    gitignore_path.write_text(current + rule, encoding="utf-8")
    return "added_agents_rule"


def parse_artifacts(raw: str) -> list[str]:
    if raw == "all":
        return list(ARTIFACTS.keys())
    requested = []
    for item in raw.split(","):
        key = item.strip()
        if not key:
            continue
        if key not in ARTIFACTS:
            valid = ", ".join(sorted(ARTIFACTS))
            raise SystemExit(f"Unknown artifact '{key}'. Valid values: {valid}, all")
        requested.append(key)
    if not requested:
        raise SystemExit("No artifacts requested.")
    return requested


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Create typed working artifacts under "
            ".agents/<skill-name>/YYYY-MM/MM-DD/NN/ and ensure ignore hygiene."
        )
    )
    parser.add_argument(
        "--dir",
        default=".",
        help="Target repository directory. Defaults to the current directory.",
    )
    parser.add_argument(
        "--artifacts",
        default="all",
        help=(
            "Comma-separated artifact keys to create. "
            "Valid: drift-map,reviewed-surfaces,exec-plan,retrospective,all"
        ),
    )
    parser.add_argument(
        "--date",
        default=None,
        help="Override date in YYYY-MM-DD format. Defaults to today.",
    )
    parser.add_argument(
        "--run",
        default=None,
        help="Override numeric run bucket for the day, e.g. 01 or 2. Defaults to the next available bucket.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite artifact files if they already exist.",
    )
    args = parser.parse_args()

    repo_root = Path(args.dir).resolve()
    if not repo_root.exists() or not repo_root.is_dir():
        raise SystemExit(f"Target directory does not exist: {repo_root}")

    requested = parse_artifacts(args.artifacts)
    month_dir, day_dir, iso_date = today_parts(args.date)
    skill_name = Path(__file__).resolve().parent.parent.name

    day_root = repo_root / ".agents" / skill_name / month_dir / day_dir
    run_bucket = normalize_run_bucket(args.run) if args.run else next_run_bucket(day_root)
    artifact_dir = day_root / run_bucket
    artifact_dir.mkdir(parents=True, exist_ok=True)

    ignore_status = ensure_gitignore(repo_root, skill_name)
    skill_root = Path(__file__).resolve().parent.parent
    repo_name = repo_root.name

    created_paths: list[Path] = []
    for artifact_key in requested:
        filename = ARTIFACTS[artifact_key]
        target = artifact_dir / filename
        if target.exists() and not args.force:
            continue
        rendered = render_template(
            read_template(skill_root, artifact_key),
            iso_date=iso_date,
            repo_name=repo_name,
            repo_root=repo_root,
        )
        target.write_text(rendered.rstrip() + "\n", encoding="utf-8")
        created_paths.append(target)

    print(f"artifact_dir={artifact_dir}")
    print(f"run_bucket={run_bucket}")
    print(f"gitignore={ignore_status}")
    for path in created_paths:
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
