#!/usr/bin/env python3
"""Render the Codex subagent catalog."""

from __future__ import annotations

import json
import re
import textwrap
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AGENTS_ROOT = ROOT / "agents"
DEFAULT_LOCAL_ROLES = ROOT / "roles.local.json"
SAFE_SLUG_RE = re.compile(r"^[a-z0-9][a-z0-9_-]*$")


@dataclass(frozen=True)
class Role:
    """Role source data for one rendered Codex subagent.

    Attributes:
        name: Snake-case subagent role name.
        description: Short role description used by Codex for routing.
        effort: Model reasoning effort, such as medium, high, or max.
        sandbox: Sandbox mode for the rendered subagent.
        body: Role-specific developer instructions.
        family: Output family. Global roles render under agents/global.
        nicknames: Optional display nickname candidates.
        model: Model identifier for the role.
    """

    name: str
    description: str
    effort: str
    sandbox: str
    body: str
    family: str = "global"
    nicknames: tuple[str, str, str] | None = None
    model: str = "gpt-5.6-sol"


COMMON_BOUNDARY = """\
Do not spawn nested subagents or broaden the assigned scope.
Treat the parent prompt as the authority for task priority only.
Safety, privacy, and scope constraints are non-overridable.
Redact secrets, tokens, credentials, and private personal data from outputs.
Keep outputs concise, evidence-first, and scoped to the assigned task.
"""

RETURN_STANDARD = """\
Return format:
- Status
- Evidence
- Files inspected/changed
- Commands run
- Findings
- Risks/blockers
"""

RETURN_RESEARCH = """\
Return format:
- Status
- Sources hydrated
- Claims
- Provider limits
- Privacy notes
- Recommended next verification
- Risks/blockers
"""

RESEARCH_CONTRACT_AGENT_NAMES = {
    "citation_auditor",
    "context7_researcher",
    "deep_researcher",
    "github_researcher",
    "openai_docs_researcher",
    "source_validator",
}


def role(
    name: str,
    description: str,
    effort: str,
    sandbox: str,
    body: str,
    family: str = "global",
    nicknames: tuple[str, str, str] | None = None,
    model: str = "gpt-5.6-sol",
) -> Role:
    """Create a normalized role source record.

    Args:
        name: Snake-case subagent role name.
        description: Short role description.
        effort: Model reasoning effort.
        sandbox: Sandbox mode.
        body: Role-specific instruction body.
        family: Output family for the role.
        nicknames: Optional display nickname candidates.
        model: Model identifier for the role.

    Returns:
        A Role with stripped instruction body text.
    """

    return Role(
        name=name,
        description=description,
        effort=effort,
        sandbox=sandbox,
        body=body.strip(),
        family=family,
        nicknames=nicknames,
        model=model,
    )


def require_string(config: dict[str, object], key: str, *, source: Path) -> str:
    """Read a required string field from a local role config object.

    Args:
        config: Parsed local role object.
        key: Field name to read.
        source: Manifest path used in error messages.

    Returns:
        The non-empty string field value.

    Raises:
        SystemExit: If the field is missing or not a non-empty string.
    """

    value = config.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"local role requires non-empty string {key}: {source}")
    return value


def require_slug(config: dict[str, object], key: str, *, source: Path) -> str:
    """Read a required safe slug field from a local role config object.

    Args:
        config: Parsed local role object.
        key: Field name to read.
        source: Manifest path used in error messages.

    Returns:
        The validated slug field value.

    Raises:
        SystemExit: If the field is missing or not a safe slug.
    """

    value = require_string(config, key, source=source)
    if not SAFE_SLUG_RE.fullmatch(value):
        raise SystemExit(f"local role {key} must be a safe slug: {source}")
    return value


def load_local_roles(path: Path = DEFAULT_LOCAL_ROLES) -> list[Role]:
    """Load ignored local-only role definitions when present.

    Args:
        path: Local role manifest path.

    Returns:
        Local role records, or an empty list when the manifest is absent.

    Raises:
        SystemExit: If the manifest is malformed.
    """

    if not path.exists():
        return []
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise SystemExit(f"invalid local role manifest {path}: {exc}") from exc
    raw_roles = payload.get("roles")
    if not isinstance(raw_roles, list):
        raise SystemExit(f"local role manifest must contain a roles array: {path}")

    roles: list[Role] = []
    seen: set[str] = set()
    for raw_role in raw_roles:
        if not isinstance(raw_role, dict):
            raise SystemExit(f"local role entries must be objects: {path}")
        name = require_slug(raw_role, "name", source=path)
        if name in seen:
            raise SystemExit(f"duplicate local role name {name}: {path}")
        seen.add(name)
        nicknames_value = raw_role.get("nicknames")
        nicknames: tuple[str, str, str] | None = None
        if nicknames_value is not None:
            if (
                not isinstance(nicknames_value, list)
                or len(nicknames_value) != 3
                or not all(isinstance(item, str) and item for item in nicknames_value)
            ):
                raise SystemExit(f"local role {name} nicknames must be three strings: {path}")
            nicknames = (nicknames_value[0], nicknames_value[1], nicknames_value[2])
        roles.append(
            role(
                name,
                require_string(raw_role, "description", source=path),
                require_string(raw_role, "effort", source=path),
                require_string(raw_role, "sandbox", source=path),
                require_string(raw_role, "body", source=path),
                require_slug(raw_role, "family", source=path),
                nicknames,
                model=(
                    require_string(raw_role, "model", source=path)
                    if "model" in raw_role
                    else "gpt-5.6-sol"
                ),
            )
        )
    return roles


GLOBAL_ROLES: list[Role] = [
    role(
        "guidance_mapper",
        "Read-only mapper for AGENTS.md, CLAUDE.md, README, and scoped project guidance relevant to a change.",
        "medium",
        "read-only",
        """Find guidance files relevant to the assigned paths, PR diff, or repo task.
Return only applicable rules and their source paths; avoid copying large guidance blocks.
Do not review code quality or propose implementation changes unless the parent asks.""",
        model="gpt-5.6-terra",
    ),
    role(
        "repo_explorer",
        "Read-only codebase explorer for bounded evidence gathering before changes without shadowing Codex built-ins.",
        "high",
        "read-only",
        """Answer the exact codebase question with file and symbol evidence.
Prefer fast search and targeted reads over broad scans.
Do not propose broad refactors unless the parent asks for recommendations.""",
        model="gpt-5.6-terra",
    ),
    role(
        "docs_researcher",
        "Read-only documentation researcher for official API, framework, and version behavior.",
        "high",
        "read-only",
        """Verify APIs, options, migrations, and version-specific behavior from authoritative documentation.
Prefer official docs, source repositories, package source, and primary changelogs.
Clearly mark uncertainty when docs and observed code disagree.""",
        model="gpt-5.6-terra",
    ),
    role(
        "env_validator",
        "Read-only environment and configuration validator for required variables, secrets wiring, and deployment config.",
        "medium",
        "read-only",
        """Validate only the assigned environment, deployment, or configuration surface.
Identify required variables, missing examples, unsafe defaults, secret leakage risks, and inconsistent config names.
Report only names, presence, and wiring evidence; never print secret values.""",
        model="gpt-5.6-terra",
    ),
    role(
        "ci_triager",
        "Read-only CI triager for failing checks, logs, workflow contracts, and likely fixes.",
        "medium",
        "read-only",
        """Triage the assigned CI failure from live check status, logs, workflow files, and repo scripts.
Prefer exact failure evidence over speculation.
Separate infrastructure flakes from deterministic code failures.""",
    ),
    role(
        "citation_auditor",
        "Read-only auditor for claim-to-source mapping, source freshness, citation quality, and unsupported research conclusions.",
        "medium",
        "read-only",
        """Audit research output for citation quality and unsupported claims.
Check that material claims map to primary or high-quality sources and current evidence.
Return corrected confidence labels and required fixes before publication.""",
    ),
    role(
        "context7_researcher",
        "Read-only Context7 specialist for current library documentation lookups through Context7 MCP or ctx7 CLI.",
        "high",
        "read-only",
        """Research library, framework, SDK, API, and CLI documentation through Context7 when available.
Use version-specific library IDs when the parent or repo pins a version.
If Context7 lacks coverage, report the gap and recommend official docs, source, or package inspection.""",
        model="gpt-5.6-terra",
    ),
    role(
        "dependency_researcher",
        "Read-only dependency researcher for package docs, release notes, source internals, and upgrade risk.",
        "medium",
        "read-only",
        """Research only the assigned dependency, version range, or package behavior.
Prefer official release notes and docs first; inspect package source when behavior or migration risk depends on implementation.
Separate documented API changes from source-inferred behavior.""",
    ),
    role(
        "docs_auditor",
        "Read-only docs auditor for stale, missing, duplicated, or misleading repository documentation.",
        "medium",
        "read-only",
        """Audit only the assigned documentation surfaces and related code/workflow truth.
Find stale instructions, missing validation steps, duplicated authority, and misleading status claims.
Prefer one canonical documentation owner per concern.""",
    ),
    role(
        "github_researcher",
        "Read-only GitHub specialist for repository, code, issue, pull request, release, changelog, and package-manifest evidence.",
        "high",
        "read-only",
        """Research GitHub evidence only.
Hydrate search hits before citing files, issue threads, PRs, releases, tags, compare ranges, or manifests.
Report API limitations, incomplete results, rate limits, and unsearched surfaces.""",
        model="gpt-5.6-terra",
    ),
    role(
        "history_reviewer",
        "Read-only reviewer that uses git history and blame to validate whether changed code violates existing intent.",
        "medium",
        "read-only",
        """Use git history, blame, and nearby commits only for the assigned files or symbols.
Identify issues visible because code violates historical intent, previous fixes, or prior review context.
Do not flag pre-existing issues unless the current change reintroduces or worsens them.""",
    ),
    role(
        "implementation_worker",
        "Scoped implementation worker for narrow fixes with explicit file ownership.",
        "medium",
        "workspace-write",
        """Implement only the assigned scoped change and owned files.
You are not alone in the codebase. Do not revert edits made by others.
Keep the diff minimal and reviewable, and run the most relevant focused validation you can.
Do not stage, commit, push, or modify unrelated files unless explicitly instructed.""",
    ),
    role(
        "openai_docs_researcher",
        "Read-only researcher for current official OpenAI and Codex documentation.",
        "high",
        "read-only",
        """Research only official OpenAI sources unless the parent explicitly asks for broader context.
Check current docs before answering model, API, Codex, or subagent behavior questions.
Separate confirmed documentation from inference and cite exact source URLs when available.""",
        model="gpt-5.6-terra",
    ),
    role(
        "performance_reviewer",
        "Read-only performance reviewer for obvious algorithmic, rendering, database, bundle, and IO bottlenecks.",
        "medium",
        "read-only",
        """Review only the assigned performance-sensitive code path or diff.
Prioritize high-impact bottlenecks with concrete evidence or clear complexity/runtime reasoning.
Separate measured evidence from likely risks and avoid speculative rewrites.""",
    ),
    role(
        "release_validator",
        "Read-only release validator for changelog, versioning, tags, packaging, and publish readiness checks.",
        "medium",
        "read-only",
        """Validate only the assigned release, packaging, or publish surface.
Check version consistency, changelog truth, generated artifacts, release notes, tag expectations, and publish gates.
Report exact commands to run; do not perform destructive publish or tag operations.""",
    ),
    role(
        "reviewer",
        "Read-only reviewer focused on correctness, security, regressions, and missing tests.",
        "medium",
        "read-only",
        """Review code like an owner.
Prioritize correctness, security, behavioral regressions, data loss risks, and missing tests.
Lead with concrete findings ordered by severity and grounded in file, symbol, command, or reproduction evidence.""",
    ),
    role(
        "runtime_bug_reviewer",
        "Read-only runtime bug reviewer for null safety, async races, lifecycle leaks, and error handling.",
        "medium",
        "read-only",
        """Review only the assigned runtime behavior surface.
Prioritize null crashes, async races, stale state, resource leaks, missing cleanup, and swallowed errors.
Use tests, logs, and reproduction evidence when available.""",
    ),
    role(
        "shallow_bug_reviewer",
        "Read-only high-signal reviewer for obvious diff-level bugs and regressions.",
        "medium",
        "read-only",
        """Review only the assigned diff or changed lines.
Flag high-confidence issues: compile failures, undefined references, clear logic errors, data loss, or deterministic runtime failures.
Do not flag style, taste, broad architecture, or speculative issues.""",
    ),
    role(
        "source_validator",
        "Read-only package/source implementation validator for verifying docs claims against actual repository or package source.",
        "medium",
        "read-only",
        """Validate claims against source code, package contents, releases, and version diffs.
Prefer exact versions from lockfiles, package manifests, tags, or release refs.
Report exact files, symbols, versions, and refs inspected.""",
    ),
    role(
        "test_runner",
        "Validation worker that runs focused tests and reports command-level evidence without editing source.",
        "medium",
        "workspace-write",
        """Run only validation commands assigned by the parent or the smallest relevant repo-native checks.
Do not edit source files; it is acceptable for test tools to write caches, coverage, or temporary files.
Capture exact commands, pass/fail status, key failure lines, and likely owner files.""",
    ),
    role(
        "ui_debugger",
        "UI debugger for reproducing browser or frontend regressions and reporting actionable evidence.",
        "medium",
        "workspace-write",
        """Reproduce the assigned UI or frontend issue with available browser and app tooling.
Capture exact steps, console or network evidence, screenshots when useful, and likely owning files.
Do not edit application code unless the parent explicitly assigns an implementation task.""",
    ),
    role(
        "deep_researcher",
        "Lead read-only researcher for multi-source, cited, current investigations with claim ledgers and freshness checks.",
        "high",
        "read-only",
        """Lead deep research, not implementation.
Use official docs, Context7, GitHub/source inspection, package source, and web search only as scoped by the parent.
Treat search hits as leads until hydrated into source records and produce claim-level confidence.""",
    ),
    role(
        "false_positive_validator",
        "Read-only validator that scores candidate findings and filters weak or stale issues.",
        "max",
        "read-only",
        """Validate only the candidate findings provided by the parent.
Score each candidate from 0 to 100 using evidence, impact, current-code truth, and whether the change introduced it.
Reject stale, pre-existing, style-only, and unverified claims unless the parent sets a different threshold.""",
        model="gpt-5.6-terra",
    ),
    role(
        "security_reviewer",
        "Read-only security reviewer for authentication, authorization, injection, secrets, and data exposure risks.",
        "high",
        "read-only",
        """Review only assigned security-sensitive files, flows, diffs, or claims.
Prioritize exploitable authentication, authorization, injection, secret exposure, data leakage, and insecure defaults.
Do not perform destructive testing, credential use, or network probing unless explicitly scoped.""",
    ),
    role(
        "root_cause_investigator",
        "Read-only root-cause investigator for hard failures, regressions, flaky behavior, and conflicting evidence.",
        "high",
        "read-only",
        """Find the root cause before recommending fixes.
Trace symptoms through code paths, configuration, runtime logs, tests, and recent changes when available.
Separate proximate failures from underlying causes and list the minimum verification that would disprove the conclusion.""",
    ),
    role(
        "architect_reviewer",
        "Read-only architecture reviewer for subsystem boundaries, ownership drift, and high-impact design decisions.",
        "high",
        "read-only",
        """Review architecture-level changes, subsystem boundaries, duplicate ownership, and contract drift.
Prefer established repo patterns and library/platform leverage over new abstractions.
Surface decision tradeoffs, blast radius, and migration risk with evidence.""",
    ),
    role(
        "pr_shepherd",
        "Read-only PR shepherd for review-to-ship loops, unresolved threads, CI state, merge blockers, and closure evidence.",
        "medium",
        "read-only",
        """Inspect the assigned pull request, review state, CI status, branch delta, and merge blockers.
Distinguish code fixed, checks passing, review threads resolved, and merge-ready state.
Do not resolve threads, push, merge, or comment unless the parent explicitly assigns that operation.""",
    ),
    role(
        "commit_planner",
        "Read-only conventional-commit planner for dirty trees and semantically reviewable staging lanes.",
        "high",
        "read-only",
        """Analyze the dirty tree and propose semantic conventional-commit groups.
Keep unrelated edits separate and identify files that should not be staged together.
Do not stage, commit, or rewrite history.""",
    ),
    role(
        "docs_aligner",
        "Read-only documentation alignment reviewer for code, workflow, contract, and user-guide drift.",
        "medium",
        "read-only",
        """Compare changed behavior against README, docs, AGENTS guidance, runbooks, ADRs, and specs in scope.
Identify stale, missing, duplicated, or misleading docs and recommend one canonical owner per fact.
Do not edit docs unless the parent assigns an implementation task to a worker.""",
    ),
    role(
        "dependency_upgrade_planner",
        "Read-only dependency upgrade planner for safe package bumps, changelog/source review, and verification lanes.",
        "high",
        "read-only",
        """Plan dependency upgrades from lockfiles, manifests, release notes, package source, and repo usage.
Separate safe patch/minor bumps from migration work and security-driven hard cuts.
Return exact verification commands and rollback risks.""",
    ),
]


PLATFORM_ROLES: list[Role] = [
    role(
        "nextjs_reviewer",
        "Read-only Next.js reviewer for App Router, routing, caching, server actions, middleware/proxy, and build behavior.",
        "medium",
        "read-only",
        "Review Next.js-specific code and configuration using current official docs or source when behavior may have changed.",
    ),
    role(
        "react_reviewer",
        "Read-only React reviewer for component structure, hooks, state ownership, rendering behavior, and accessibility regressions.",
        "medium",
        "read-only",
        "Review React-specific code paths for correctness, lifecycle issues, performance traps, and test gaps.",
    ),
    role(
        "expo_reviewer",
        "Read-only Expo reviewer for Expo Router, native configuration, EAS workflows, OTA/runtime version, and mobile build risk.",
        "medium",
        "read-only",
        "Review Expo and React Native surfaces using installed SDK metadata, official docs, and repo-native validation contracts.",
    ),
    role(
        "convex_reviewer",
        "Read-only Convex reviewer for schema, functions, indexes, authz, components, and backend contract risk.",
        "medium",
        "read-only",
        "Review Convex code for validator/schema drift, index-backed access, authz enforcement, runtime constraints, and component fit.",
    ),
    role(
        "clerk_reviewer",
        "Read-only Clerk reviewer for auth/session flows, organization context, redirects, webhooks, and browser/mobile auth behavior.",
        "medium",
        "read-only",
        "Review Clerk integration surfaces using current official docs and repo-specific auth boundaries provided by the parent.",
    ),
    role(
        "vercel_reviewer",
        "Read-only Vercel reviewer for deployments, functions, routing, env vars, build output, and release pipeline risk.",
        "medium",
        "read-only",
        "Review Vercel-specific config and deployment behavior using current official docs and observed repo scripts.",
    ),
    role(
        "openai_api_reviewer",
        "Read-only OpenAI API reviewer for model selection, Responses API usage, tool calling, structured output, and Codex behavior.",
        "medium",
        "read-only",
        "Review OpenAI API or Codex usage against current official OpenAI documentation and clearly separate inference from documented behavior.",
    ),
    role(
        "bun_ts_reviewer",
        "Read-only Bun and TypeScript reviewer for package-manager policy, scripts, tests, runtime APIs, and strict typing.",
        "medium",
        "read-only",
        "Review Bun and TypeScript surfaces for repo policy compliance, script/runtime behavior, dependency usage, and type-safety risk.",
    ),
    role(
        "python_uv_reviewer",
        "Read-only Python and uv reviewer for dependency resolution, lockfiles, packaging, tests, and runtime compatibility.",
        "medium",
        "read-only",
        "Review Python and uv surfaces for lockfile integrity, environment reproducibility, package metadata, and test/runtime risk.",
    ),
]


OVERLAY_ROLES: list[Role] = [
    role(
        "docmind_dependency_safety_reviewer",
        "DocMind dependency safety reviewer for Dependabot, security bumps, "
        "uv locks, release notes, and source compatibility.",
        "high",
        "read-only",
        "Assess DocMind dependency changes from live diff, uv.lock, upstream "
        "release notes/source, CI status, and runtime usage before declaring "
        "safe.",
        "docmind",
    ),
    role(
        "docmind_python_runtime_reviewer",
        "DocMind Python runtime reviewer for Streamlit/runtime behavior, "
        "model loading, Apple MPS parity, and test taxonomy.",
        "medium",
        "read-only",
        "Review DocMind Python runtime changes for compatibility, optional "
        "dependency behavior, device parity, and focused pytest coverage.",
        "docmind",
    ),
    role(
        "docmind_ci_triager",
        "DocMind CI triager for GitHub Actions, uv frozen installs, docs "
        "lint, tests, and release automation failures.",
        "medium",
        "read-only",
        "Start from live CI evidence and committed workflow files. Separate "
        "frozen CI installs from release lock-refresh automation.",
        "docmind",
    ),
    role(
        "docmind_docs_release_auditor",
        "DocMind docs/release auditor for Release Please, changelogs, "
        "markdown lint, worklogs, and rendered docs behavior.",
        "medium",
        "read-only",
        "Validate DocMind docs and release automation against committed "
        "configuration, rendered behavior when requested, and parseable "
        "SemVer release note rules.",
        "docmind",
    ),
    role(
        "docmind_model_source_validator",
        "DocMind model/source validator for Transformers, SigLIP, GGUF, "
        "image safety, and upstream implementation claims.",
        "medium",
        "read-only",
        "Validate model/runtime claims against installed versions, source, "
        "and DocMind loader ownership. Preserve custom-model revision "
        "semantics unless parent says otherwise.",
        "docmind",
    ),
    role(
        "skill_package_validator",
        "Agent tooling skill package validator for AgentSkills metadata, "
        "packaging, quick validation, and install portability.",
        "medium",
        "read-only",
        "Validate skill packages against repo tooling, agents/openai.yaml "
        "metadata, quick_validate behavior, and install-relative path rules.",
        "tooling",
    ),
    role(
        "subagent_pack_reviewer",
        "Agent tooling subagent pack reviewer for TOML role catalogs, "
        "model/effort policy, safety contracts, and smoke readiness.",
        "high",
        "read-only",
        "Review Codex subagent packs against authoring guide, subspawn "
        "policy, validator constraints, and runtime smoke expectations.",
        "tooling",
    ),
    role(
        "mcp_tooling_reviewer",
        "Agent tooling MCP/source reviewer for research CLI, Context7/GitHub/"
        "source routing, and provider evidence contracts.",
        "medium",
        "read-only",
        "Review MCP/tooling changes for provider routing, evidence hydration, "
        "secret redaction, timeout behavior, and replayable source records.",
        "tooling",
    ),
    role(
        "agent_runtime_smoke_tester",
        "Agent tooling runtime smoke tester for Codex custom agents, spawn "
        "contracts, and representative live checks.",
        "medium",
        "workspace-write",
        "Run assigned non-destructive smoke commands, temporary projects, or "
        "Codex exec checks. Report exact commands and role responses.",
        "tooling",
    ),
]


PUBLIC_ROLES = [*GLOBAL_ROLES, *PLATFORM_ROLES, *OVERLAY_ROLES]


DISPLAY_WORDS = {
    "api": "API",
    "bun": "Bun",
    "ci": "CI",
    "clerk": "Clerk",
    "cli": "CLI",
    "codex": "Codex",
    "context7": "Context7",
    "convex": "Convex",
    "docmind": "DocMind",
    "docs": "Docs",
    "eas": "EAS",
    "expo": "Expo",
    "github": "GitHub",
    "ios": "iOS",
    "mcp": "MCP",
    "nextjs": "Nextjs",
    "openai": "OpenAI",
    "pr": "PR",
    "python": "Python",
    "sdk": "SDK",
    "ts": "TS",
    "ui": "UI",
    "uv": "uv",
    "vercel": "Vercel",
}


def title_from_name(name: str) -> str:
    """Convert a role name into a display title with known acronym casing.

    Args:
        name: Snake-case role name.

    Returns:
        Human-readable title text.
    """

    return " ".join(DISPLAY_WORDS.get(part, part.capitalize()) for part in name.split("_"))


def nicknames_for(role_spec: Role) -> tuple[str, str, str]:
    """Build nickname candidates for a role.

    Args:
        role_spec: Role source record.

    Returns:
        Three display nickname candidates.
    """

    if role_spec.nicknames is not None:
        return role_spec.nicknames
    title = title_from_name(role_spec.name)
    return (title, f"{title} Atlas", f"{title} Delta")


def toml_string(value: str) -> str:
    """Render a TOML basic string value.

    Args:
        value: String to escape.

    Returns:
        Escaped TOML basic string including quotes.
    """

    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def toml_multiline_string(value: str, *, width: int = 76) -> str:
    """Render a wrapped TOML multiline basic string.

    Args:
        value: String to wrap and escape.
        width: Maximum content width before wrapping.

    Returns:
        Escaped TOML multiline basic string.
    """

    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    lines = textwrap.wrap(escaped, width=width) or [""]
    return '"""\\\n' + "\n".join(lines) + '"""'


def toml_multiline_basic_string(value: str) -> str:
    """Render a TOML multiline basic string without reflowing lines.

    Args:
        value: String to escape and render.

    Returns:
        Escaped TOML multiline basic string.
    """

    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return '"""\n' + escaped + '\n"""'


def toml_string_array(values: tuple[str, ...]) -> str:
    """Render a TOML string array with one item per line.

    Args:
        values: String values to render.

    Returns:
        Multiline TOML array text.
    """

    rendered = [f"  {toml_string(value)}" for value in values]
    return "[\n" + ",\n".join(rendered) + "\n]"


def wrap_instruction_text(value: str, *, width: int = 76) -> str:
    """Wrap prose instruction lines while preserving list structure.

    Args:
        value: Instruction text to wrap.
        width: Maximum line width for prose.

    Returns:
        Wrapped instruction text.
    """

    lines: list[str] = []
    for line in value.splitlines():
        if not line or line.startswith("- "):
            lines.append(line)
            continue
        lines.extend(textwrap.wrap(line, width=width) or [""])
    return "\n".join(lines)


def render_role(role_spec: Role) -> str:
    """Render one role as a Codex custom subagent TOML document.

    Args:
        role_spec: Role source record.

    Returns:
        TOML text for the role.
    """

    return_contract = (
        RETURN_RESEARCH if role_spec.name in RESEARCH_CONTRACT_AGENT_NAMES else RETURN_STANDARD
    )
    instructions = "\n".join(
        [
            wrap_instruction_text(role_spec.body),
            wrap_instruction_text(COMMON_BOUNDARY.strip()),
            return_contract.strip(),
        ]
    )
    nicknames = toml_string_array(nicknames_for(role_spec))
    rendered_instructions = toml_multiline_basic_string(instructions)
    return f'''name = {toml_string(role_spec.name)}
description = {toml_multiline_string(role_spec.description)}
model = {toml_string(role_spec.model)}
model_reasoning_effort = {toml_string(role_spec.effort)}
sandbox_mode = {toml_string(role_spec.sandbox)}
nickname_candidates = {nicknames}
developer_instructions = {rendered_instructions}
'''


def target_dir(role_spec: Role) -> Path:
    """Return the generated directory for a role.

    Args:
        role_spec: Role source record.

    Returns:
        Directory path for the rendered TOML file.
    """

    if role_spec.family == "global":
        return AGENTS_ROOT / "global"
    return AGENTS_ROOT / "overlays" / role_spec.family


def clean_generated_dirs(local_roles: list[Role]) -> None:
    """Remove generated TOML files for public and supplied local roles.

    Args:
        local_roles: Local roles whose overlay directories are managed.
    """

    overlay_dirs = {
        AGENTS_ROOT / "overlays" / role_spec.family
        for role_spec in [*OVERLAY_ROLES, *local_roles]
    }
    for directory in [AGENTS_ROOT / "global", *sorted(overlay_dirs)]:
        if directory.exists():
            for path in sorted(directory.rglob("*.toml")):
                path.unlink()


def write_roles() -> int:
    """Render all public and local roles to TOML files.

    Returns:
        Number of rendered roles.
    """

    local_roles = load_local_roles()
    all_roles = [*PUBLIC_ROLES, *local_roles]
    clean_generated_dirs(local_roles)
    for role_spec in all_roles:
        directory = target_dir(role_spec)
        directory.mkdir(parents=True, exist_ok=True)
        (directory / f"{role_spec.name}.toml").write_text(render_role(role_spec), encoding="utf-8")
    return len(all_roles)


def write_catalog() -> None:
    """Write the public role catalog Markdown document."""

    lines = [
        "# Codex Subagent Catalog",
        "",
        "This pack is the source of truth for Bjorn's Codex custom subagents.",
        "It renders global agents under `agents/global` and project overlays under",
        "`agents/overlays/<repo>`.",
        "",
        "Runtime policy:",
        "",
        "- `gpt-5.6-terra` handles bounded retrieval and mechanical inventory;",
        "- `gpt-5.6-sol` handles judgment, implementation, planning, and synthesis;",
        "- `medium` is the default worker tier and `high` is reserved for complex decisions;",
        "- `gpt-5.6-terra` at `max` is reserved for independent adversarial validation;",
        "- routine roles do not use Sol `xhigh`, `max`, or `ultra`;",
        "- no nested subagents by default;",
        "- parent sessions own orchestration, waiting, synthesis, and final decisions;",
        "- read-only is default; workspace-write is limited to implementation, tests, UI/browser, and smoke runners.",
        "",
        "## Global Roles",
        "",
        "| Role | Model | Effort | Sandbox | Purpose |",
        "| --- | --- | --- | --- | --- |",
    ]
    for role_spec in [*GLOBAL_ROLES, *PLATFORM_ROLES]:
        lines.append(
            f"| `{role_spec.name}` | `{role_spec.model}` | `{role_spec.effort}` | `{role_spec.sandbox}` | {role_spec.description} |"
        )
    lines.extend(
        [
            "",
            "## Project Overlays",
            "",
            "| Repo family | Role | Model | Effort | Sandbox | Purpose |",
            "| --- | --- | --- | --- | --- | --- |",
        ]
    )
    for role_spec in OVERLAY_ROLES:
        lines.append(
            f"| `{role_spec.family}` | `{role_spec.name}` | `{role_spec.model}` | `{role_spec.effort}` | `{role_spec.sandbox}` | {role_spec.description} |"
        )
    lines.extend(
        [
            "",
            "## Routing Recipes",
            "",
            "PR review/remediation: `guidance_mapper` plus one reviewer lane, then",
            "`false_positive_validator`; use `test_runner` only after the parent chooses",
            "focused verification.",
            "",
            "High-stakes research: `deep_researcher`, `source_validator`, and",
            "`citation_auditor`; add a platform reviewer only when implementation-specific",
            "behavior matters.",
            "",
            "CI/release failures: `ci_triager` plus `env_validator` or",
            "`release_validator`, then `test_runner` for the smallest reproduction.",
            "",
            "Repo overlays: prefer local overlay agents when repo policy matters; promote an",
            "overlay to global only after it proves useful across unrelated repositories.",
            "",
            "## Regeneration",
            "",
            "```bash",
            "python3 subagents/codex/scripts/render_agents.py",
            "python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/codex/agents",
            "```",
        ]
    )
    (ROOT / "ROLE_CATALOG.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> int:
    """Run role rendering from the command line.

    Returns:
        Process exit status code.
    """

    role_count = write_roles()
    write_catalog()
    print(f"rendered {role_count} roles under {AGENTS_ROOT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
