# Codex Custom Subagent Authoring Guide

Use this reference when creating new custom agent roles or changing template
defaults. Prefer current official OpenAI Codex docs when behavior may have
changed.

Last verified against OpenAI Codex docs and source-adjacent behavior:
2026-05-07.

## Current Codex Model

Custom agents are standalone TOML files:

- personal agents: `~/.codex/agents/*.toml`
- project agents: `.codex/agents/*.toml`

Each standalone file defines one custom agent. Required fields:

- `name`
- `description`
- `developer_instructions`

Useful optional fields:

- `nickname_candidates`
- `model`
- `model_reasoning_effort`
- `sandbox_mode`
- `mcp_servers`
- `skills.config`

Project-scoped config loads only for trusted projects. Global settings remain
under `[agents]` in Codex config:

- `agents.max_threads`, default 6 when unset
- `agents.max_depth`, default 1 when unset
- `agents.job_max_runtime_seconds`, used by CSV fan-out jobs

Keep `agents.max_depth = 1` unless recursive delegation is explicitly needed.
Nested fan-out increases cost, latency, and predictability risk.

## Authoring Rules

- Keep each role narrow and opinionated.
- Prefer snake_case names and matching filenames.
- Do not shadow built-in Codex agent names (`default`, `worker`, `explorer`)
  unless the override is intentional and documented.
- Give each role a clear stop condition and return format.
- Include a parent-boundary rule, a no-nested-subagents rule, and a secret
  redaction rule in every reusable template.
- Pin models in role files only when the role has a stable reason to differ
  from the parent session.
- Use read-only sandboxes for audit, review, exploration, documentation, and CI
  triage roles.
- Use workspace-write only when the role must run commands that write caches,
  drive a browser, or edit source.
- Do not put secrets, tokens, local absolute project paths, or user-specific
  credentials in templates.

## Model Defaults

Use tiered defaults:

- `gpt-5.5`: demanding review, security, debugging, implementation, and
  ambiguous multi-step work.
- `gpt-5.4`: fallback when `gpt-5.5` is unavailable or a workflow is pinned to
  it.
- `gpt-5.4-mini`: lighter read-heavy scans, documentation checks, inventories,
  and supporting workers.
- `gpt-5.3-codex-spark`: low-latency text-only triage when depth is not needed.

Use `model_reasoning_effort = "medium"` by default. Use `high` only for roles
that need complex logic tracing, security review, edge-case analysis, or
debugging. Avoid `xhigh` in reusable templates unless the role is explicitly
for unusually hard bounded reasoning.

## Runtime Compatibility Notes

Pair custom agents with `$subspawn` for runtime orchestration.

For Codex multi-agent v2 surfaces, `spawn_agent` may expose `task_name` and
`fork_turns`. Current public Codex source has a sharp edge: omitting
`fork_turns` defaults to full-history fork, and full-history forks reject
explicit `agent_type`, `model`, or `reasoning_effort` overrides. For custom or
specialized agents, use a fresh or partial fork unless the user explicitly asks
for full-history inheritance.

When the active tool schema is legacy and exposes `fork_context`, do not assume
full-context forks can combine safely with custom role or model overrides. Use
fresh, bounded prompts with the needed context embedded.

## Context7 Notes

Use Context7 for current library, framework, SDK, API, and CLI documentation.
Current `ctx7` CLI usage is:

```bash
ctx7 library <name> "<full question>"
ctx7 docs <libraryId> "<full question>"
```

The historical `ctx7 docs --research` mode appeared in `ctx7` 0.4.0 and was
removed in 0.4.1. Source changelogs indicate it was removed because long
research calls created timeout problems in MCP clients. Treat old research
mode as an implementation reference only; do not depend on it in templates
without a bounded timeout and fallback plan.

For deep research, prefer a resumable orchestration helper that combines
Context7, official docs, web/Exa search, GitHub/source inspection, and
`opensrc`, rather than one opaque long-running Context7 call.

## Validation Checklist

Before installing or recommending a role:

1. Parse TOML successfully.
2. Confirm required fields are present and non-empty.
3. Confirm `name` is snake_case and matches the filename.
4. Confirm `model_reasoning_effort` and `sandbox_mode` values are valid.
5. Confirm nickname candidates are unique and use supported characters.
6. Confirm the role cannot edit source unless its sandbox and instructions
   intentionally allow it.
7. Confirm the role does not shadow a built-in Codex agent name.
8. Run `scripts/subagent_creator.py doctor` when install/runtime behavior is
   in question.
9. Run a temp-project smoke workflow when runtime wiring matters.

## Helper CLI

Use `scripts/subagent_creator.py`:

- `list --packs`: list bundled roles and packs.
- `doctor`: inspect Codex binary/config/agent directories and template health.
- `install`: copy selected templates without overwriting by default.
- `sync`: overwrite selected templates with backups by default.
- `diff`: compare bundled templates to installed roles.
- `backup`: copy installed roles before risky changes.
- `validate`: parse and validate TOML files.
- `smoke`: create a temporary project and optional live Codex smoke prompt.

## Sources To Refresh

- `https://developers.openai.com/codex/subagents`
- `https://developers.openai.com/codex/concepts/subagents`
- `https://developers.openai.com/codex/config-reference`
- `https://github.com/openai/codex`
