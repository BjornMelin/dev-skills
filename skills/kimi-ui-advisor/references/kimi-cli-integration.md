# Kimi CLI Integration Notes

Use this reference when debugging or changing the wrapper.

## Current Evidence

- Official Kimi Code CLI docs describe `kimi --print` as non-interactive and
  suitable for scripting.
- `--quiet` is shorthand for `--print --output-format text
  --final-message-only`.
- `--output-format=stream-json` emits JSONL messages. With
  `--final-message-only`, the assistant message content is still a string, so
  the wrapper must parse the assistant content as the second-stage JSON object.
- Print mode is non-interactive and auto-approves tool calls for the invocation.
  The local 1.44.0 source confirms the CLI passes `runtime_afk=ui == "print"`.
- The safer pattern is to use `--agent-file` with a custom Kimi agent that omits
  shell and write tools.
- Local verification during authoring:
  - `kimi --version` returned `kimi, version 1.44.0`.
  - `kimi info` returned wire protocol `1.10` and Python `3.13.12`.
  - GitHub `MoonshotAI/kimi-cli` `main` advertised package version `1.45.0`
    in `pyproject.toml`.

## Important CLI Flags

```bash
kimi --print -p "task"
kimi --print --output-format=stream-json --final-message-only -p "task"
kimi --quiet -p "task"
kimi --agent-file /path/to/agent.yaml
kimi --work-dir /path/to/repo
kimi --no-thinking
kimi --max-steps-per-turn 8
```

Print-mode exit codes:

- `0`: success
- `1`: non-retryable failure such as auth, config, or quota exhaustion
- `75`: retryable provider/transient failure

The wrapper defaults to `--no-thinking` for shorter deterministic JSON output.
Use `--thinking` only when a UI task needs deeper exploration and extra latency
is acceptable.

## Wrapper Modes

The wrapper adds Codex-facing modes on top of Kimi print mode:

```bash
python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py --mode audit
python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py --mode redesign
python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py --mode component
python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py --mode screenshot-review --image /tmp/screen.png
python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py --compare --before-image /tmp/before.png --after-image /tmp/after.png
python3 skills/kimi-ui-advisor/scripts/kimi_ui_advisor.py --design-brief-file skills/kimi-ui-advisor/templates/design-brief.md
```

`--compare` is a shortcut for `--mode compare`. Screenshot and compare modes
depend on the custom agent's `ReadMediaFile` tool.

## Custom Kimi Agent

The bundled agent intentionally allows only:

- `ReadFile`
- `ReadMediaFile`
- `Glob`
- `Grep`
- `SearchWeb`
- `FetchURL`

Do not add `Shell`, `WriteFile`, `StrReplaceFile`, `Agent`, or
`AskUserQuestion` unless the skill design is revisited.
