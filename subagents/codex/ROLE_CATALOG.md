# Codex Subagent Catalog

This pack is the source of truth for Bjorn's Codex custom subagents.
It renders global agents under `agents/global` and project overlays under
`agents/overlays/<repo>`.

Runtime policy:

- all roles use `gpt-5.5`; effort tier controls depth;
- `low` is for deterministic mapping and inventory;
- `high` is for most expert review, research, validation, and scoped work;
- `xhigh` is for high-risk work with ambiguity or conflicting evidence;
- no nested subagents by default;
- parent sessions own orchestration, waiting, synthesis, and final decisions;
- read-only is default; workspace-write is limited to implementation, tests, UI/browser, and smoke runners.

## Global Roles

| Role | Effort | Sandbox | Purpose |
| --- | --- | --- | --- |
| `guidance_mapper` | `low` | `read-only` | Read-only mapper for AGENTS.md, CLAUDE.md, README, and scoped project guidance relevant to a change. |
| `repo_explorer` | `low` | `read-only` | Read-only codebase explorer for bounded evidence gathering before changes without shadowing Codex built-ins. |
| `docs_researcher` | `low` | `read-only` | Read-only documentation researcher for official API, framework, and version behavior. |
| `env_validator` | `low` | `read-only` | Read-only environment and configuration validator for required variables, secrets wiring, and deployment config. |
| `ci_triager` | `high` | `read-only` | Read-only CI triager for failing checks, logs, workflow contracts, and likely fixes. |
| `citation_auditor` | `high` | `read-only` | Read-only auditor for claim-to-source mapping, source freshness, citation quality, and unsupported research conclusions. |
| `context7_researcher` | `high` | `read-only` | Read-only Context7 specialist for current library documentation lookups through Context7 MCP or ctx7 CLI. |
| `dependency_researcher` | `high` | `read-only` | Read-only dependency researcher for package docs, release notes, source internals, and upgrade risk. |
| `docs_auditor` | `high` | `read-only` | Read-only docs auditor for stale, missing, duplicated, or misleading repository documentation. |
| `github_researcher` | `high` | `read-only` | Read-only GitHub specialist for repository, code, issue, pull request, release, changelog, and package-manifest evidence. |
| `history_reviewer` | `high` | `read-only` | Read-only reviewer that uses git history and blame to validate whether changed code violates existing intent. |
| `implementation_worker` | `high` | `workspace-write` | Scoped implementation worker for narrow fixes with explicit file ownership. |
| `openai_docs_researcher` | `high` | `read-only` | Read-only researcher for current official OpenAI and Codex documentation. |
| `performance_reviewer` | `high` | `read-only` | Read-only performance reviewer for obvious algorithmic, rendering, database, bundle, and IO bottlenecks. |
| `release_validator` | `high` | `read-only` | Read-only release validator for changelog, versioning, tags, packaging, and publish readiness checks. |
| `reviewer` | `high` | `read-only` | Read-only reviewer focused on correctness, security, regressions, and missing tests. |
| `runtime_bug_reviewer` | `high` | `read-only` | Read-only runtime bug reviewer for null safety, async races, lifecycle leaks, and error handling. |
| `shallow_bug_reviewer` | `high` | `read-only` | Read-only high-signal reviewer for obvious diff-level bugs and regressions. |
| `source_validator` | `high` | `read-only` | Read-only package/source implementation validator for verifying docs claims against actual repository or package source. |
| `test_runner` | `high` | `workspace-write` | Validation worker that runs focused tests and reports command-level evidence without editing source. |
| `ui_debugger` | `high` | `workspace-write` | UI debugger for reproducing browser or frontend regressions and reporting actionable evidence. |
| `deep_researcher` | `xhigh` | `read-only` | Lead read-only researcher for multi-source, cited, current investigations with claim ledgers and freshness checks. |
| `false_positive_validator` | `xhigh` | `read-only` | Read-only validator that scores candidate findings and filters weak or stale issues. |
| `security_reviewer` | `xhigh` | `read-only` | Read-only security reviewer for authentication, authorization, injection, secrets, and data exposure risks. |
| `root_cause_investigator` | `xhigh` | `read-only` | Read-only root-cause investigator for hard failures, regressions, flaky behavior, and conflicting evidence. |
| `architect_reviewer` | `xhigh` | `read-only` | Read-only architecture reviewer for subsystem boundaries, ownership drift, and high-impact design decisions. |
| `pr_shepherd` | `high` | `read-only` | Read-only PR shepherd for review-to-ship loops, unresolved threads, CI state, merge blockers, and closure evidence. |
| `commit_planner` | `high` | `read-only` | Read-only conventional-commit planner for dirty trees and semantically reviewable staging lanes. |
| `docs_aligner` | `high` | `read-only` | Read-only documentation alignment reviewer for code, workflow, contract, and user-guide drift. |
| `dependency_upgrade_planner` | `high` | `read-only` | Read-only dependency upgrade planner for safe package bumps, changelog/source review, and verification lanes. |
| `nextjs_reviewer` | `high` | `read-only` | Read-only Next.js reviewer for App Router, routing, caching, server actions, middleware/proxy, and build behavior. |
| `react_reviewer` | `high` | `read-only` | Read-only React reviewer for component structure, hooks, state ownership, rendering behavior, and accessibility regressions. |
| `expo_reviewer` | `high` | `read-only` | Read-only Expo reviewer for Expo Router, native configuration, EAS workflows, OTA/runtime version, and mobile build risk. |
| `convex_reviewer` | `high` | `read-only` | Read-only Convex reviewer for schema, functions, indexes, authz, components, and backend contract risk. |
| `clerk_reviewer` | `high` | `read-only` | Read-only Clerk reviewer for auth/session flows, organization context, redirects, webhooks, and browser/mobile auth behavior. |
| `vercel_reviewer` | `high` | `read-only` | Read-only Vercel reviewer for deployments, functions, routing, env vars, build output, and release pipeline risk. |
| `openai_api_reviewer` | `high` | `read-only` | Read-only OpenAI API reviewer for model selection, Responses API usage, tool calling, structured output, and Codex behavior. |
| `bun_ts_reviewer` | `high` | `read-only` | Read-only Bun and TypeScript reviewer for package-manager policy, scripts, tests, runtime APIs, and strict typing. |
| `python_uv_reviewer` | `high` | `read-only` | Read-only Python and uv reviewer for dependency resolution, lockfiles, packaging, tests, and runtime compatibility. |

## Project Overlays

| Repo family | Role | Effort | Sandbox | Purpose |
| --- | --- | --- | --- | --- |
| `docmind` | `docmind_dependency_safety_reviewer` | `xhigh` | `read-only` | DocMind dependency safety reviewer for Dependabot, security bumps, uv locks, release notes, and source compatibility. |
| `docmind` | `docmind_python_runtime_reviewer` | `high` | `read-only` | DocMind Python runtime reviewer for Streamlit/runtime behavior, model loading, Apple MPS parity, and test taxonomy. |
| `docmind` | `docmind_ci_triager` | `high` | `read-only` | DocMind CI triager for GitHub Actions, uv frozen installs, docs lint, tests, and release automation failures. |
| `docmind` | `docmind_docs_release_auditor` | `high` | `read-only` | DocMind docs/release auditor for Release Please, changelogs, markdown lint, worklogs, and rendered docs behavior. |
| `docmind` | `docmind_model_source_validator` | `high` | `read-only` | DocMind model/source validator for Transformers, SigLIP, GGUF, image safety, and upstream implementation claims. |
| `tooling` | `skill_package_validator` | `high` | `read-only` | Agent tooling skill package validator for AgentSkills metadata, packaging, quick validation, and install portability. |
| `tooling` | `subagent_pack_reviewer` | `xhigh` | `read-only` | Agent tooling subagent pack reviewer for TOML role catalogs, model/effort policy, safety contracts, and smoke readiness. |
| `tooling` | `mcp_tooling_reviewer` | `high` | `read-only` | Agent tooling MCP/source reviewer for research CLI, Context7/GitHub/source routing, and provider evidence contracts. |
| `tooling` | `agent_runtime_smoke_tester` | `high` | `workspace-write` | Agent tooling runtime smoke tester for Codex custom agents, spawn contracts, and representative live checks. |

## Routing Recipes

PR review/remediation: `guidance_mapper` plus one reviewer lane, then
`false_positive_validator`; use `test_runner` only after the parent chooses
focused verification.

High-stakes research: `deep_researcher`, `source_validator`, and
`citation_auditor`; add a platform reviewer only when implementation-specific
behavior matters.

CI/release failures: `ci_triager` plus `env_validator` or
`release_validator`, then `test_runner` for the smallest reproduction.

Repo overlays: prefer local overlay agents when repo policy matters; promote an
overlay to global only after it proves useful across unrelated repositories.

## Regeneration

```bash
python3 subagents/codex/scripts/render_agents.py
python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/codex/agents
```
