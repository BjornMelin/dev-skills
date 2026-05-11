# codex-dev TUI Reference

`codex-dev-tui` is an optional Ratatui workbench for local `codex-dev` task
capsules. By default it opens a read-only dashboard over a `.codex/tasks` root
so an operator can scan recent capsules before opening one capsule in detail.
It reads the existing capsule JSON contracts through `codex-dev-core` and
renders an operator view for quick scanning. It does not own policy gates, PR
remediation, or capsule business logic.

Tracking: #20, #28, and #52.

## Ownership Boundary

The TUI consumes:

- `capsule.json` as the `codex-dev.task-capsule.v1` contract after
  `codex_dev_core::validate_capsule`;
- `verification.json` as `codex-dev.verification.v1`;
- `pr.json` as `codex-dev.pr.v1`;
- `subagents.json` as `codex-dev.subagents.v1` in dashboard mode;
- `codex_dev_core::validate_capsule` for validation errors.

The TUI must not scrape Markdown notes or duplicate policy-gate decisions.
Automation should continue to use `codex-dev --json` and contract files as the
machine-readable interfaces. Dashboard rendering is intentionally read-only: it
does not run validation commands, mutate PR state, or execute remediation
actions.

## Install And Run

Build the binary:

```bash
cargo build -p codex-dev-tui
```

Open the dashboard for the default local task root:

```bash
cargo run -q -p codex-dev-tui
```

Open the dashboard for an explicit task root:

```bash
cargo run -q -p codex-dev-tui -- --root .codex/tasks
```

Open a local task capsule directly:

```bash
cargo run -q -p codex-dev-tui -- --capsule .codex/tasks/<id>
```

Interactive mode polls for terminal input and refresh ticks every 250
milliseconds by default. Use `--tick-ms <milliseconds>` to tune that interval;
`0` is rejected so the TUI cannot busy-loop.

Keys:

- dashboard: up/down arrows or `j`/`k` select capsules
- dashboard: `enter` opens the selected valid capsule
- dashboard: `f` cycles filters and `s` cycles sort order
- detail view: `b` or backspace returns to the dashboard
- `tab`, right arrow, or `l`: next panel
- `shift-tab`, left arrow, or `h`: previous panel
- `r`: reread dashboard or capsule JSON contracts
- `q`, escape, or ctrl-c: quit

The dashboard shows task title, capsule state, validation summary, evidence
count, subagent batch summary, PR state, and last update time. Missing task
roots, unreadable entries, and invalid capsules are rendered as diagnostics
instead of panicking or starting command execution.

## Deterministic Render Smoke

Use `--render-once` for automation, CI logs, and review evidence. It renders
one frame through Ratatui's `TestBackend`, prints the buffer, and does not enter
raw terminal mode.

Render the dashboard:

```bash
cargo run -q -p codex-dev-tui -- \
  --root .codex/tasks \
  --render-once \
  --width 120 \
  --height 32
```

Render a single capsule:

```bash
cargo run -q -p codex-dev-tui -- \
  --capsule .codex/tasks/<id> \
  --render-once \
  --width 100 \
  --height 30
```

Single-capsule render exits nonzero when the capsule is invalid. Dashboard
render exits successfully when the task root or individual capsules are invalid
because those states are part of the screen diagnostics contract.

## Testing Contract

The crate keeps UI state and rendering testable without opening a real
terminal:

- state loading tests create a real `codex-dev-core` capsule and read its JSON
  contracts;
- render snapshot tests assert the `TestBackend` buffer includes capsule,
  validation, and PR summaries;
- dashboard tests assert root discovery, invalid-capsule diagnostics, filter
  changes, sort changes, and open-single-capsule behavior;
- cleanup tests prove the restore guard runs exactly once, including on drop.

The exact validation matrix lives in `docs/runbooks/validation.md`. Focused TUI
checks are:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-tui --all-targets -- -D warnings
cargo check -p codex-dev-tui
cargo test -p codex-dev-tui
```

Run the runbook's `--render-once` smoke when state loading or rendering changes.
