# Agents SDK + Codex MCP: consistent workflows (single + multi-agent)

This reference distills the “consistent workflows” pattern:

- run **Codex CLI as an MCP server** so it becomes a tool,
- orchestrate multi-step work using the **OpenAI Agents SDK**,
- enforce **gated handoffs** so the workflow is deterministic and auditable,
- enable **tracing** so you can audit prompts, tool calls, file writes, and timing.

Pair this with an **ExecPlan** (`references/execplans.md`) for long work so you can resume after compaction/restarts.

## Core components

1. **Codex MCP server**
   - Launch `codex mcp-server` as a long-running stdio MCP process.
   - Tools exposed: `codex` (start) and `codex-reply` (continue).
   - Treat each tool call as a unit of work and capture `threadId` for resume.

2. **Orchestrator agent (Project Manager)**
   - Owns the plan, gates, and state.
   - Schedules specialized worker agents via handoffs.
   - Blocks advancement until required artifacts exist and checks pass.

3. **Specialized worker agents**
   - Narrow scopes (designer, frontend, backend, tester, security reviewer).
   - Produce specific deliverables only (files or structured reports).
   - Hand control back to the orchestrator after delivering.

4. **Tracing**
   - Use traces to debug and audit: prompts, tool calls, handoffs, and timing.
   - Group related work into a single trace per workflow run (use a stable `group_id` per user/session/run).

## Recommended shape: gated handoffs

Use gates as explicit “contracts” between roles:

- File existence: expected artifacts are present at known paths.
- Commands: required checks were run (tests/lint/build) and succeeded.
- Structured boundaries: when a worker returns structured JSON, validate it (schema).

Example gates (typical):

- PM writes `REQUIREMENTS.md`, `TEST.md`, `AGENT_TASKS.md` → gate: all exist.
- Designer writes `design/design_spec.md` → gate: file exists.
- Frontend writes `frontend/index.html` and backend writes `backend/server.js` → gate: both exist.
- Tester produces `tests/TEST_PLAN.md` (and optionally a script) → gate: exists + runs.

## Codex MCP tool parameters (high signal)

In worker instructions, be explicit about safety and scope:

- For analysis-only work: `{ "approval-policy": "never", "sandbox": "read-only" }`
- For controlled file writes: `{ "approval-policy": "never", "sandbox": "workspace-write" }`

Always pass a clear `cwd` and prefer writing outputs into dedicated subfolders per role.

## Make it resumable

Record resume pointers in the ExecPlan:

- `threadId` values (for Codex MCP `codex-reply`)
- JSONL log paths (if you capture `codex exec --json` elsewhere)
- artifact paths for each stage
- the exact next command/prompt to run

For multi-agent runs, also record:

- orchestrator run identifier
- trace/group identifiers (so you can navigate traces later)

## Failure handling

When a gate fails:

- do not “continue anyway”
- ask the owning worker to re-run their scoped work with the missing artifact/check as the only goal
- update the ExecPlan Progress with what failed and what will be retried

