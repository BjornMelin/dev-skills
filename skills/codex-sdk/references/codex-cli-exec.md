# Codex CLI – `codex exec` for automation

Use `codex exec` when you need:

- a CI/cron-friendly interface
- JSONL progress (`--json`)
- structured final output (`--output-schema <file>`)
- resumable multi-stage pipelines (`codex exec resume ...`)

## Key behavior

- Default output: progress to `stderr`, final message only to `stdout`.
- `--json`: `stdout` becomes a JSONL stream (one JSON object per line).

## Common flags (high signal)

- `--json`
- `--output-schema <path>`
- `--output-last-message <path>` / `-o <path>`
- `--sandbox read-only|workspace-write|danger-full-access`
- `--ask-for-approval untrusted|on-failure|on-request|never`
- `--full-auto` (shortcut; still avoid in untrusted contexts)
- `--cd <dir>`
- `--add-dir <dir>` (prefer over `danger-full-access`)

## Resuming

- `codex exec resume --last "<prompt>"`
- `codex exec resume <SESSION_ID> "<prompt>"`

## Durable logs

Always capture JSONL for audits and debugging:

```bash
codex exec --json "<prompt>" | tee codex.jsonl
```

For multi-hour work, also record the prompt, JSONL path, and resume IDs in an ExecPlan (`references/execplans.md`).

Then ingest to SQLite:

```bash
python3 scripts/codex_jsonl_to_sqlite.py --db codex.sqlite --init
cat codex.jsonl | python3 scripts/codex_jsonl_to_sqlite.py --db codex.sqlite --run-label "my-run"
```
