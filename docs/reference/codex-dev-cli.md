# codex-dev CLI Reference

`codex-dev` is the development operating-layer CLI for local task capsules.
It is separate from `codex-research`: research evidence stays research-owned,
while `codex-dev` records the local task capsule for a development branch.
Future lanes add policy, PR, bootstrap packs, and TUI surfaces.

Tracking: #20 and #22.

## Installation

From the repository root:

```bash
cargo build -p codex-dev
cargo run -q -p codex-dev -- --help
```

The binary supports `--json` globally for machine-readable command output. With
`--json`, command errors still print a `codex-dev.output.v1` envelope with
`ok: false` and a structured `result.error.message`, then exit nonzero.

## Capsule Root

By default, capsules are created under:

```text
.codex/tasks/<timestamp>-<slug>/
```

This path is ignored by git. PR descriptions should summarize capsule evidence
instead of committing local capsule directories.

## Commands

```text
codex-dev [--json] capsule <command>
```

Capsule commands:

- `init`
- `validate`
- `status`
- `render`

## capsule init

Create a local task capsule with the canonical v1 layout.

```bash
cargo run -q -p codex-dev -- capsule init \
  --title "Build codex-dev task capsules" \
  --objective "Add the capsule CLI core" \
  --branch feat/codex-dev-task-capsules \
  --issue 22
```

Deterministic fixture-friendly options:

```bash
cargo run -q -p codex-dev -- --json capsule init \
  --title "Build codex-dev task capsules" \
  --branch feat/codex-dev-task-capsules \
  --issue 22 \
  --root /tmp/codex-dev-smoke \
  --id test-capsule \
  --created-at 2026-05-09T04:00:00Z
```

The command writes:

```text
capsule.json
plan.md
decisions.md
evidence.jsonl
verification.json
subagents.json
pr.json
retrospective.md
```

`--id` must be one safe path segment containing only ASCII letters, numbers,
`-`, or `_`. `--force` replaces an existing capsule directory at the same ID;
it does not append to prior capsule history.

`--status` accepts the same snake_case values persisted in `capsule.json`:
`active`, `blocked`, `ready_for_pr`, `in_review`, `merged`, or `closed`.

## capsule validate

Validate required files and JSON schema identifiers:

```bash
cargo run -q -p codex-dev -- --json capsule validate .codex/tasks/<id>
```

Invalid capsules exit nonzero. With `--json`, the command still prints a
`codex-dev.output.v1` envelope with `ok: false` and `result.valid: false`.

## capsule status

Print the task capsule summary:

```bash
cargo run -q -p codex-dev -- capsule status .codex/tasks/<id>
```

## capsule render

Render a Markdown summary from the contract JSON:

```bash
cargo run -q -p codex-dev -- capsule render .codex/tasks/<id>
```

Automation should read the JSON contract files or `--json` output. Markdown
files remain human notes.

## Validation

Run after changing `crates/codex-dev/`:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
```

Use [Validation](../runbooks/validation.md) for the canonical local matrix and
task capsule smoke.
