# Codex configuration knobs (high signal)

Codex CLI defaults come from `~/.codex/config.toml` and can be overridden per run via CLI flags (`--config`, `--profile`, etc.).

## Approvals and sandbox

- `approval_policy`: `untrusted | on-failure | on-request | never`
- `sandbox_mode`: `read-only | workspace-write | danger-full-access`
- `sandbox_workspace_write.network_access`: allow outbound network inside `workspace-write`
- Prefer adding writable paths via `--add-dir` / `sandbox_workspace_write.writable_roots` instead of `danger-full-access`.

## Exec tooling

- `features.unified_exec`: enable the PTY-backed unified exec tool (beta)
- `features.exec_policy`: enforce execpolicy checks for shell/unified exec (experimental; on by default)

## Web search

- `features.web_search_request`: allow the model to issue web searches (stable)
- CLI: `--search` (enables web search tool access)

## Project guidance discovery

- `project_doc_fallback_filenames`: additional instruction filenames to treat like `AGENTS.md`
- `project_doc_max_bytes`: cap for combined instructions chain

