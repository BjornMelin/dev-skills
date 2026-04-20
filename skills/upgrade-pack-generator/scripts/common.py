#!/usr/bin/env python3
"""Shared helpers for upgrade-pack-generator scripts."""

from __future__ import annotations

import json
import re
import shlex
import shutil
from collections import Counter
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError as exc:  # pragma: no cover - environment-specific guard
    raise SystemExit(
        "PyYAML is required for upgrade-pack-generator scripts. "
        "Install it with `python3 -m pip install pyyaml`."
    ) from exc


EXCLUDED_PARTS = {
    ".cache",
    ".codex",
    ".git",
    ".idea",
    ".next",
    ".turbo",
    ".vscode",
    "__pycache__",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "opensrc",
    "out",
    ".agents",
}

ROOT_LOCKFILES = {
    "bun.lock": "bun",
    "bun.lockb": "bun",
    "pnpm-lock.yaml": "pnpm",
    "package-lock.json": "npm",
    "yarn.lock": "yarn",
}

FAMILY_OVERLAY_KEYWORDS = {
    "convex": ("convex",),
    "expo-eas": ("expo", "eas", "react-native"),
    "lucide-react": ("lucide",),
    "nextjs": ("next",),
    "shadcn-radix-ui": ("shadcn", "radix"),
    "turborepo": ("turbo", "monorepo"),
}

FRAMEWORK_PACKAGES = {
    "next": "nextjs",
    "expo": "expo",
    "react-native": "react-native",
    "turbo": "turborepo",
    "radix-ui": "radix-ui",
    "lucide-react": "lucide-react",
    "convex": "convex",
}

DOCS_CI_FILES = (
    "AGENTS.md",
    "README.md",
    "README",
    "package.json",
    "pnpm-workspace.yaml",
    "bunfig.toml",
)

MANAGER_TOKEN_PATTERNS = {
    "bun": re.compile(r"\bbunx?\b|\bbun run\b"),
    "pnpm": re.compile(r"\bpnpm\b"),
    "npm": re.compile(r"\bnpm run\b|\bnpx\b"),
    "yarn": re.compile(r"\byarn\b"),
}

PACKAGE_JSON_SECTIONS = ("dependencies", "devDependencies", "peerDependencies")
SOURCE_TEXT_SUFFIXES = {".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs", ".json", ".md", ".yml", ".yaml"}
SOURCE_MAP_FILENAME = "package_source_map.json"


def skill_root_from_script(script_path: str | Path) -> Path:
    """Return the upgrade-pack-generator skill root for a script path."""
    return Path(script_path).expanduser().resolve().parents[1]


def bundled_source_map_path(script_path: str | Path) -> Path:
    """Return the bundled package source-map path."""
    return skill_root_from_script(script_path) / "references" / "source-maps" / SOURCE_MAP_FILENAME


def load_bundled_source_map(script_path: str | Path) -> list[dict[str, Any]]:
    """Load the bundled source map."""
    path = bundled_source_map_path(script_path)
    if not path.exists():
        return []
    payload = load_json(path)
    if not isinstance(payload, list):
        return []
    return [item for item in payload if isinstance(item, dict)]


def source_map_entry(script_path: str | Path, package_name: str) -> dict[str, Any] | None:
    """Return the first matching bundled source-map entry for a package."""
    normalized = package_name.strip()
    if not normalized:
        return None
    for entry in load_bundled_source_map(script_path):
        if str(entry.get("packageName") or "").strip() == normalized:
            return entry
    return None


def tool_available(name: str) -> bool:
    """Return whether a binary is present on PATH."""
    return shutil.which(name) is not None


def safe_read_text(path: str | Path) -> str:
    """Read UTF-8 text from disk while tolerating decoding issues."""
    return Path(path).read_text(encoding="utf-8", errors="ignore")


def load_yaml(path: str | Path) -> Any:
    """Load a YAML file."""
    with Path(path).open("r", encoding="utf-8") as handle:
        return yaml.safe_load(handle)


def dump_yaml(path: str | Path, data: Any) -> None:
    """Write YAML with deterministic ordering preserved."""
    with Path(path).open("w", encoding="utf-8") as handle:
        yaml.safe_dump(data, handle, sort_keys=False, allow_unicode=False)


def load_json(path: str | Path) -> Any:
    """Load JSON from disk."""
    return json.loads(safe_read_text(path))


def normalize_slug(value: str) -> str:
    """Normalize a package or family name into a folder-safe slug."""
    slug = value.strip().lower().replace("/", "-")
    slug = slug.replace("@", "")
    slug = re.sub(r"[^a-z0-9-]+", "-", slug)
    slug = re.sub(r"-{2,}", "-", slug).strip("-")
    return slug or "upgrade-pack"


def normalize_package_version_for_source(version: str) -> str:
    """Normalize a manifest version string into an opensrc-friendly pinned version."""
    cleaned = version.strip()
    cleaned = re.sub(r"^[~^<>=\s]+", "", cleaned)
    cleaned = cleaned.split(" ", 1)[0].split("||", 1)[0].strip()
    match = re.search(r"\d+(?:\.\d+){0,3}(?:[-+][A-Za-z0-9.-]+)?", cleaned)
    return match.group(0) if match else cleaned


def titleize_package(value: str) -> str:
    """Generate a simple display title for a package name."""
    base = value.replace("@", "").replace("/", " ")
    parts = re.split(r"[-_\s]+", base.strip())
    return " ".join(part.capitalize() for part in parts if part)


def repo_path(path: str | Path) -> Path:
    """Resolve and validate a repo root path."""
    root = Path(path).expanduser().resolve()
    if not root.exists() or not root.is_dir():
        raise SystemExit(f"Repo root does not exist: {root}")
    return root


def root_package_json_data(root: Path) -> dict[str, Any]:
    """Return the root package.json payload when present."""
    package_json = root / "package.json"
    if not package_json.exists():
        return {}
    data = load_json(package_json)
    return data if isinstance(data, dict) else {}


def manifest_dependency_map(data: dict[str, Any]) -> dict[str, str]:
    """Collect package versions across dependency sections."""
    detected: dict[str, str] = {}
    for section in PACKAGE_JSON_SECTIONS:
        values = data.get(section) or {}
        if not isinstance(values, dict):
            continue
        for package, version in values.items():
            if isinstance(package, str) and isinstance(version, str):
                detected[package] = version
    return detected


def manifest_dependency_names(data: dict[str, Any]) -> set[str]:
    """Return all dependency names declared in a manifest."""
    return set(manifest_dependency_map(data).keys())


def manifest_scripts(data: dict[str, Any]) -> dict[str, str]:
    """Return normalized script strings from a manifest."""
    scripts = data.get("scripts") or {}
    if not isinstance(scripts, dict):
        return {}
    return {
        str(name): str(command)
        for name, command in scripts.items()
        if isinstance(name, str) and isinstance(command, str)
    }


def _is_excluded(path: Path) -> bool:
    return any(part in EXCLUDED_PARTS for part in path.parts)


def iter_named_files(root: Path, filename: str) -> list[Path]:
    """Find files with a given name while skipping heavy/generated directories."""
    results: list[Path] = []
    for path in root.rglob(filename):
        if _is_excluded(path):
            continue
        results.append(path)
    return sorted(results)


def root_lockfiles(root: Path) -> list[str]:
    """Return root lockfiles present at the repo root."""
    present: list[str] = []
    for name in ROOT_LOCKFILES:
        if (root / name).exists():
            present.append(name)
    return present


def package_manager_field(root: Path) -> str | None:
    """Return packageManager from root package.json when present."""
    package_json = root / "package.json"
    if not package_json.exists():
        return None
    data = load_json(package_json)
    raw = data.get("packageManager")
    if not raw or not isinstance(raw, str):
        return None
    return raw.split("@", 1)[0].strip() or None


def docs_ci_hints(root: Path) -> dict[str, int]:
    """Count package-manager hints from docs and CI files."""
    counter: Counter[str] = Counter()
    candidates = [root / name for name in DOCS_CI_FILES if (root / name).exists()]

    workflows = root / ".github" / "workflows"
    if workflows.exists():
        candidates.extend(
            path
            for path in workflows.rglob("*")
            if path.is_file() and path.suffix in {".yml", ".yaml"}
        )

    for path in candidates:
        text = path.read_text(encoding="utf-8", errors="ignore")
        for manager, pattern in MANAGER_TOKEN_PATTERNS.items():
            counter[manager] += len(pattern.findall(text))

    return {key: value for key, value in sorted(counter.items()) if value}


def detect_package_manager(root: Path) -> dict[str, Any]:
    """Detect the repo package manager with packageManager, lockfiles, then docs/CI."""
    field = package_manager_field(root)
    locks = root_lockfiles(root)
    hints = docs_ci_hints(root)

    if field in {"bun", "pnpm", "npm", "yarn"}:
        return {
            "package_manager": field,
            "detected_by": "packageManager",
            "package_manager_field": field,
            "root_lockfiles": locks,
            "docs_ci_hints": hints,
        }

    lock_managers = {ROOT_LOCKFILES[name] for name in locks}
    if len(lock_managers) == 1:
        return {
            "package_manager": next(iter(lock_managers)),
            "detected_by": "root-lockfile",
            "package_manager_field": field,
            "root_lockfiles": locks,
            "docs_ci_hints": hints,
        }

    if len(lock_managers) > 1:
        return {
            "package_manager": "mixed",
            "detected_by": "root-lockfiles",
            "package_manager_field": field,
            "root_lockfiles": locks,
            "docs_ci_hints": hints,
        }

    if hints:
        manager = max(hints.items(), key=lambda item: item[1])[0]
        return {
            "package_manager": manager,
            "detected_by": "docs-ci-hints",
            "package_manager_field": field,
            "root_lockfiles": locks,
            "docs_ci_hints": hints,
        }

    return {
        "package_manager": "unknown",
        "detected_by": "unknown",
        "package_manager_field": field,
        "root_lockfiles": locks,
        "docs_ci_hints": hints,
    }


def command_family_variables(package_manager: str) -> dict[str, str]:
    """Return package-manager command variable suggestions."""
    mapping = {
        "bun": {
            "PM_DLX": "bunx",
            "PM_RUN": "bun run",
            "PM_TEST": "bun test",
            "PM_AUDIT": "bun audit",
        },
        "pnpm": {
            "PM_DLX": "pnpm dlx",
            "PM_RUN": "pnpm",
            "PM_TEST": "pnpm test -- --run",
            "PM_AUDIT": "pnpm audit --json",
        },
        "npm": {
            "PM_DLX": "npx",
            "PM_RUN": "npm run",
            "PM_TEST": "npm test -- --run",
            "PM_AUDIT": "npm audit --json",
        },
        "yarn": {
            "PM_DLX": "yarn dlx",
            "PM_RUN": "yarn",
            "PM_TEST": "yarn test --run",
            "PM_AUDIT": "<repo-native yarn audit command>",
        },
    }
    return mapping.get(
        package_manager,
        {
            "PM_DLX": "<repo-native dlx command>",
            "PM_RUN": "<repo-native run command>",
            "PM_TEST": "<repo-native test command>",
            "PM_AUDIT": "<repo-native audit command>",
        },
    )


def workspace_manifest_records(root: Path) -> list[dict[str, Any]]:
    """Return normalized workspace package records, including the repo root."""
    records: list[dict[str, Any]] = []
    for path in iter_named_files(root, "package.json"):
        data = load_json(path)
        if not isinstance(data, dict):
            continue
        rel_path = path.relative_to(root)
        workspace_path = rel_path.parent.as_posix() or "."
        dependencies = manifest_dependency_map(data)
        scripts = manifest_scripts(data)
        package_name = data.get("name")
        records.append(
            {
                "workspace_path": workspace_path,
                "package_json_path": rel_path.as_posix(),
                "package_name": package_name if isinstance(package_name, str) else None,
                "is_root": workspace_path == ".",
                "dependencies": dependencies,
                "dependency_names": sorted(dependencies),
                "scripts": scripts,
                "data": data,
            }
        )
    return sorted(records, key=lambda item: (not item["is_root"], item["workspace_path"]))


def root_manifest_record(root: Path) -> dict[str, Any]:
    """Return the normalized root manifest record."""
    for record in workspace_manifest_records(root):
        if record["is_root"]:
            return record
    return {
        "workspace_path": ".",
        "package_json_path": "package.json",
        "package_name": None,
        "is_root": True,
        "dependencies": {},
        "dependency_names": [],
        "scripts": {},
        "data": {},
    }


def workspace_dir(root: Path, record: dict[str, Any]) -> Path:
    """Return the workspace root directory for a package record."""
    workspace_path = str(record.get("workspace_path") or ".")
    return root if workspace_path == "." else root / workspace_path


def workspace_display_path(record: dict[str, Any]) -> str:
    """Return a human-readable workspace path label."""
    return str(record.get("workspace_path") or ".")


def workspace_slug(record: dict[str, Any]) -> str:
    """Return a stable slug for a workspace path."""
    path = workspace_display_path(record)
    return "root" if path == "." else normalize_slug(path)


def workspace_reference(record: dict[str, Any]) -> str:
    """Return a concise owner label for a workspace."""
    package_name = record.get("package_name")
    if isinstance(package_name, str) and package_name.strip():
        return f"{workspace_display_path(record)} ({package_name})"
    return workspace_display_path(record)


def workspace_exists_any(root: Path, record: dict[str, Any], candidates: list[str]) -> str | None:
    """Return the first workspace-relative candidate path that exists."""
    base = workspace_dir(root, record)
    for candidate in candidates:
        path = base / candidate
        if path.exists():
            prefix = workspace_display_path(record)
            rel = Path(candidate).as_posix()
            return rel if prefix == "." else f"{prefix}/{rel}"
    return None


def package_versions_from_manifest(record: dict[str, Any], packages: list[str]) -> dict[str, str]:
    """Return detected package versions from a specific manifest record."""
    dependencies = record.get("dependencies") or {}
    return {package: str(dependencies[package]) for package in packages if package in dependencies}


def package_versions_from_repo(root: Path, packages: list[str]) -> dict[str, str]:
    """Return detected package versions from the root package manifest."""
    return package_versions_from_manifest(root_manifest_record(root), packages)


def manifests_declaring_package(root: Path, package: str) -> list[dict[str, Any]]:
    """Return all manifest records that declare a given package."""
    return [
        record
        for record in workspace_manifest_records(root)
        if package in (record.get("dependencies") or {})
    ]


def pick_script(record: dict[str, Any], candidates: list[str]) -> str | None:
    """Pick the first matching script name from a record."""
    scripts = record.get("scripts") or {}
    for candidate in candidates:
        if candidate in scripts:
            return candidate
    return None


def root_script_command(package_manager: str, script: str) -> str:
    """Build a root-level script command for the detected package manager."""
    if package_manager == "bun":
        return f"bun run {script}"
    if package_manager == "pnpm":
        return f"pnpm {script}"
    if package_manager == "npm":
        return f"npm run {script}"
    if package_manager == "yarn":
        return f"yarn {script}"
    return f"<repo-native root command for {script}>"


def dlx_command(package_manager: str, package_spec: str, args: str = "") -> str:
    """Build a package-manager-aware dlx command."""
    suffix = f" {args.strip()}" if args.strip() else ""
    if package_manager == "bun":
        return f"bunx {package_spec}{suffix}"
    if package_manager == "pnpm":
        return f"pnpm dlx {package_spec}{suffix}"
    if package_manager == "npm":
        return f"npx {package_spec}{suffix}"
    if package_manager == "yarn":
        return f"yarn dlx {package_spec}{suffix}"
    return f"<repo-native dlx command> {package_spec}{suffix}".rstrip()


def workspace_script_command(package_manager: str, record: dict[str, Any], script: str) -> str:
    """Build a workspace-aware script command for the detected package manager."""
    if record.get("is_root"):
        return root_script_command(package_manager, script)

    workspace_name = record.get("package_name")
    workspace_path = workspace_display_path(record)

    if package_manager == "bun":
        if workspace_name:
            return f"bun run --filter {shlex.quote(str(workspace_name))} {script}"
        return f"bun run --cwd {shlex.quote(workspace_path)} {script}"
    if package_manager == "pnpm":
        if workspace_name:
            return f"pnpm --filter {shlex.quote(str(workspace_name))} {script}"
        return f"pnpm -C {shlex.quote(workspace_path)} {script}"
    if package_manager == "npm":
        target = shlex.quote(str(workspace_name or workspace_path))
        return f"npm run {script} --workspace {target}"
    if package_manager == "yarn":
        if workspace_name:
            return f"yarn workspace {shlex.quote(str(workspace_name))} {script}"
        return f"(cd {shlex.quote(workspace_path)} && yarn {script})"
    return f"<repo-native workspace command for {workspace_path}:{script}>"


def repo_local_skill_overlays(root: Path, family_slug: str) -> list[dict[str, str]]:
    """Return repo-local skill overlays that match a family slug."""
    keywords = FAMILY_OVERLAY_KEYWORDS.get(family_slug, ())
    if not keywords:
        return []

    skills_dir = root / ".agents" / "skills"
    if not skills_dir.exists():
        return []

    overlays: list[dict[str, str]] = []
    for skill_md in sorted(skills_dir.glob("*/SKILL.md")):
        skill_dir = skill_md.parent.name
        skill_key = skill_dir.lower()
        if not any(keyword in skill_key for keyword in keywords):
            continue
        matched = ", ".join(sorted(keyword for keyword in keywords if keyword in skill_key))
        overlays.append(
            {
                "skill_name": skill_dir,
                "skill_path": repo_relative_path(root, skill_md),
                "reason": f"matched family keywords: {matched}",
            }
        )
    return overlays


def repo_relative_path(root: Path, path: Path) -> str:
    """Return a POSIX repo-relative path string."""
    return path.relative_to(root).as_posix()


def source_files_under(base: Path) -> list[Path]:
    """Return source-like files under a directory while respecting exclusions."""
    return [
        path
        for path in base.rglob("*")
        if path.is_file() and not _is_excluded(path) and path.suffix in SOURCE_TEXT_SUFFIXES
    ]


def detect_import_inventory(base: Path, patterns: dict[str, str]) -> dict[str, int]:
    """Count matches for import or API patterns under a specific directory."""
    files = source_files_under(base)
    counts: dict[str, int] = {}
    for label, pattern in patterns.items():
        compiled = re.compile(pattern)
        total = 0
        for path in files:
            total += len(compiled.findall(safe_read_text(path)))
        counts[label] = total
    return counts


def app_route_files(root: Path, record: dict[str, Any]) -> list[str]:
    """Return repo-relative App Router file paths for a workspace."""
    base = workspace_dir(root, record)
    matches: set[Path] = set()
    for app_base in (base / "app", base / "src" / "app"):
        if not app_base.exists():
            continue
        for pattern in ("**/page.*", "**/layout.*", "**/route.*", "**/default.*", "**/not-found.*"):
            matches.update(path for path in app_base.glob(pattern) if path.is_file())
    return [repo_relative_path(root, path) for path in sorted(matches)]


def repo_exists_any(root: Path, candidates: list[str]) -> str | None:
    """Return the first repo-relative candidate path that exists."""
    for candidate in candidates:
        path = root / candidate
        if path.exists():
            return candidate
    return None


def next_repo_probes(root: Path, record: dict[str, Any]) -> dict[str, list[str]]:
    """Build high-signal Next.js probe data for a workspace owner."""
    package_json = record.get("data") or {}
    scripts = record.get("scripts") or {}
    base = workspace_dir(root, record)
    next_config = workspace_exists_any(
        root,
        record,
        ["next.config.ts", "next.config.mjs", "next.config.js", "next.config.cjs"],
    )
    next_config_text = safe_read_text(root / next_config) if next_config else ""
    app_files = app_route_files(root, record)
    has_app_router = bool(app_files)
    has_pages_router = workspace_exists_any(root, record, ["pages", "src/pages"]) is not None
    router_mode = (
        "hybrid"
        if has_app_router and has_pages_router
        else "app-router"
        if has_app_router
        else "pages-router"
        if has_pages_router
        else "unknown"
    )
    root_layout = workspace_exists_any(
        root,
        record,
        ["app/layout.tsx", "app/layout.jsx", "src/app/layout.tsx", "src/app/layout.jsx"],
    )
    root_layout_text = safe_read_text(root / root_layout) if root_layout else ""
    typecheck_script = str(scripts.get(pick_script(record, ["typecheck", "type-check"]) or "") or "")

    import_counts = detect_import_inventory(
        base,
        {
            "next/font": r"""from\s+["']next/font""",
            "next/image": r"""from\s+["']next/image["']""",
            "next/link": r"""from\s+["']next/link["']""",
            "next/navigation": r"""from\s+["']next/navigation["']""",
            "next/server": r"""from\s+["']next/server["']""",
            "next/cache": r"""from\s+["']next/cache["']""",
        },
    )
    enabled_imports = [label for label, count in import_counts.items() if count > 0]
    route_handlers = [path for path in app_files if path.endswith("/route.ts") or path.endswith("/route.js")]
    static_export_enabled = 'output: "export"' in next_config_text or "output: 'export'" in next_config_text
    custom_image_loader = (
        "loaderFile" in next_config_text
        or 'loader: "custom"' in next_config_text
        or "loader: 'custom'" in next_config_text
    )
    root_dynamic_error = (
        'export const dynamic = "error"' in root_layout_text
        or "export const dynamic = 'error'" in root_layout_text
    )

    posture = [
        f"owner workspace: `{workspace_reference(record)}`",
        f"router mode: `{router_mode}`",
        f"next config: `{next_config or 'not found'}`",
        f"static export enabled: `{'yes' if static_export_enabled else 'no'}`",
        f"custom image loader: `{'yes' if custom_image_loader else 'no'}`",
        f"turbopack config present: `{'yes' if 'turbopack' in next_config_text else 'no'}`",
        f"cacheComponents enabled: `{'yes' if 'cacheComponents' in next_config_text else 'no'}`",
        f"typedRoutes enabled: `{'yes' if 'typedRoutes' in next_config_text else 'no'}`",
        f"root layout dynamic=error: `{'yes' if root_dynamic_error else 'no'}`",
        f"`next typegen` in typecheck script: `{'yes' if 'next typegen' in typecheck_script else 'no'}`",
        f"proxy file present: `{'yes' if workspace_exists_any(root, record, ['proxy.ts', 'proxy.js', 'src/proxy.ts', 'src/proxy.js']) else 'no'}`",
        f"middleware file present: `{'yes' if workspace_exists_any(root, record, ['middleware.ts', 'middleware.js', 'src/middleware.ts', 'src/middleware.js']) else 'no'}`",
        f"route handlers present: `{'yes' if route_handlers else 'no'}`",
    ]
    inventory = [
        f"App Router files detected: `{len(app_files)}`",
        f"App Router file sample: `{', '.join(app_files[:6]) or 'none'}`",
        f"Next import surfaces detected: `{', '.join(enabled_imports) or 'none'}`",
        f"Import counts: `{', '.join(f'{label}={count}' for label, count in import_counts.items())}`",
    ]
    return {
        "Repo posture": posture,
        "Surface inventory": inventory,
    }


def detect_frameworks(root: Path) -> dict[str, Any]:
    """Detect broad framework and package-family signals from the repo."""
    frameworks: set[str] = set()
    records = workspace_manifest_records(root)
    dependencies_seen: set[str] = set()

    for record in records:
        dependencies_seen.update(record["dependency_names"])

    for package, framework in FRAMEWORK_PACKAGES.items():
        if package in dependencies_seen:
            frameworks.add(framework)

    if any(dep.startswith("@radix-ui/react-") for dep in dependencies_seen):
        frameworks.add("radix-ui")

    components_paths = iter_named_files(root, "components.json")
    if components_paths:
        frameworks.add("shadcn")

    return {
        "frameworks_detected": sorted(frameworks),
        "package_json_paths": [record["package_json_path"] for record in records],
        "components_json_paths": [repo_relative_path(root, path) for path in components_paths],
        "workspace_package_roots": [record["workspace_path"] for record in records],
        "workspace_package_names": [
            name for name in (record.get("package_name") for record in records) if isinstance(name, str)
        ],
        "is_monorepo": len(records) > 1,
    }


def detect_repo_context(root: Path) -> dict[str, Any]:
    """Build the repo context payload used by manifest bootstrap."""
    package_manager = detect_package_manager(root)
    frameworks = detect_frameworks(root)
    context = {
        "repo_root": str(root),
        **package_manager,
        **frameworks,
        "command_variables": command_family_variables(package_manager["package_manager"]),
    }
    return context


def unique_list(items: list[str]) -> list[str]:
    """Return items with stable first-seen ordering preserved."""
    return list(dict.fromkeys(items))


def recursive_merge(base: dict[str, Any], override: dict[str, Any]) -> dict[str, Any]:
    """Recursively merge dictionaries."""
    result = dict(base)
    for key, value in override.items():
        if key in result and isinstance(result[key], dict) and isinstance(value, dict):
            result[key] = recursive_merge(result[key], value)
        else:
            result[key] = value
    return result


def family_override_dir(script_path: str | Path) -> Path:
    """Return the family override directory for the current skill."""
    return Path(script_path).resolve().parents[1] / "references" / "family-overrides"


def available_override_paths(script_path: str | Path) -> list[Path]:
    """List all override YAML files."""
    overrides = family_override_dir(script_path)
    return sorted(path for path in overrides.glob("*.yaml") if path.is_file())
