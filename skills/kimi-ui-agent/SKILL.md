---
name: kimi-ui-agent
description: Explicit-only Kimi-powered frontend/UI orchestration skill and CLI for repo profiling, design-system context generation, Codex/Kimi Code/Claude Code adapter setup, plan-first git worktree runs, status/reply/finalize lifecycle control, and review artifacts. Use only when the user explicitly invokes `$kimi-ui-agent` or asks to use Kimi UI Agent for frontend/UI audits, redesigns, components, screenshots, implementation planning, or worktree orchestration.
---

# Kimi UI Agent

Use the bundled Bun CLI script as the deterministic controller for
Kimi-powered frontend/UI work. The CLI owns repo scanning, adapter rendering,
path validation, project config, run state, and artifact files. The active
harness, Codex, Kimi Code, or Claude Code, owns judgment, edits, and final
verification.

## Start

1. Confirm the user explicitly invoked `$kimi-ui-agent` or explicitly asked for
   Kimi UI Agent.
2. Run:

   ```bash
   KIMI_UI_AGENT_SKILL_DIR="${KIMI_UI_AGENT_SKILL_DIR:-${CODEX_HOME:-$HOME/.codex}/skills/kimi-ui-agent}"
   KIMI_UI_AGENT_CLI="${KIMI_UI_AGENT_CLI:-$KIMI_UI_AGENT_SKILL_DIR/scripts/kimi-ui-agent.ts}"
   bun "$KIMI_UI_AGENT_CLI" --json doctor
   ```

3. If this repo has not been profiled, run setup in dry-run mode first:

   ```bash
   bun "$KIMI_UI_AGENT_CLI" --json setup --dry-run
   ```

4. Apply setup only when durable project intelligence should be written:

   ```bash
   bun "$KIMI_UI_AGENT_CLI" --json setup --apply
   ```

5. Read `.agents/kimi-ui-agent/project-profile.md`,
   `.agents/kimi-ui-agent/frontend-map.md`,
   `.agents/kimi-ui-agent/design-system.md`,
   `.agents/kimi-ui-agent/verification.md`, and
   `.agents/kimi-ui-agent/protected-paths.md` when present.

## Workflow

- Use `setup` for first-run deterministic repo scanning and lean project context
  generation. Read [setup-profile](references/setup-profile.md) before changing
  profile behavior or interpreting profile files.
- Use `install --dry-run` to render project-local Codex, Kimi Code, and Claude
  Code adapters. Read [install-targets](references/install-targets.md) before
  applying adapter writes.
- Use `start --task "<task>" --dry-run` to preview a plan-first isolated
  worktree run. Apply only after checking the intended worktree, branch, and
  artifact paths.
- Use `status`, `reply`, `continue`, `finalize`, and `abort` to manage a run.
  Read [lifecycle](references/lifecycle.md) before modifying run protocol or
  interpreting artifacts.
- Use `mcp` only as a stdio MCP server configured by Codex, Kimi Code, or Claude
  Code. Read [adapter-contracts](references/adapter-contracts.md) before
  changing MCP tool schemas.

## Guardrails

- Keep this skill explicit-only. Do not enable implicit invocation for an
  editing/worktree orchestrator.
- Do not paste secrets or private tokens into Kimi, Claude, Codex, config files,
  or artifact files.
- Keep install and setup dry-run-first. Every write requires `--apply`.
- Treat generated project context as editable hints, not source of truth. Verify
  against current code before acting.
- Keep Kimi launch plan-first by default. Autonomous or yolo modes are opt-in
  and must be named by the user.
- Hooks are workflow guards, not a sandbox. Use worktree isolation, harness
  permissions, and path containment as the real safety boundary.

## Resources

- `scripts/kimi-ui-agent.ts`: Bun TypeScript CLI entrypoint. Invoke it with
  `bun "$KIMI_UI_AGENT_CLI"` after setting `KIMI_UI_AGENT_CLI` to the installed
  or repo-local script path; do not assume a global `kimi-ui-agent` binary
  exists.
- `references/security-and-privacy.md`: sanitization, path, env, and artifact
  safety rules.
- `references/output-contract.md`: JSON and Markdown artifact contract.
- `references/kimi-code.md`, `references/claude-code.md`, and
  `references/codex.md`: harness-specific notes.
- `assets/design-brief.md`: optional prompt input for high-stakes UI work.
