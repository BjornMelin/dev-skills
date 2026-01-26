from __future__ import annotations

import argparse
import json
import os
import re
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Iterable, Literal


Severity = Literal["high", "medium", "low", "info"]


@dataclass(frozen=True)
class Finding:
    severity: Severity
    file: str
    message: str
    hint: str


IGNORE_DIR_NAMES = {
    ".git",
    ".hg",
    ".svn",
    ".venv",
    "venv",
    "node_modules",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    "dist",
    "build",
    ".tox",
    ".idea",
    ".vscode",
    "opensrc",
    "vendor",
    "third_party",
    "external",
}


def _walk_files(root: Path) -> Iterable[Path]:
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = [d for d in dirnames if d not in IGNORE_DIR_NAMES]
        base = Path(dirpath)
        for filename in filenames:
            yield base / filename


def _is_dockerfile_name(name: str) -> bool:
    return name in {"Dockerfile", "Containerfile"} or name.startswith("Dockerfile.") or name.startswith("Containerfile.")


def _is_compose_name(name: str) -> bool:
    lower = name.lower()
    if lower in {"compose.yml", "compose.yaml", "docker-compose.yml", "docker-compose.yaml"}:
        return True
    if lower.startswith("docker-compose.") and (lower.endswith(".yml") or lower.endswith(".yaml")):
        return True
    return False


FROM_RE = re.compile(r"^\s*FROM\s+([^\s]+)", re.IGNORECASE)
INSTRUCTION_RE = re.compile(r"^\s*(?P<ins>CMD|ENTRYPOINT|ARG|ENV)\b(?P<rest>.*)$", re.IGNORECASE)


def _looks_like_secret_name(name: str) -> bool:
    upper = name.upper()
    secret_fragments = ("SECRET", "TOKEN", "PASSWORD", "PASSWD", "API_KEY", "ACCESS_KEY", "PRIVATE_KEY", "CREDENTIAL")
    return any(f in upper for f in secret_fragments)


def audit_dockerfile(path: Path, root: Path) -> list[Finding]:
    findings: list[Finding] = []
    rel = str(path.relative_to(root))
    text = path.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()

    from_images: list[str] = []
    seen_cmd_or_entrypoint_json_issue = False
    apt_update_seen = False
    apt_lists_cleaned = False

    for line in lines:
        m = FROM_RE.match(line)
        if m:
            from_images.append(m.group(1))
        if re.search(r"rm\s+-rf\s+/var/lib/apt/lists", line):
            apt_lists_cleaned = True
        if re.search(r"apt-get\s+update", line):
            apt_update_seen = True

        im = INSTRUCTION_RE.match(line)
        if im:
            ins = im.group("ins").upper()
            rest = im.group("rest").strip()
            if ins in {"CMD", "ENTRYPOINT"}:
                # Best practice: JSON array form, helps signal handling and avoids shell pitfalls.
                if rest and not rest.lstrip().startswith("["):
                    seen_cmd_or_entrypoint_json_issue = True
            if ins in {"ARG", "ENV"}:
                # Heuristic: flag obvious secret names being set in Dockerfile.
                # - ARG NAME=... or ARG NAME
                # - ENV NAME=value ... (only checks first assignment)
                token = rest.split()[0] if rest else ""
                name = token.split("=", 1)[0].strip()
                if name and _looks_like_secret_name(name):
                    findings.append(
                        Finding(
                            severity="medium",
                            file=rel,
                            message=f"{ins} sets a likely secret variable '{name}' (heuristic).",
                            hint="Manual review: if this is a secret, do not bake it into the image. Use BuildKit secret mounts or runtime secrets.",
                        )
                    )

    for img in from_images:
        # Image pinning checks (heuristic)
        if ":" not in img and "@" not in img:
            findings.append(
                Finding(
                    severity="medium",
                    file=rel,
                    message=f"Base image '{img}' has no explicit tag.",
                    hint="Pin to a tag (and optionally digest) for reproducibility; avoid implicit 'latest'.",
                )
            )
        if img.endswith(":latest") or ":latest@" in img:
            findings.append(
                Finding(
                    severity="high",
                    file=rel,
                    message=f"Base image '{img}' uses the 'latest' tag.",
                    hint="Avoid 'latest' in production. Pin to an explicit version (and optionally digest).",
                )
            )

    if re.search(r"^\s*ADD\s+", text, flags=re.IGNORECASE | re.MULTILINE):
        findings.append(
            Finding(
                severity="low",
                file=rel,
                message="Uses ADD instruction.",
                hint="Prefer COPY unless you explicitly need ADD features (tar auto-extract/URL).",
            )
        )

    if re.search(r"curl\s+[^|]*\|\s*(sh|bash)", text):
        findings.append(
            Finding(
                severity="high",
                file=rel,
                message="Pipes curl output to a shell.",
                hint="Download to a file, verify checksum/signature, then execute.",
            )
        )

    if re.search(r"^\s*USER\s+root\s*$", text, flags=re.IGNORECASE | re.MULTILINE):
        findings.append(
            Finding(
                severity="medium",
                file=rel,
                message="Explicitly sets USER root.",
                hint="Prefer a non-root runtime user; only use root for build steps when required.",
            )
        )

    if not re.search(r"^\s*USER\s+", text, flags=re.IGNORECASE | re.MULTILINE):
        findings.append(
            Finding(
                severity="medium",
                file=rel,
                message="No USER set in Dockerfile.",
                hint="Set a non-root USER in the final runtime stage (or justify why root is required).",
            )
        )

    if re.search(r"^\s*HEALTHCHECK\s+", text, flags=re.IGNORECASE | re.MULTILINE) is None:
        findings.append(
            Finding(
                severity="info",
                file=rel,
                message="No HEALTHCHECK defined.",
                hint="Consider adding HEALTHCHECK or enforce health via orchestrator (Compose/K8s) depending on needs.",
            )
        )

    if "apt-get install" in text and "--no-install-recommends" not in text:
        findings.append(
            Finding(
                severity="low",
                file=rel,
                message="apt-get install without --no-install-recommends (heuristic).",
                hint="Use --no-install-recommends and remove apt lists to reduce image size.",
            )
        )

    if re.search(r"^\s*COPY\s+\.?\s*\.\s*$", text, flags=re.IGNORECASE | re.MULTILINE):
        findings.append(
            Finding(
                severity="low",
                file=rel,
                message="COPY . . detected (heuristic).",
                hint="Ensure a strong .dockerignore; consider copying only needed files to improve cache and reduce leaks.",
            )
        )

    if seen_cmd_or_entrypoint_json_issue:
        findings.append(
            Finding(
                severity="low",
                file=rel,
                message="CMD/ENTRYPOINT not using JSON array form (heuristic).",
                hint="Prefer JSON array syntax for CMD/ENTRYPOINT to improve signal handling and avoid shell pitfalls.",
            )
        )

    if apt_update_seen and not apt_lists_cleaned:
        findings.append(
            Finding(
                severity="low",
                file=rel,
                message="apt-get update detected without cleaning /var/lib/apt/lists (heuristic).",
                hint="After apt installs, clean apt lists to reduce layer size: rm -rf /var/lib/apt/lists/*",
            )
        )

    if re.search(r"apt-get\s+upgrade", text):
        findings.append(
            Finding(
                severity="low",
                file=rel,
                message="apt-get upgrade detected (heuristic).",
                hint="Avoid apt-get upgrade in images; prefer rebuilding regularly and pinning base images/deps.",
            )
        )

    return findings

def audit_compose_file(path: Path, root: Path) -> list[Finding]:
    findings: list[Finding] = []
    rel = str(path.relative_to(root))
    text = path.read_text(encoding="utf-8", errors="replace")

    def flag(pattern: str, severity: Severity, message: str, hint: str) -> None:
        if re.search(pattern, text, flags=re.IGNORECASE | re.MULTILINE):
            findings.append(Finding(severity=severity, file=rel, message=message, hint=hint))

    flag(
        r"^\s*privileged:\s*true\s*$",
        "high",
        "Compose uses privileged: true.",
        "Avoid privileged. Use capabilities/permissions narrowly; consider seccomp/apparmor/no-new-privileges.",
    )
    flag(
        r"^\s*network_mode:\s*host\s*$",
        "high",
        "Compose uses network_mode: host.",
        "Avoid host networking unless required; prefer explicit networks and published ports.",
    )
    flag(
        r"^\s*pid:\s*host\s*$",
        "high",
        "Compose uses pid: host.",
        "Avoid pid namespace sharing unless required; it weakens isolation.",
    )
    flag(
        r"^\s*user:\s*[\"']?0([\"']|\s|$)",
        "medium",
        "Compose sets user: 0 (root).",
        "Prefer a non-root user and set filesystem permissions appropriately.",
    )
    flag(
        r"/var/run/docker\.sock",
        "high",
        "Compose mounts the Docker socket.",
        "Avoid mounting docker.sock; it grants effectively root-on-host. Use a build service or scoped API if needed.",
    )
    flag(
        r"^\s*-\s*/:\s*/",
        "high",
        "Compose mounts host root filesystem (/:/...).",
        "Avoid broad host mounts; scope to specific directories and consider read-only mounts.",
    )
    flag(
        r"^\s*cap_add:\s*$",
        "info",
        "Compose uses cap_add (manual review needed).",
        "Ensure only minimal capabilities are added; prefer drop all + add a tiny set if required.",
    )

    if "healthcheck:" not in text:
        findings.append(
            Finding(
                severity="info",
                file=rel,
                message="No healthcheck section found (heuristic).",
                hint="Add healthchecks for critical services or document how health is determined.",
            )
        )

    return findings


def main() -> int:
    parser = argparse.ArgumentParser(description="Heuristic audit for Dockerfiles and Compose files.")
    parser.add_argument("--root", type=Path, default=Path("."), help="Repo root (default: .)")
    parser.add_argument("--json", action="store_true", help="Output JSON")
    args = parser.parse_args()

    root = args.root.resolve()
    dockerfiles: list[Path] = []
    compose_files: list[Path] = []
    for p in _walk_files(root):
        if not p.is_file():
            continue
        if _is_dockerfile_name(p.name):
            dockerfiles.append(p)
        if _is_compose_name(p.name):
            compose_files.append(p)

    findings: list[Finding] = []
    for df in sorted(dockerfiles):
        findings.extend(audit_dockerfile(df, root))
    for cf in sorted(compose_files):
        findings.extend(audit_compose_file(cf, root))

    if dockerfiles and not (root / ".dockerignore").exists():
        findings.append(
            Finding(
                severity="medium",
                file=".dockerignore",
                message="No .dockerignore present, but Dockerfiles exist.",
                hint="Add a .dockerignore to avoid leaking secrets and to improve build performance.",
            )
        )

    if args.json:
        print(json.dumps([asdict(f) for f in findings], indent=2, sort_keys=True))
        return 0

    if not findings:
        print("Docker Architect · Audit: no findings (heuristic).")
        return 0

    print("Docker Architect · Audit findings (heuristic)")
    for f in findings:
        print(f"- [{f.severity}] {f.file}: {f.message}")
        print(f"  hint: {f.hint}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
