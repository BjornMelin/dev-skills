# Output Contract

All CLI `--json` commands return:

```json
{
  "ok": true,
  "command": "doctor",
  "message": "doctor complete",
  "result": {}
}
```

Errors return:

```json
{
  "ok": false,
  "command": "start",
  "message": "--task is required"
}
```

Machine-readable run files use:

- `run.json`: current run state.
- `events.jsonl`: append-only lifecycle events.

Human-readable run files use Markdown so Codex, Kimi Code, Claude Code, and the
user can inspect decisions without a custom viewer.
