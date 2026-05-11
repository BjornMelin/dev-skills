# codex-dev Core Reference

`codex-dev-core` is the shared Rust crate for local `codex-dev` task capsule
contracts and read models. It is intentionally small: CLI parsing, terminal UI
rendering, hosted provider calls, subprocess execution, and merge/review actions
live outside this crate.

Tracking: #40, #42, #43, and #44.

## Public Boundary

The crate owns:

- schema constants such as `CAPSULE_SCHEMA`, `PR_SCHEMA`, and
  `POLICY_GATES_SCHEMA`;
- serde data models for `capsule.json`, `evidence.jsonl`,
  `verification.json`, `subagents.json`, `pr.json`, and `policy.json`;
- local capsule helpers including `init_capsule`, `validate_capsule`,
  `capsule_status`, `render_capsule`, `append_evidence`,
  `record_subagent_plan`, `record_subagent_outcome`,
  `record_subagent_synthesis`, `record_pr_snapshot`, and `pr_status`;
- policy and PR evidence data models such as `PolicyManifest`,
  `PolicyGate`, `PrControlPlan`, and `PrControlCommand`; policy gates include
  command, source, working directory, required tools, local/network/secret
  posture, and failure interpretation metadata;
- subagent plan/outcome/synthesis data models for recording `subspawn` planner
  output, prompt IDs/hashes, duplicate-role warnings, human-verified
  dispositions, and synthesis notes;
- evidence summaries used by status/render output, including total count,
  count by kind, and latest record by kind;
- narrow JSON/JSONL file helpers used by `codex-dev` when recording policy
  and evidence appender output.

The crate does not own:

- Clap command definitions or shell UX;
- Ratatui/crossterm terminal rendering;
- `gh`, `review-pack`, CodeRabbit, or hosted GitHub API calls;
- subprocess execution for policy gates;
- default executable policy-gate or hosted PR command recipes;
- compatibility shims for obsolete capsule layouts.

## Dependency Policy

`codex-dev-core` should stay free of CLI/TUI dependencies. It must not depend on
`clap`, `ratatui`, `crossterm`, `review-pack`, `gh`, or provider SDKs. Keep
dependencies limited to serde, time, error handling, and local filesystem
contract helpers unless a future issue proves a new dependency is unavoidable.

## Crate Checks

Run after changing `crates/codex-dev-core/`:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo test -p codex-dev-core
cargo check --workspace
```
