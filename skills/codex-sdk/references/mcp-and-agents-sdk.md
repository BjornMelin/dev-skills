# Codex as an MCP server + OpenAI Agents SDK orchestration

This is the most flexible route for “agent teams”:

- run Codex as an MCP server (`codex mcp-server`)
- orchestrate it from a separate process (OpenAI Agents SDK, your own runner, etc.)
- keep each role scoped and auditable

## Why MCP matters

MCP turns Codex into a tool your orchestrator can call:

- `codex`: start a session (returns `threadId`)
- `codex-reply`: continue a session by `threadId`

This separation lets you:

- maintain strict orchestration logic outside Codex
- run multiple Codex sessions in parallel
- record traces and enforce gates

## Tool contract (Codex MCP server)

When you run Codex as an MCP server, it exposes two tools:

- `codex` (start a session)
- `codex-reply` (continue a session)

High-signal parameters to pass to `codex`:

- `prompt` (required)
- `cwd` (working directory)
- `sandbox` (`read-only` | `workspace-write` | `danger-full-access`)
- `approval-policy` (`untrusted` | `on-request` | `on-failure` | `never`)
- `include-plan-tool` (if you want Codex to emit plan/todo updates)
- `model` (optional override)

`codex-reply` requires:

- `threadId` (required)
- `prompt` (required)

Notes:

- Some surfaces also accept `conversationId` as a deprecated alias for `threadId`.
- Prefer reading the `threadId` from a tool call result’s structured content, when present.

## Recommended multi-agent shape (gated handoffs)

Roles:

- **Project Manager (orchestrator)**: owns the plan, gates, and state
- **Designer**: produces specs
- **Developer(s)**: implement scoped changes
- **Tester/Verifier**: runs tests, validates behavior, decides pass/fail
- (optional) **Reviewer/Security**: structured review + policy checks

Gates:

- “file exists” checks for expected artifacts
- “tests passed” checks for each stage
- “structured output validated” checks for tool boundaries

### Make the gates durable

For multi-hour runs, put the gating logic in an ExecPlan (`references/execplans.md`):

- the PM/orchestrator owns the plan file and updates it as gates pass/fail
- the plan records the exact artifact paths, commands, and expected outputs
- the plan records resume pointers (`threadId`, JSONL paths, DB path)

## Tracing

If using OpenAI Agents SDK, enable tracing and group multi-step workflows into a single trace.

## Tooling: dynamic MCP servers

Add specialized MCP servers (filesystem, issue trackers, etc.) and treat each as a bounded capability. Prefer allowlists for enabled tools per server.
