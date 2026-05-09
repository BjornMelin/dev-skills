# codex-dev TUI Reference

`codex-dev-tui` is an optional Ratatui workbench for local `codex-dev` task
capsules. It reads the existing capsule JSON contracts and renders an operator
view for quick scanning. It does not own policy gates, PR remediation, or
capsule business logic.

Tracking: #20 and #28.

## Ownership Boundary

The TUI consumes:

- `capsule.json` through `codex_dev::capsule_status`;
- `verification.json` as `codex-dev.verification.v1`;
- `pr.json` through `codex_dev::pr_status`;
- `codex_dev::validate_capsule` for validation errors.

The TUI must not scrape Markdown notes or duplicate policy-gate decisions.
Automation should continue to use `codex-dev --json` and contract files as the
machine-readable interfaces.

## Install And Run

Build the binary:

```bash
cargo build -p codex-dev-tui
```

Open a local task capsule:

```bash
cargo run -q -p codex-dev-tui -- --capsule .codex/tasks/<id>
```

Keys:

- `tab`, right arrow, or `l`: next panel
- `shift-tab`, left arrow, or `h`: previous panel
- `r`: reread capsule JSON contracts
- `q`, escape, or ctrl-c: quit

## Deterministic Render Smoke

Use `--render-once` for automation, CI logs, and review evidence. It renders
one frame through Ratatui's `TestBackend`, prints the buffer, and does not enter
raw terminal mode. It exits nonzero when the capsule is invalid.

```bash
cargo run -q -p codex-dev-tui -- \
  --capsule .codex/tasks/<id> \
  --render-once \
  --width 100 \
  --height 30
```

## Testing Contract

The crate keeps UI state and rendering testable without opening a real
terminal:

- state loading tests create a real `codex-dev` capsule and read its JSON
  contracts;
- render snapshot tests assert the `TestBackend` buffer includes capsule,
  validation, and PR summaries;
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
