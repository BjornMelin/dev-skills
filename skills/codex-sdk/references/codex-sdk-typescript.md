# Codex SDK (TypeScript) – patterns that hold up in production

The Codex SDK (`@openai/codex-sdk`) wraps the bundled `codex` binary and communicates via JSONL events over stdin/stdout.

## Core concepts

- `Codex`: creates and resumes threads
- `Thread`: a conversation that can span multiple turns
- `thread.run(...)`: returns a completed turn (buffered)
- `thread.runStreamed(...)`: streams `ThreadEvent` records as the agent works

## Defaults (recommendation)

Use the least privileges needed:

- analysis-only: `sandboxMode: "read-only"`, `approvalPolicy: "never"`
- controlled edits: `sandboxMode: "workspace-write"`, `approvalPolicy: "on-request"` or `"on-failure"`

## Structured output (robust boundary)

Treat every “downstream decision” as a structured output boundary:

1. define a Zod schema (strict object, `additionalProperties: false` behavior)
2. generate JSON Schema (`z.toJSONSchema(schema)`)
3. pass it as `outputSchema`
4. `JSON.parse(turn.finalResponse)` then validate with Zod

## Streaming event handling (what matters)

Handle at minimum:

- `turn.failed` and top-level `error`
- `turn.completed` (token usage)
- `item.*` lifecycle events
- item types:
  - `agent_message` (final response)
  - `command_execution` (command, output, exit code, status)
  - `file_change` (patch applied, paths)
  - `mcp_tool_call` (server/tool, args, results/errors)
  - `todo_list` (plan tool updates)

## Resuming threads

Persist the `thread.id` (after `thread.started`) and resume with `codex.resumeThread(threadId)`.

If you need end-to-end durability, store thread IDs and JSONL event logs in SQLite (see `state-memory-sqlite.md`).

