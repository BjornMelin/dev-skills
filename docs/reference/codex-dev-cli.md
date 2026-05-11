# codex-dev CLI Reference

`codex-dev` is the development operating-layer CLI for local task capsules.
It is separate from `codex-research`: research evidence stays research-owned,
while `codex-dev` records the local task capsule for a development branch.
It also plans or executes repo-native policy gates, records subspawn plan,
outcome, and synthesis evidence, captures normalized PR evidence, and records
those outcomes in the task capsule. The optional `codex-dev-tui` workbench
reads these same contracts for terminal scanning.
Shared capsule schemas and local read/write helpers live in
[`codex-dev-core`](codex-dev-core.md). The `codex-dev` CLI crate keeps Clap
parsing, command output, and policy subprocess execution.

Tracking: #20, #22, #23, #25, #42, #43, #44, and #55.

## Installation

From the repository root:

```bash
cargo build -p codex-dev
cargo install --path crates/codex-dev --locked --force
cargo run -q -p codex-dev -- --help
```

Use [Global CLI Workflow](../runbooks/global-cli-workflow.md) for the
three-binary install/update workflow, completions, manpages, and isolated
install smokes.

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
- `subagents`
- `policy`
- `pr`
- `completions`
- `manpage`

Capsule subcommands:

- `capsule init`
- `capsule validate`
- `capsule status`
- `capsule render`

Evidence subcommands:

- `evidence append`

Subagent subcommands:

- `subagents record-plan`
- `subagents record-outcome`
- `subagents record-synthesis`

Policy subcommands:

- `policy manifest`
- `policy docs-check`
- `policy run`

PR subcommands:

- `pr agent`
- `pr agent-action`
- `pr plan`
- `pr readiness`
- `pr record`
- `pr status`

Artifact commands:

- `completions <bash|elvish|fish|powershell|zsh>`
- `manpage`

## completions

Generate shell completions from the canonical Clap command definition:

```bash
cargo run -q -p codex-dev -- completions zsh > /tmp/_codex-dev
codex-dev completions bash > ~/.local/share/dev-skills/completions/bash/codex-dev
codex-dev completions fish > ~/.local/share/dev-skills/completions/fish/codex-dev.fish
```

Without `--json`, the command writes the completion script directly to stdout
and does not modify shell startup files. With global `--json`, the same content
is wrapped in the standard output envelope at `result.content`.

## manpage

Generate a roff manpage from the canonical Clap command definition:

```bash
cargo run -q -p codex-dev -- manpage > /tmp/codex-dev.1
codex-dev manpage > ~/.local/share/man/man1/codex-dev.1
```

Without `--json`, the command writes roff directly to stdout and does not
install it automatically. With global `--json`, the same content is wrapped in
the standard output envelope at `result.content`.

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

Validate required files, JSON schema identifiers, and typed contract semantics:

```bash
cargo run -q -p codex-dev -- --json capsule validate .codex/tasks/<id>
```

Invalid capsules exit nonzero. With `--json`, the command still prints a
`codex-dev.output.v1` envelope with `ok: false` and `result.valid: false`.
Validation is intentionally strict: every required capsule file must exist, and
contract files such as `subagents.json`, `pr.json`, and `policy.json` must keep
their documented schema identifiers and value invariants.

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
- `--residual-risk <text>` records known risks or follow-up risk.
- `--artifact <path-or-id>` may be repeated for local artifacts.

The command validates the record before writing. Invalid records fail nonzero
with a typed JSON error envelope under `--json` and do not append to
`evidence.jsonl`. Empty text, control characters, empty repeated values, an
`--exit-code` without `--command`, and out-of-range confidence are rejected.
The command also rejects symlinked JSON/JSONL capsule contract files before
validation or writing. Successful appends update `capsule.json.updated_at`
monotonically; backfilled evidence does not move the capsule timestamp
backwards.

## subagents record-plan

Record a `subspawn_plan.py --json` output into `subagents.json` and append a
subagent evidence record:

```bash
python3 skills/subspawn/scripts/subspawn_plan.py plan \
  --preset review \
  --task "pre-PR branch review" \
  --scope "current branch diff" \
  --json > /tmp/pre-pr-review-plan.json

cargo run -q -p codex-dev -- --json subagents record-plan \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --source /tmp/pre-pr-review-plan.json \
  --command "python3 skills/subspawn/scripts/subspawn_plan.py plan --preset review --json" \
  --recorded-at 2026-05-09T06:00:00Z
```

`codex-dev` does not spawn agents. It reads the planner JSON, validates the
batch ID, role names, non-empty task text, duplicate-role warning path lists,
and one prompt per role. Duplicate prompt rows and prompts for unplanned roles
are rejected instead of normalized. The recorder preserves
`duplicate_roles_ignored` and `registry_issues`, stores stable prompt IDs, and
stores SHA-256 prompt hashes. It does not store raw prompt text in
`subagents.json`; keep the source plan as a local artifact when the full prompt
is needed.

## subagents record-outcome

Record one planned agent's outcome, disposition, and supporting references:

```bash
cargo run -q -p codex-dev -- --json subagents record-outcome \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --role reviewer \
  --status completed \
  --summary "no blocking findings" \
  --disposition accepted \
  --human-verified \
  --source-id reviewer:1 \
  --artifact review-notes.md \
  --recorded-at 2026-05-09T06:10:00Z
```

Supported outcome statuses are `planned`, `running`, `completed`, `failed`,
`timed_out`, `closed`, and `blocked`. Supported dispositions are `accepted`,
`rejected`, `mixed`, `informational`, and `pending`. The command requires
`--human-verified` so capsules distinguish agent output from parent-session
judgment. `--source-id` and `--artifact` must each be provided at least once so
the capsule can prove what output or artifact was assessed.

## subagents record-synthesis

Record the parent synthesis for a batch:

```bash
cargo run -q -p codex-dev -- --json subagents record-synthesis \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --status completed \
  --summary "review batch clean after follow-up fixes" \
  --human-verified \
  --source-id synthesis:pre-pr-review \
  --artifact review-summary.md \
  --recorded-at 2026-05-09T06:20:00Z
```

Supported synthesis statuses are `completed`, `partial`, and `blocked`.
`completed` synthesis is accepted only after every planned role has a terminal
human-verified outcome (`completed`, `failed`, `timed_out`, or `closed`) and a
final disposition. Outcome and synthesis commands require at least one
`--source-id` and one `--artifact`, update `subagents.json`, append `subagent`
evidence to `evidence.jsonl`, and update `capsule.json.updated_at`
monotonically. They reject invalid capsules and symlinked JSON/JSONL contract
files before validation or writing.

## policy manifest

Print the built-in repo-native gate manifest:

```bash
# codex-dev:policy-manifest-smoke:start
cargo run -q -p codex-dev -- --json policy manifest --profile codex_dev
cargo run -q -p codex-dev -- --json policy manifest --profile full_local
# codex-dev:policy-manifest-smoke:end
```

The default profile is `codex_dev`. Supported profiles are:

- `codex_dev`
- `codex_dev_tui`
- `codex_research`
- `skills`
- `bootstrap_install`
- `docs`
- `release`
- `full_local`

The manifest is versioned as `codex-dev.policy-gates.v1` and ties each gate to
its tracked runbook source. Each gate records the command, working directory,
required tools, required/network/secrets flags, and failure interpretation.
Built-in profiles are local by default and do not require secrets or network
access; networked advisory checks stay explicit in the release runbook.

## policy docs-check

Check machine-owned Markdown mirrors of policy manifest commands against the
Rust-owned profile list:

```bash
cargo run -q -p codex-dev -- --json policy docs-check
```

The checker reads marked `codex-dev:policy-manifest-*` snippets in AGENTS.md,
README.md, this CLI reference, and [Validation](../runbooks/validation.md). It
is read-only and exits nonzero when any mirror drifts from the Rust policy
profiles.

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
  --profile codex_dev \
  --execute
```

Executed required-gate failures set `ok: false` and exit nonzero. Use
`--keep-going` to continue after a failed required gate. Gates marked as
network-using are skipped unless `--allow-network` is passed. Gates marked as
secret-using are skipped unless `--allow-secrets` is passed. The built-in local
profiles currently have no network or secret gates.

Execution discovers the repository root from the current directory or capsule
path before running repo-native commands. Pass `--repo-root <path>` for
installed-binary workflows where discovery would be ambiguous. If the capsule
path is inside one repo and the current directory is inside another, execution
fails until `--repo-root` makes the target explicit. Gate `working_directory`
values are repo-relative and cannot escape the selected root.

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
The plan includes JSON-producing `gh pr view`, `gh pr checks`, REST review,
REST review-comment, and GraphQL review-thread commands whose saved outputs can
be passed to `pr record` with the matching `--source-kind`. The GraphQL
review-thread command uses `--paginate --slurp` with an `$endCursor` query, so
complete multi-page thread sets can be recorded from one saved JSON artifact.

## pr agent

Gather live hosted PR state, normalize it into the capsule, and print a
deterministic dry-run action plan:

```bash
cargo run -q -p codex-dev -- --json pr agent \
  --capsule .codex/tasks/<id> \
  --repo BjornMelin/dev-skills \
  --number 25
```

The output schema is `codex-dev.pr-agent-state.v1`. The command is always a
hosted-write dry run: it can run `gh` read commands and write local capsule
evidence, but it does not resolve threads, comment, retry CI, enable auto-merge,
or merge the PR. It records:

- normalized PR state in `pr.json`;
- raw captured provider JSON under `pr-agent-sources/<timestamp>/`;
- a `pr-agent-state.json` report with source records, diagnostics, and
  recommended next actions;
- a `decision` evidence row in `evidence.jsonl`.

Live collection uses these read-only sources:

- `gh pr view --json number,url,state,isDraft,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid,headRefName,baseRefName,baseRefOid,updatedAt,labels`
- `gh pr checks --json bucket,completedAt,description,event,link,name,startedAt,state,workflow`
- `gh api --paginate --slurp repos/<owner>/<repo>/pulls/<number>/reviews?per_page=100`
- `gh api --paginate --slurp repos/<owner>/<repo>/pulls/<number>/comments?per_page=100`
- `gh api graphql --paginate --slurp` for `reviewThreads(first:100, after:$endCursor)`
- `gh api rate_limit`

Command failures, malformed JSON, missing permissions, non-authoritative
pagination, and low rate-limit state are surfaced as diagnostics. A failed
source does not make the agent infer clean review or CI state from stale data.
Use replay mode for deterministic tests or manual evidence review:

```bash
cargo run -q -p codex-dev -- --json pr agent \
  --capsule .codex/tasks/<id> \
  --repo BjornMelin/dev-skills \
  --number 25 \
  --source-dir /tmp/captured-pr-sources
```

The replay directory uses the same filenames written by live mode:
`gh-pr-view.json`, `gh-pr-checks.json`, `gh-reviews.json`,
`gh-review-comments.json`, `gh-review-threads.json`, and `gh-rate-limit.json`.

## pr agent-action

Plan or apply one explicit hosted PR action:

```bash
cargo run -q -p codex-dev -- --json pr agent-action \
  --capsule .codex/tasks/<id> \
  --repo BjornMelin/dev-skills \
  --number 25 \
  --plan-id reply-coderabbit-stale-thread \
  --action reply-review-comment \
  --review-comment-id 123456789 \
  --body "@coderabbitai Verified against current code; this thread is stale."
```

The output schema is `codex-dev.pr-agent-hosted-action.v1`. Without `--apply`,
the command captures fresh PR state, writes
`pr-agent-actions/<plan-id>/before-state.json`, writes
`pr-agent-actions/<plan-id>/plan.json`, appends a `decision` evidence row, and
does not perform a hosted mutation. With `--apply`, the command rejects
`--source-dir`, captures live state, executes the planned hosted command, writes
after-state evidence when the command applies or is skipped as a duplicate, and
appends a `review` evidence row.

Supported actions:

- `post-issue-comment`: requires `--body` or `--body-file`.
- `reply-review-comment`: requires `--review-comment-id` plus `--body` or
  `--body-file`. GitHub only supports replies to top-level review comments.
- `resolve-review-thread`: requires `--thread-id`.
- `unresolve-review-thread`: requires `--thread-id`.
- `add-labels`: requires one or more `--label`.
- `remove-labels`: requires one or more `--label`.
- `rerun-failed-jobs`: requires `--run-id`.

Every action requires explicit `--repo`, `--number`, and `--plan-id`. Hosted
writes require `--apply`; dry-run mode may use `--source-dir` with the same
source filenames as `pr agent`. Apply mode fails closed when required live
state capture has error diagnostics. Comment and review-reply actions append a
hidden `codex-dev-pr-agent:<plan-hash>` marker and perform a duplicate check
before applying so re-running the same plan does not post duplicate comments.
Thread and label actions verify the requested target is present in current PR
state and skip if it is already in the desired state. Failed-job reruns fetch
the workflow run first, require same-repository head identity, allowed workflow
events, matching PR head branch/SHA, and a bound workflow-run URL before
POSTing; non-failed or still-running runs are skipped instead of rerun.

Permission diagnostics are advisory and local. The report records whether
`GITHUB_TOKEN` or `GH_TOKEN` is visible to the process and lists the GitHub
permission class expected by the selected action. Actual authorization remains
with GitHub and failed hosted commands are recorded with redacted stderr
excerpts.

## pr readiness

Evaluate whether a PR is ready to close out, rerun failed jobs, or merge:

```bash
cargo run -q -p codex-dev -- --json pr readiness \
  --capsule .codex/tasks/<id> \
  --repo BjornMelin/dev-skills \
  --number 25 \
  --poll-attempts 3 \
  --poll-interval-seconds 60 \
  --merge
```

The output schema is `codex-dev.pr-agent-readiness.v1`. The command uses the
same live or replayed sources as `pr agent`, writes `pr-readiness.json` and
`pr-readiness.md`, and appends a `decision` evidence row. It evaluates:

- check state, allowlisted conclusions, and GitHub Actions run IDs parsed only
  from same-repository check URLs;
- authoritative hosted review-thread state separately from `reviewDecision`;
- stale/outdated review comments without treating stale comments as unresolved
  threads;
- draft state, mergeability, `mergeStateStatus`, head SHA, and branch refs;
- final status as `ready`, `waiting`, `blocked`, `merged`, or `stopped`.

The command exits successfully only when the final status is `ready` or
`merged` and no requested hosted action failed. With `--json`, non-ready states
still emit the full readiness report before exiting nonzero.

Polling is bounded by `--poll-attempts`; there is no daemon mode. Replay mode
is deterministic and accepts `--source-dir`; apply mode rejects replayed
sources and must capture current hosted state.

Hosted mutations are opt-in. `--rerun-failed` plans reruns for failed checks
whose URLs expose GitHub Actions run IDs; adding `--apply` delegates each run to
the existing `rerun-failed-jobs` hosted action, which rechecks workflow-run
repository, event, PR binding, head branch, and head SHA before POSTing.
`--merge` plans a `gh pr merge` command only after a ready final state. Adding
`--apply` captures fresh live PR state immediately before merging, re-evaluates
readiness, and only then executes with
`--match-head-commit <fresh-head-sha>`. Merge uses `--squash` by default and
supports `--merge-method`, `--delete-branch`, `--merge-subject`, and
`--merge-body`.

Readiness deliberately distinguishes code fixed from hosted review threads
resolved. A stale `changes_requested` review decision is only downgraded to a
warning when authoritative thread state is clean; unresolved hosted threads
remain blocking even if local code has been patched.

## pr record

Record a local normalized PR snapshot fixture into a task capsule:

```bash
cargo run -q -p codex-dev -- --json pr record \
  --capsule .codex/tasks/<id> \
  --source /tmp/pr-snapshot.json \
  --checked-at 2026-05-09T05:00:00Z
```

The default `--source-kind normalized` accepts the local fixture shape below:

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

`pr record` can also normalize saved provider/tool output directly:

```bash
cargo run -q -p codex-dev -- --json pr record \
  --capsule .codex/tasks/<id> \
  --source /tmp/gh-pr-view.json \
  --source-kind gh-pr-view \
  --source-command "gh pr view 25 --repo BjornMelin/dev-skills --json number,url,state,isDraft,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid,headRefName,baseRefName,baseRefOid,updatedAt,labels"
```

Supported `--source-kind` values:

- `normalized`: existing local `pr.json` fixture shape.
- `gh-pr-view`: `gh pr view --json number,url,state,isDraft,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid,headRefName,baseRefName,baseRefOid,updatedAt,labels`.
- `gh-pr-checks`: `gh pr checks --json bucket,completedAt,description,event,link,name,startedAt,state,workflow`.
- `gh-reviews`: REST review submission arrays from `gh api repos/<owner>/<repo>/pulls/<number>/reviews`; supports single arrays and `--paginate --slurp` page arrays, then collapses to each reviewer's latest active state before deriving `review_decision`.
- `gh-review-threads`: GraphQL `reviewThreads.nodes` output from `gh api graphql`; supports single-page objects and `--paginate --slurp` page arrays, counts resolved, current unresolved, and outdated threads separately, and is authoritative only when the final `pageInfo.hasNextPage` is false.
- `gh-review-comments`: REST review-comment arrays; supports single arrays and `--paginate --slurp` page arrays, groups comments by thread root (`in_reply_to_id` or `id`) and counts threads whose current `position` is null and whose original position/line is present as outdated evidence, but does not infer unresolved thread state from REST comments alone.
- `review-pack-remaining`: JSON or text output from `review-pack remaining`; records the unresolved count.

All non-`normalized` source kinds require explicit PR identity unless it can be
derived from a GitHub pull request URL in the saved source. Pass `--repo
OWNER/REPO` and `--number PR_NUMBER` for source shapes that do not include that
URL.

All source kinds, including `normalized`, add a `sources[]` trace entry with the
source kind, `codex-dev.pr-source-parser.v1`, retrieval timestamp, source path,
and the optional `--source-command` used to fetch the saved artifact. Use
`--retrieved-at` when a saved artifact was fetched before the local record time.
Partial sources such as `gh-pr-checks` and `gh-review-comments` merge into the
existing capsule snapshot and do not mark review-thread state authoritative by
themselves.

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
cargo run -q -p codex-dev -- --json policy docs-check
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
cargo run -q -p codex-dev -- --json pr agent --help
cargo run -q -p codex-dev -- --json pr agent-action --help
cargo run -q -p codex-dev -- --json pr readiness --help
cargo run -q -p codex-dev -- --json pr record --help
```

Use [Validation](../runbooks/validation.md) for the canonical local matrix and
task capsule smoke. Use [Local Release and Supply Chain](../runbooks/local-release-supply-chain.md)
for release/install supply-chain evidence.
