# codex-dev CLI Reference

`codex-dev` is the development operating-layer CLI for local task capsules.
It is separate from `codex-research`: research evidence stays research-owned,
while `codex-dev` records the local task capsule for a development branch.
It also plans or executes repo-native policy gates, captures normalized PR
evidence, and records those outcomes in the task capsule. The optional
`codex-dev-tui` workbench reads these same contracts for terminal scanning.
Shared capsule schemas and local read/write helpers live in
[`codex-dev-core`](codex-dev-core.md). The `codex-dev` CLI crate keeps Clap
parsing, command output, and policy subprocess execution.

Tracking: #20, #22, #23, #25, and #42.

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
codex-dev [--json] <command>
```

Top-level commands:

- `capsule`
- `evidence`
- `policy`
- `pr`

Capsule subcommands:

- `capsule init`
- `capsule validate`
- `capsule status`
- `capsule render`

Evidence subcommands:

- `evidence append`

Policy subcommands:

- `policy manifest`
- `policy run`

PR subcommands:

- `pr plan`
- `pr record`
- `pr status`

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
policy.json
output.md
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
Validation is intentionally strict: every required capsule file must exist, and
contract files such as `pr.json` and `policy.json` must keep their documented
schema identifiers.

## capsule status

Print the task capsule summary:

```bash
cargo run -q -p codex-dev -- capsule status .codex/tasks/<id>
```

Human output includes compact evidence counts. `--json` output includes an
`evidence` summary with total record count, count by kind, and the latest
timestamp and summary per kind.

## capsule render

Render a Markdown summary from the contract JSON:

```bash
cargo run -q -p codex-dev -- capsule render .codex/tasks/<id>
```

Automation should read the JSON contract files or `--json` output. Markdown
files remain human notes. Rendered Markdown includes an `Evidence` section with
the total record count and latest record by kind.

## evidence append

Append one structured evidence record to `evidence.jsonl`:

```bash
cargo run -q -p codex-dev -- --json evidence append \
  --capsule .codex/tasks/<id> \
  --kind decision \
  --summary "Use one typed evidence append command" \
  --source-id issue:42 \
  --actor codex \
  --tool codex-dev \
  --confidence 95 \
  --residual-risk "future PR normalizers still need fixtures" \
  --artifact docs/reference/codex-dev-cli.md \
  --at 2026-05-09T06:00:00Z
```

Supported `--kind` values are `command`, `subagent`, `review`, `ci`,
`decision`, `research`, `manual`, and `output`.

Fields:

- `--capsule <path>` points at an already-valid capsule.
- `--kind <kind>` selects the typed evidence kind.
- `--summary <text>` is required and must be non-empty.
- `--at <RFC3339>` is optional; it defaults to the current time.
- `--command <command>` and `--exit-code <code>` record command evidence.
  `--exit-code` requires `--command`.
- `--source-id <id>` may be repeated for local source IDs such as issue IDs,
  fixture IDs, or sanitized IDs from an external evidence ledger. The command
  does not fetch or ingest provider output.
- `--actor <name>` and `--tool <name>` record who or what produced the
  evidence.
- `--confidence <0..100>` records a bounded confidence score when useful.
- `--residual-risk <text>` records known caveats or follow-up risk.
- `--artifact <path-or-id>` may be repeated for local artifacts.

The command validates the record before writing. Invalid records fail nonzero
with a typed JSON error envelope under `--json` and do not append to
`evidence.jsonl`. Empty text, control characters, empty repeated values, an
`--exit-code` without `--command`, and out-of-range confidence are rejected.
The command also rejects symlinked JSON/JSONL capsule contract files before
validation or writing. Successful appends update `capsule.json.updated_at`
monotonically; backfilled evidence does not move the capsule timestamp
backwards.

## policy manifest

Print the built-in repo-native gate manifest:

```bash
cargo run -q -p codex-dev -- --json policy manifest
```

The default profile is `codex_dev`. The manifest is versioned as
`codex-dev.policy-gates.v1` and ties each gate to its source in
`docs/runbooks/validation.md`. The default profile contains only local gates
that do not require secrets or network access.

## policy run

Plan or execute policy gates and record the result in a capsule:

```bash
cargo run -q -p codex-dev -- --json policy run --capsule .codex/tasks/<id>
```

By default, `policy run` is a dry run. It updates `verification.json`, appends
planned gate evidence to `evidence.jsonl`, and updates `capsule.json`
`updated_at` monotonically, but does not execute commands.

Execute gates explicitly:

```bash
cargo run -q -p codex-dev -- --json policy run \
  --capsule .codex/tasks/<id> \
  --execute
```

Executed required-gate failures set `ok: false` and exit nonzero. Use
`--keep-going` to continue after a failed required gate. Gates marked as
network-using are skipped unless `--allow-network` is passed; the built-in
`codex_dev` profile currently has no network or secret gates.

Execution discovers the repository root from the current directory or capsule
path before running repo-native commands. Pass `--repo-root <path>` for
installed-binary workflows where discovery would be ambiguous.

## pr plan

Print the live-command plan for capturing hosted PR evidence:

```bash
cargo run -q -p codex-dev -- --json pr plan \
  --repo BjornMelin/dev-skills \
  --number 25
```

The output schema is `codex-dev.pr-control-plan.v1`. Commands are intentionally
network- and secrets-marked because they use live GitHub auth, `review-pack`,
and `gh-pr-review-fix` surfaces. `codex-dev` does not reimplement review
remediation; it records the command plan so agents can run the canonical tools
and capture the normalized result. Commands that need caller-supplied artifacts
set `manual_input`; for example `review-pack remaining` requires the bundle path
created by `review-pack start` and is not marked as directly executable.

## pr record

Record a local normalized PR snapshot fixture into a task capsule:

```bash
cargo run -q -p codex-dev -- --json pr record \
  --capsule .codex/tasks/<id> \
  --source /tmp/pr-snapshot.json \
  --checked-at 2026-05-09T05:00:00Z
```

The fixture is local-only. Live provider output should be normalized before it
is recorded. The accepted input shape is:

```json
{
  "repository": "BjornMelin/dev-skills",
  "number": 25,
  "url": "https://github.com/BjornMelin/dev-skills/pull/25",
  "state": "OPEN",
  "checks": [
    {
      "name": "CodeRabbit",
      "status": "COMPLETED",
      "conclusion": "SUCCESS",
      "url": "https://example.test/check"
    }
  ],
  "review_threads": {
    "unresolved": 0
  }
}
```

`pr record` requires an already-valid capsule. It writes `pr.json`, appends
review evidence to `evidence.jsonl`, updates `capsule.json.updated_at`
monotonically, and adds the PR number to `capsule.json.pull_requests` when it
is not already present. It rejects symlinked JSON/JSONL capsule contract files
before validation or writing. It does not create missing capsule contracts or
repair a drifted schema name; use `capsule init --force` only when replacing
the full local capsule layout is intentional.

## pr status

Print the PR snapshot currently stored in the capsule:

```bash
cargo run -q -p codex-dev -- pr status --capsule .codex/tasks/<id>
```

## Validation

Run after changing `crates/codex-dev/`:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev-core --all-targets -- -D warnings
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo test -p codex-dev-core
cargo check -p codex-dev
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- --json evidence append --capsule <fixture-capsule> --kind decision --summary "fixture decision"
cargo run -q -p codex-dev -- --json capsule status <fixture-capsule>
cargo run -q -p codex-dev -- --json policy manifest
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
```

Use [Validation](../runbooks/validation.md) for the canonical local matrix and
task capsule smoke.
