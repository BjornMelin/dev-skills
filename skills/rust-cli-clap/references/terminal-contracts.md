# Terminal Contracts

## Streams

- stdout is for primary command output.
- stderr is for diagnostics, progress, warnings, and trace-style information.
- JSON, NDJSON, CSV, or other machine output must be complete, deterministic, and free of human prose.
- Progress bars belong on stderr and should disable automatically when stderr is not a terminal.

## Exit Codes

Keep exit codes small and documented for automation-heavy tools:

- `0`: success
- `1`: general runtime failure
- `2`: invalid user input or usage
- Additional codes only when callers can realistically branch on them.

Avoid panics for expected user errors. Convert domain failures into typed errors and a presentation layer that gives enough context to act.

## Color and Formatting

- Respect `NO_COLOR` and non-TTY output.
- Provide `--color auto|always|never` for tools with rich output.
- Use tables only for humans. Provide JSON for scripts.
- Keep terminal width responsive; avoid wrapping data in ways that break copy/paste.

## Logging and Verbosity

Prefer a consistent verbosity shape:

- Default: quiet unless user action is needed.
- `-v`/`--verbose`: explain major steps.
- `-vv` or tracing filter: detailed diagnostics.
- `--quiet`: suppress non-essential human output.

Do not log secrets, tokens, URLs with credentials, or full request bodies by default.

## Config and Environment

Use an explicit source model. When reporting effective config, include source labels but redact secrets.

Avoid hidden environment variables. If an env var is supported, document it, test it, and include it in help when appropriate.
