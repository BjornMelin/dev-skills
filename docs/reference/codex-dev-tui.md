# codex-dev TUI Reference

`codex-dev-tui` is an optional Ratatui workbench for local `codex-dev` task
capsules. By default it opens a read-only dashboard over a `.codex/tasks` root
so an operator can scan recent capsules before opening one capsule in detail.
It reads the existing capsule JSON contracts through `codex-dev-core` and stays
aligned with capsule validation semantics. The dashboard and detail panels are
composed into the read-only `tui_operator_panels.v1` view from shared core
contracts such as `skill_inventory.v1`, `task_index.v1`, `orchestration_run.v1`,
and PR-agent reports. The TUI renders operator state for quick scanning and does
not own policy gates, PR remediation, or capsule business logic.

Tracking: #20, #28, #52, #53, #55, #82, and #84.

## Ownership Boundary

The TUI consumes:

- `capsule.json` as the `codex-dev.task-capsule.v1` contract after
  `codex_dev_core::validate_capsule`;
- `verification.json` as `codex-dev.verification.v1`;
- `evidence.jsonl` as append-only `codex-dev.evidence.v1` records;
- `pr.json` as `codex-dev.pr.v1`;
- `subagents.json` as `codex-dev.subagents.v1`;
- optional `pr-agent-state.json` as `codex-dev.pr-agent-state.v1`;
- optional `pr-readiness.json` as `codex-dev.pr-agent-readiness.v1`;
- optional `pr-agent-actions/<plan-id>/plan.json` files as
  `codex-dev.pr-agent-hosted-action.v1`;
- `skill_inventory.v1` and `task_index.v1` from `codex-dev-core` for dashboard
  skill health and task root panels;
- `orchestration_run.v1` from `codex-dev-core` for subspawn batch completion,
  diagnostics, runtime agent IDs, and wait metadata;
- `tui_operator_panels.v1` as the TUI-owned composition contract for rendered
  operator panels and next-action exports;
- `codex_dev_core::validate_capsule` for validation errors.

The TUI must not scrape Markdown notes or duplicate policy-gate decisions.
Automation should continue to use `codex-dev --json` and contract files as the
machine-readable interfaces. Dashboard and detail rendering are intentionally
read-only: they do not run validation commands, mutate PR state, execute
remediation actions, or make live GitHub/provider calls. Optional PR-agent
artifacts are local evidence only; malformed optional artifacts appear as
redacted diagnostics instead of panics.

## Install And Run

Build the binary:

```bash
cargo build -p codex-dev-tui
cargo install --path crates/codex-dev-tui --locked --force
```

Use [Global CLI Workflow](../runbooks/global-cli-workflow.md) for the
three-binary install/update workflow and shell artifact generation.

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

Generate shell completions and a manpage:

```bash
cargo run -q -p codex-dev-tui -- completions zsh > /tmp/_codex-dev-tui
cargo run -q -p codex-dev-tui -- manpage > /tmp/codex-dev-tui.1
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
count, subagent batch summary, PR state, last update time, task-index totals,
and skill-health totals from `skill_inventory.v1`. Missing task roots,
unreadable entries, and invalid capsules are rendered as diagnostics instead of
panicking or starting command execution.

Single-capsule detail mode has these panels:

- Overview: capsule objective, branch/issue/PR pointers, and loaded artifact
  summaries.
- Evidence: local `evidence.jsonl` counts, kind totals, recent evidence
  records, source IDs, artifact paths, confidence, residual risk, and warnings
  for missing source context or stale capsule evidence counts. It deliberately
  hides raw command output and provider dumps.
- Subagents: delegation batches, mode/scope, completed and human-verified agent
  counts, agent summaries, source IDs, artifacts, and synthesis status.
- Orchestration: `orchestration_run.v1` completion coverage, expected and
  recorded roles, runtime agent IDs, wait results, synthesis status, stale
  evidence warnings, registry warnings, and diagnostics.
- PR: normalized `pr.json` snapshot, check state, and authoritative versus
  non-authoritative review-thread status.
- PR Agent: local PR-agent state/readiness/action artifacts. It distinguishes
  dry-run plans from apply-requested or executed hosted actions, summarizes
  readiness blockers, wait reasons, warnings, failing/pending checks, and
  action status without printing raw stdout/stderr.
- Next Actions: render-only commands and action summaries derived from
  PR-agent state actions, PR-agent readiness/action artifacts, and
  orchestration diagnostics. The panel does not execute commands or perform
  hosted writes. For PR-agent hosted actions it renders high-level `codex-dev`
  invocations instead of body-bearing `gh` mutation commands, and generated
  command placeholders use shell-safe tokens such as `CAPSULE_DIR` and
  `SUMMARY`.
- Validation: required and optional gate summaries plus artifact diagnostics.
- Help: command and automation reminder.

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
  validation, evidence, subagent, orchestration, PR, PR-agent, and next-action
  summaries;
- dashboard tests assert root discovery, invalid-capsule diagnostics, filter
  changes, sort changes, skill health, task index, and open-single-capsule
  behavior;
- optional PR-agent artifact tests assert malformed local artifacts render as
  diagnostics and redact capsule paths;
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
