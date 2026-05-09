# codex-dev CLI Reference

`codex-dev` is the development operating-layer CLI for local task capsules.
It is separate from `codex-research`: research evidence stays research-owned,
while `codex-dev` records the local task capsule for a development branch.
It also plans or executes repo-native policy gates, captures normalized PR
evidence, and records those outcomes in the task capsule. Future lanes add
bootstrap packs and TUI surfaces.

Tracking: #20, #22, #23, and #25.

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
- `policy`
- `pr`

Capsule subcommands:

- `capsule init`
- `capsule validate`
- `capsule status`
- `capsule render`

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
`updated_at`, but does not execute commands.

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
and capture the normalized result.

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

`pr record` writes `pr.json`, appends review evidence to `evidence.jsonl`,
updates `capsule.json.updated_at`, and adds the PR number to
`capsule.json.pull_requests` when it is not already present.

## pr status

Print the PR snapshot currently stored in the capsule:

```bash
cargo run -q -p codex-dev -- pr status --capsule .codex/tasks/<id>
```

## Validation

Run after changing `crates/codex-dev/`:

```bash
cargo fmt --all --check
cargo clippy -p codex-dev --all-targets -- -D warnings
cargo check -p codex-dev
cargo test -p codex-dev
cargo run -q -p codex-dev -- --help
cargo run -q -p codex-dev -- --json policy manifest
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
```

Use [Validation](../runbooks/validation.md) for the canonical local matrix and
task capsule smoke.
