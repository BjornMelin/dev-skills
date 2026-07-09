# codex-dev CLI Reference

`codex-dev` is the development operating-layer CLI for local task capsules.
It is separate from `codex-research`: research evidence stays research-owned,
while `codex-dev` records the local task capsule for a development branch.
It also plans or executes repo-native policy gates, records subspawn plan,
outcome, and synthesis evidence, captures normalized PR evidence, and records
those outcomes in the task capsule. It also owns the native Bun platform
command surface for `bun-dev` audits, safe fixes, validation, and reference
sync. The optional `codex-dev-tui` workbench
reads these same contracts for terminal scanning.
Shared capsule schemas and local read/write helpers live in
[`codex-dev-core`](codex-dev-core.md). The `codex-dev` CLI crate keeps Clap
parsing, command output, and policy subprocess execution.

Tracking: #20, #22, #23, #25, #42, #43, #44, #55, #80, #82, and #89.

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
- `research`
- `bun`
- `tool`
- `subagents`
- `orchestration`
- `policy`
- `local`
- `skills`
- `bootstrap`
- `task`
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

Research subcommands:

- `research import-bundle`

Bun subcommands:

- `bun audit`
- `bun rules list`
- `bun rules show`
- `bun fixes plan`
- `bun fixes apply`
- `bun validate plan`
- `bun validate run`
- `bun references status`
- `bun references plan`
- `bun references sync`
- `bun doctor`
- `bun benchmark`

Tool subcommands:

- `tool import`

Subagent subcommands:

- `subagents record-plan`
- `subagents record-outcome`
- `subagents record-synthesis`

Orchestration subcommands:

- `orchestration plan`
- `orchestration record`
- `orchestration close`
- `orchestration verify`

Policy subcommands:

- `policy manifest`
- `policy explain`
- `policy docs-check`
- `policy run`

Local subcommands:

- `local doctor`
- `local status`

Skills subcommands:

- `skills catalog`
- `skills inventory`
- `skills validate`
- `skills audit`
- `skills sync-kimi`

Bootstrap subcommands:

- `bootstrap status`
- `bootstrap plan`

Task subcommands:

- `task list`
- `task show`
- `task export`

PR subcommands:

- `pr agent`
- `pr agent-action`
- `pr plan`
- `pr readiness`
- `pr record`
- `pr review start`
- `pr review refresh`
- `pr review query`
- `pr review render`
- `pr review apply-suggestions`
- `pr review closeout`
- `pr status`

Review subcommands:

- `review ingest`
- `review render`
- `review query`

Commit subcommands:

- `commit plan`
- `commit validate`

Artifact commands:

- `completions <bash|elvish|fish|powershell|zsh>`
- `manpage`

## bun

Use `codex-dev bun` for the Bun platform tooling migrated from
`~/repos/cli/skill-tools`.

```bash
cargo run -q -p codex-dev -- --json bun audit --root .
cargo run -q -p codex-dev -- --json bun fixes plan --root .
cargo run -q -p codex-dev -- --json bun validate plan --root .
cargo run -q -p codex-dev -- --json bun references status
```

Native Bun results use nested schemas such as
`codex-dev.bun-audit.v1`, `codex-dev.bun-fixes.v1`,
`codex-dev.bun-validate.v1`, and `codex-dev.bun-references.v1` inside the
standard `codex-dev.output.v1` envelope. Audit cache writes are disabled by
default; pass `--write-cache` only when reusable cache entries are desired.
Safe fixes emit hashes and diffs by default, with complete before/after content
available behind `--full-content`.

`bun-platform` remains as a temporary compatibility binary. New automation
should use `codex-dev bun ...`.

## tool import

Import an external JSON report as task-capsule evidence:

```bash
cargo run -q -p codex-dev -- --json tool import \
  --capsule .codex/tasks/<task> \
  --tool bun-platform \
  --report /tmp/bun-audit.json \
  --kind output
```

The command appends a `codex-dev.evidence.v1` record and returns
`external_tool_report_import.v1` with the report schema, SHA-256 hash, evidence
record, and updated evidence summary.

## local doctor

Run a read-only local preflight for the development workstation and checkout:

```bash
cargo run -q -p codex-dev -- --json local doctor
```

The command uses the standard `codex-dev.output.v1` JSON envelope. The local
readiness contract lives at `result.schema: "codex-dev.local-doctor.v1"`. The
command checks local state only: installed `codex-dev`, `codex-dev-tui`, and
`codex-research` binaries, required tool availability, GitHub authentication
source class without printing tokens, ignored capsule root status, cache roots,
and built-in policy profile summaries. It does not run network probes, mutate
config, repair the environment, or execute policy gates. Subprocess probes run
with a sanitized environment; JSON output intentionally includes local absolute
paths for diagnostics and should be treated as workstation evidence, not a
public artifact.

GitHub config discovery follows `GH_CONFIG_DIR`, then `XDG_CONFIG_HOME/gh`, then
`HOME/.config/gh`. The global `codex-research` cache path follows
`XDG_CACHE_HOME/codex-research`, then `HOME/.cache/codex-research`.

Fixture-friendly options:

```bash
cargo run -q -p codex-dev -- --json local doctor \
  --repo-root /path/to/dev-skills \
  --checked-at 2026-05-12T05:00:00Z
```

Missing globally installed `codex-dev` binaries are warnings by default so
source validation with `cargo run` stays hermetic to the checkout. Use
`--strict-global-binaries` when validating the global install posture:

```bash
codex-dev --json local doctor --strict-global-binaries
```

`github.auth_class` is a category such as `env_token`, `gh_config`,
`gh_available_no_auth_hint`, or `gh_missing`; full secrets are never printed.
When required commands or the ignored capsule root are missing, `ok` is false
and the JSON envelope exits nonzero.

## local status

Print the same contract in compact status mode:

```bash
cargo run -q -p codex-dev -- --json local status
```

`local status` uses the same standard JSON envelope and local readiness result
schema with `result.mode: "status"` so automation can share one parser while
humans can request a status-oriented readiness summary.

## skills catalog

Emit the public Agent Skills Lab catalog artifact consumed by `bjornmelin.io`:

```bash
cargo run -q -p codex-dev -- --json skills catalog \
  --out catalog/agent-skills-lab.json
```

The command uses the standard `codex-dev.output.v1` JSON envelope. The public
artifact itself lives at `result.schemaVersion:
"agent_skills_lab_catalog.v1"` and is also written as raw JSON when `--out` is
provided. It reuses the tracked skill inventory read model, includes only valid
public skills, validates the source commit when run from a Git checkout, adds
source links from the source repository and commit, adds copyable `npx skills
add` install commands, and converts package, docs, resource, and validation data
into positive readiness labels for the portfolio marketplace. It does not
package skills, call skills.sh, run network checks, mutate portfolio files, or
expose local absolute paths.

**Resource counts read the working tree, so regenerate from a clean checkout.**
The `resources` counts (references/scripts/etc.) walk each skill's directories on
disk. Build-artifact and dependency directories are ignored automatically
(`target/`, `node_modules/`, `dist/`, `build/`, `out/`, `.next/`, `.turbo/`,
`.venv/`, `.cache/`, `.git/`, `coverage/`, `__pycache__/`, and `*.pyc/*.pyo`), but
`.gitignore`d generated files and empty local directories can still skew counts
versus a fresh checkout. CI regenerates the catalog from the pushed commit and
diffs it, so a locally polluted artifact fails the "Verify catalog artifact" gate.
When regenerating for commit, run against a clean tree — either `git stash`/clean
the worktree first, or point `--repo-root` at a detached worktree:
`git worktree add --detach /tmp/ds-clean HEAD`.

Fixture-friendly options:

```bash
cargo run -q -p codex-dev -- --json skills catalog \
  --repo-root /path/to/dev-skills \
  --generated-at 2026-05-20T08:00:00Z \
  --source-repository https://github.com/BjornMelin/dev-skills \
  --source-commit 0123456789abcdef \
  --source-ref main
```

Compact shape:

```json
{
  "schema": "codex-dev.output.v1",
  "ok": true,
  "command": "skills catalog",
  "result": {
    "schemaVersion": "agent_skills_lab_catalog.v1",
    "sourceRepository": "https://github.com/BjornMelin/dev-skills",
    "skillsCount": 57,
    "skills": [
      {
        "name": "subspawn",
        "slug": "subspawn",
        "skillMdPath": "skills/subspawn/SKILL.md",
        "readinessLabels": ["Valid", "Packaged", "Documented"],
        "installCommands": {
          "codexGlobal": "npx skills add BjornMelin/dev-skills --skill subspawn -g -a codex -y"
        }
      }
    ]
  }
}
```

`sourceCommit` defaults to `git rev-parse HEAD`. Pass it explicitly when
generating deterministic fixtures or when a workflow needs to pin links to a
known commit or release branch. In Git checkouts, the value must resolve to a
commit before catalog generation continues. New skill paths must be committed before the catalog can include them.

`sourceRef` defaults to `sourceCommit`. Use `--source-ref main` for tracked PR
artifacts that should link to the published branch after merge; the command
still validates catalog paths against the supplied `--source-commit`.

## skills inventory

Emit a read-only machine-readable inventory of tracked skill folders:

```bash
cargo run -q -p codex-dev -- --json skills inventory
```

The command uses the standard `codex-dev.output.v1` JSON envelope. The skill
inventory contract is owned by `codex-dev-core` and lives at
`result.schema: "skill_inventory.v1"`. It walks
immediate non-symlinked `skills/*/SKILL.md` entrypoints, parses bounded shallow
AgentSkills frontmatter fields, counts optional non-symlinked `references/`,
`scripts/`, `assets/`, `templates/`, and `agents/` resources with bounded depth
and entry-count caps, checks README and `docs/index.md` mention/link exposure
heuristics from regular non-symlinked files, reports local
`skills/dist/<skill>.skill` bundle presence using the frontmatter name only
when it is valid and directory-matching and otherwise falls back to the
directory name, and emits underbuilt signals for planning. The
`invalid_frontmatter` signal mirrors blocking validation failure; the other
signals are non-blocking buildout hints. It does not run `quick_validate.py`,
package skills, write bundles, mutate docs, or execute network checks.

Fixture-friendly options:

```bash
cargo run -q -p codex-dev -- --json skills inventory \
  --repo-root /path/to/dev-skills \
  --checked-at 2026-05-12T08:00:00Z
```

Compact shape:

```json
{
  "schema": "codex-dev.output.v1",
  "ok": true,
  "command": "skills inventory",
  "result": {
    "schema": "skill_inventory.v1",
    "total": 55,
    "valid": 55,
    "invalid": 0,
    "skills": [
      {
        "name": "subspawn",
        "path": "skills/subspawn",
        "skill_md": "skills/subspawn/SKILL.md",
        "exposure": {
          "readme_catalog": true,
          "docs_index": true
        },
        "package": {
          "path": "skills/dist/subspawn.skill",
          "present": true,
          "rejected": false
        },
        "validation": {
          "valid": true,
          "errors": []
        }
      }
    ]
  }
}
```

Validation is intentionally a shallow Rust subset of the durable public
frontmatter checks used by `tools/skill/quick_validate.py`: required string
`name` and `description`, allowed frontmatter keys, hyphen-case names matching
the directory, and non-empty descriptions without angle brackets. The Python
validator and packager remain the authorities for full spec validation and
`.skill` archive creation; this command owns the read-only inventory report.

## skills validate

Validate skill entrypoints with the same bounded Rust frontmatter checks used by
`skills inventory`, but with blocking validation intent:

```bash
cargo run -q -p codex-dev -- --json skills validate
cargo run -q -p codex-dev -- --json skills validate --skills-root ~/.agents/skills
```

`skills validate` emits `result.schema: "skill_inventory.v1"`. It exits nonzero
when any skill has invalid frontmatter, the skills root is missing or unsafe, or
the inventory contains error diagnostics. Use it for installed global roots when
the Python `quick_validate.py` loop would be slower or more token-heavy. Use the
Python validator before packaging when exact package-tool parity matters.

## skills audit

Audit skill hygiene beyond required frontmatter:

```bash
cargo run -q -p codex-dev -- --json skills audit
cargo run -q -p codex-dev -- --json skills audit --skills-root ~/.agents/skills
```

`skills audit` emits `result.schema: "skill_audit.v1"`. It checks validation
diagnostics, missing `agents/openai.yaml`, oversized `SKILL.md` files, stale
skill path patterns such as `~/.codex/skills`, and generated Python cache files
under bundled scripts. It also validates `archive/skills/<skill>/archive.json`
manifests without adding archived skills to the active inventory, flags archived
skills that still exist under `skills/`, missing active replacements, name
mismatches, invalid source/archive paths, missing archive reasons, missing
restore guidance, and active-catalog references. Archive `source_path` accepts
either `skills/<skill>` or plugin-origin paths shaped as
`plugins/<plugin>/skills/<skill>`, where `<plugin>` must use the same
hyphen-case skill-name syntax. `--max-skill-md-lines` defaults to `500`.

The archive summary is reported at `result.archive` with schema
`skill_archive.v1`, root, total archived skill count, and manifest-derived skill
entries. `skills inventory` and `skills validate` remain active-skill only.

## skills sync-kimi

Generate a Kimi Code skill mirror from the effective Codex skill set:

```bash
cargo run -q -p codex-dev -- --json skills sync-kimi \
  --dry-run \
  --project-root /path/to/project
```

The command uses `result.schema: "codex-dev.kimi-sync.v1"`. By default it
plans a focused mirror that includes enabled global skills, enabled
project-local skills from the requested project root, and enabled skills from
the focused Codex plugins: Clerk, Expo, Native Motion, Vercel, and Web Motion.
It reads `~/.codex/config.toml` `[[skills.config]]` rules and only mirrors
skills that remain enabled in Codex. Skill rules may match by plain global
skill name, plugin namespaced skill name such as `vercel:nextjs`, the skill
directory path, or the `SKILL.md` path. Later matching rules win, matching the
Codex config order.

Apply the mirror:

```bash
cargo run -q -p codex-dev -- --json skills sync-kimi \
  --apply \
  --project-root /path/to/project
```

Applied syncs write only under
`~/.kimi-code/codex-sync/effective/<project-hash>/`. The generated `skills/`
directory is a symlink mirror into the existing Codex/global/project skill
folders, so it does not duplicate plugin installations or copy skill content.
The `launchCommand` field is the Kimi invocation that constrains startup
loading:

```bash
kimi --skills-dir ~/.kimi-code/codex-sync/effective/<project-hash>/skills
```

Using `--skills-dir` is intentional: it makes Kimi load the generated effective
mirror instead of auto-loading every skill under `~/.agents/skills`. The command
also always excludes the Vercel plugin `shadcn` skill so the newer global
official `shadcn` skill can be used when it is enabled in Codex.

Install the convenience wrapper:

```bash
cargo run -q -p codex-dev -- --json skills sync-kimi \
  --apply \
  --install-wrapper \
  --project-root /path/to/project
```

The wrapper is installed as `~/.local/bin/kimi-codex` unless `--wrapper-path`
is supplied. It refreshes the mirror for the current working directory and then
execs Kimi with the generated `--skills-dir`:

```bash
cd /path/to/project
kimi-codex
```

Other scopes are available when needed:

```bash
cargo run -q -p codex-dev -- --json skills sync-kimi --scope all-enabled
cargo run -q -p codex-dev -- --json skills sync-kimi --scope global-only
```

`all-enabled` includes all enabled Codex plugins instead of the focused plugin
set. `global-only` skips plugin discovery and is useful for fixture or
workstation-global checks. `--launch` is interactive, requires `--apply`, and
cannot be combined with `--json`; pass Kimi arguments after `--`.

## bootstrap status

Emit a read-only machine-readable report for tracked repo bootstrap packs:

```bash
cargo run -q -p codex-dev -- --json bootstrap status
```

The command uses the standard `codex-dev.output.v1` JSON envelope with
`result.schema: "bootstrap_status.v1"`. It inspects `bootstrap/packs/*.json`,
checks the expected `dev-skills.bootstrap-pack.v1` schema, validates safe
relative targets and templates under `bootstrap/templates`, reports composed
skills and subagent source metadata, and includes the current
`bootstrap_install` policy gate IDs. It does not render files, install
subagents, run advisory host checks, or mutate any local state.
Local repository paths are redacted by default as `"<repo-root>"`; pass
`--include-local-paths` only for private workstation diagnostics.

Fixture-friendly options:

```bash
cargo run -q -p codex-dev -- --json bootstrap status \
  --repo-root /path/to/dev-skills \
  --checked-at 2026-05-12T09:00:00Z
cargo run -q -p codex-dev -- --json bootstrap status --pack codex-agent-repo
```

## bootstrap plan

Emit a read-only dry-run action plan for one bootstrap pack:

```bash
cargo run -q -p codex-dev -- --json bootstrap plan \
  --pack codex-agent-repo \
  --out "$tmp/codex" \
  --repo-name codex-smoke \
  --primary-language rust
```

The command uses `result.schema: "bootstrap_plan.v1"`. It reuses the status
validation path, then classifies each pack target as `would_write` or
`would_overwrite` against the requested output directory. Local absolute output
paths are redacted by default as `"<bootstrap-out>"`; pass
`--include-local-paths` only for private workstation diagnostics. The same flag
also includes the local repository root. The Python renderer remains the writer
of rendered files and owns `--force`, template substitution, and final output
bytes.

## task list

Emit a read-only machine-readable index of local task capsules under a task
root:

```bash
cargo run -q -p codex-dev -- --json task list
cargo run -q -p codex-dev -- --json task list --root .codex/tasks
```

The command uses the standard `codex-dev.output.v1` JSON envelope. The task
index contract lives at `result.schema: "task_index.v1"` and exposes a
structured `result.root_status` value of `ready`, `missing`, or `unusable`. It
scans only immediate entries under a ready task root, refuses to traverse
symlinked roots or task entries, validates each capsule with
`codex_dev_core::validate_capsule`, and embeds the same status summary used by
`capsule status` for valid entries. It does not run validation commands, read
providers, call GitHub, or mutate the capsule directory.

Missing task roots are reported as diagnostics with an empty task list so new
checkouts can run the smoke command before any local capsule exists. Invalid
task entries are included in `result.tasks` with `valid: false`, `errors`, and
no embedded `capsule` status. `ok` is false when any indexed entry is invalid
or `root_status` is `unusable`; a missing root remains `ok: true` for fresh
checkout smoke compatibility.

## task show

Show one task capsule by task ID under the root or by explicit path:

```bash
cargo run -q -p codex-dev -- --json task show <task-id>
cargo run -q -p codex-dev -- --json task show --root .codex/tasks <task-id>
cargo run -q -p codex-dev -- --json task show /path/to/task-capsule
```

The result uses `task_index.v1` and contains one `task` entry with validation
errors or the embedded status summary. Relative single-segment selectors are
resolved under `--root`; absolute paths and multi-segment relative paths are
treated as explicit capsule paths.

## task export

Export one valid task capsule as one JSON object for automation, TUI fixtures,
or review handoff:

```bash
cargo run -q -p codex-dev -- --json task export <task-id>
cargo run -q -p codex-dev -- --json task export --root .codex/tasks <task-id>
```

The result uses `task_index.v1` and includes the validated task entry plus the
local contract payloads from `capsule.json`, `evidence.jsonl`,
`verification.json`, `subagents.json`, `pr.json`, `policy.json`, and the human
Markdown files `plan.md`, `decisions.md`, `output.md`, and `retrospective.md`.
`task export` exits nonzero if the selected capsule is invalid because the full
contract bundle would be incomplete or unsafe to consume.

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

## research import-bundle

Import sanitized `codex-research` closeout metadata into a task capsule without
reading provider payloads, ledger bodies, cache entries, or report Markdown:

```bash
cargo run -q -p codex-dev -- --json research import-bundle \
  --capsule .codex/tasks/<id> \
  --bundle .codex/research/evidence-bundle.json \
  --source-command "codex-research --json bundle --strict" \
  --source-exit-code 0
```

The command accepts `codex-research.evidence-bundle.v1` input and emits
`research_evidence_import.v1`. It appends one `research` evidence record to
`evidence.jsonl` with bounded, defensively redacted metadata:

- namespaced source and claim IDs from the bundle ledger;
- the report path and bundle artifact paths reported by `codex-research`;
- a bounded confidence score derived from citation coverage and capped when
  failures, warnings, provider errors, missing reports, or unknown source
  freshness remain;
- residual-risk text summarizing bundle failures, warnings, provider errors,
  unknown source IDs, and uncited claims.

The importer treats the bundle file as untrusted local JSON: accepted free-form
text fields are trimmed, control-character cleaned, secret-like values redacted,
and long lists capped before JSON output or evidence persistence.

`--source-command` and `--source-exit-code` describe the source
`codex-research` command when known; `--source-exit-code` requires
`--source-command`. `--imported-at <RFC3339>` pins the import timestamp for
deterministic replay. Wrong bundle schemas, invalid capsules, or invalid
derived evidence records fail before writing.

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
  --agent-id agent-reviewer-1 \
  --status completed \
  --wait-status completed \
  --wait-elapsed-ms 1500 \
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
the capsule can prove what output or artifact was assessed. `--agent-id`,
`--wait-status`, and `--wait-elapsed-ms` are optional metadata used by the
`orchestration_run.v1` projection.

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

## orchestration plan/record/close/verify

`orchestration` is the stable operator-facing projection over the local
subspawn ledger. It records the same capsule evidence as `subagents record-*`,
then emits `result.schema: "orchestration_run.v1"` with expected roles, recorded
agent IDs, wait results, completion coverage, synthesis status, registry issues,
registry issue diagnostics, stale-evidence warnings, and missing-agent
diagnostics.

```bash
cargo run -q -p codex-dev -- --json orchestration plan \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --source /tmp/pre-pr-review-plan.json \
  --recorded-at 2026-05-09T06:00:00Z

cargo run -q -p codex-dev -- --json orchestration record \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --role reviewer \
  --agent-id agent-reviewer-1 \
  --status completed \
  --wait-status completed \
  --wait-elapsed-ms 1500 \
  --summary "no blocking findings" \
  --disposition accepted \
  --human-verified \
  --source-id reviewer:1 \
  --artifact review-notes.md

cargo run -q -p codex-dev -- --json orchestration close \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review \
  --status completed \
  --summary "review batch clean" \
  --human-verified \
  --source-id synthesis:pre-pr-review \
  --artifact review-summary.md

cargo run -q -p codex-dev -- --json orchestration verify \
  --capsule .codex/tasks/<id> \
  --batch-id pre-pr-review
```

The command family is local and explicit: it does not spawn agents, wait on
agents, call hosted APIs, or resolve GitHub review state. `plan`, `record`, and
`close` succeed when the write is valid even if the run is not complete yet.
`verify` exits nonzero until every expected role has a terminal
human-verified outcome and the parent synthesis is `completed`. Incomplete runs
include blocking diagnostics such as `incomplete_agent` and `missing_synthesis`;
older incomplete runs also include `stale_orchestration_evidence` warnings based
on `--stale-after-minutes` (default: 120). Planner-level `registry_issues` are
preserved and also emitted as `registry_issue` warnings.

Supported wait statuses are `pending_init`, `running`, `completed`, `errored`,
`interrupted`, `shutdown`, `not_found`, `timed_out`, and `not_waited`. Runtime
agent IDs are optional while a run is planned, but any non-`planned` role
without `--agent-id` produces a `missing_agent_id` warning in
`orchestration_run.v1`.

## policy manifest

Print the built-in repo-native gate manifest:

```bash
# codex-dev:policy-manifest-smoke:start
cargo run -q -p codex-dev -- --json policy manifest --profile codex_dev
cargo run -q -p codex-dev -- --json policy explain --profile codex_dev
cargo run -q -p codex-dev -- --json policy manifest --profile full_local
cargo run -q -p codex-dev -- --json policy explain --profile full_local
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
Aggregated profiles keep their inherited gates and add profile-specific
`policy explain` smokes where inheritance would otherwise only exercise
`codex_dev`.

## policy explain

Explain a policy profile without executing gates:

```bash
cargo run -q -p codex-dev -- --json policy explain --profile full_local
```

The output is wrapped in the standard `codex-dev.output.v1` envelope and uses
`result.schema: "policy_explain.v1"`. It is read-only: it reuses the Rust-owned
manifest, reads the documentation mirror, probes required tools on `PATH`, and
does not run any policy gate command. Local absolute repository and tool paths
are omitted by default; pass `--include-local-paths` only when diagnosing local
workstation setup and the JSON will not be pasted into hosted review evidence.

The report includes the gate purpose, source, rendered command, required tools,
missing local prerequisites, network/secrets posture, expected artifacts, docs
mirror status, and failure interpretation for each gate. Use it before
`policy run --execute` when deciding which profile fits a branch or when an
installed binary is being used outside the checkout; pass `--repo-root <path>`
when documentation mirror discovery would otherwise be ambiguous.

## policy docs-check

Check machine-owned Markdown mirrors of policy manifest and policy explain
commands against the Rust-owned profile list:

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
network- and secrets-marked because they use live GitHub auth and hosted PR
state. `codex-dev pr review` is the canonical review-remediation command
surface; `$gh-pr-review-fix` is the skill workflow that runs it. Commands that
need caller-supplied artifacts set `manual_input`; for example closeout requires
the worklist path emitted by `codex-dev pr review start`, the pushed head SHA,
semantic fix commit SHA, and passed validation command before `--apply`.
The plan includes JSON-producing `gh pr view`, `gh pr checks`, REST review,
REST review-comment, and GraphQL review-thread commands whose saved outputs can
be passed to `pr record` with the matching `--source-kind`. The GraphQL
review-thread command uses `--paginate --slurp` with an `$endCursor` query, so
complete multi-page thread sets can be recorded from one saved JSON artifact.

## pr review

Capture, query, patch, and close hosted PR review work with stable JSON:

```bash
cargo run -q -p codex-dev -- --json pr review start \
  --repo BjornMelin/dev-skills \
  --number 25 \
  --fresh
```

`pr review start` and `refresh` emit `codex-dev.pr-review-worklist.v1`, write
`pr-review-worklist.json` in the task capsule, include the canonical
`worklist_path` in the JSON result, include `out_path` when `--out` writes a
second copy, and preserve raw hosted evidence captured by `pr agent`. `query`
and `render` inspect that worklist without network calls. `apply-suggestions`
is dry-run unless `--apply` is passed and rewrites only exact GitHub suggestion
fences whose current file hunk still matches. When `--item` is omitted,
`apply-suggestions` processes only `actionable` worklist items; pass `--item` to
target one captured item explicitly. `closeout` emits
`codex-dev.pr-review-closeout.v1`; it is dry-run by default and resolves
current matching hosted threads only when `--apply` revalidates fresh PR
head/thread state.

## review

Normalize local review artifacts that are not hosted GitHub threads:

```bash
cargo run -q -p codex-dev -- --json review ingest \
  --source /tmp/review.md \
  --kind manual \
  --out /tmp/review-worklist.json
```

The output schema is `codex-dev.review-worklist.v1`. Use `review render` and
`review query` for local Codex, Zen, or manual notes before fixing findings.

## commit plan and validate

Plan and validate scoped semantically reviewable Conventional Commits:

```bash
cargo run -q -p codex-dev -- --json commit plan --worklist /tmp/pr-review-worklist.json
cargo run -q -p codex-dev -- --json commit validate \
  --subject "fix(codex-dev): preserve review-thread closeout evidence"
```

`commit plan` emits `codex-dev.commit-plan.v1` with semantic groups, scopes,
files, source work items, SemVer impact, and recommended validation commands.
`commit validate` emits `codex-dev.commit-validation.v1` and rejects subjects
that describe review process instead of behavior, such as `address review
feedback` or `address PR review comments`.

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
cargo run -q -p codex-dev -- --json policy explain --profile full_local
cargo run -q -p codex-dev -- --json policy docs-check
cargo run -q -p codex-dev -- --json local doctor
cargo run -q -p codex-dev -- --json local status
cargo run -q -p codex-dev -- --json skills inventory
cargo run -q -p codex-dev -- --json skills catalog --out /tmp/agent-skills-lab.json
cargo run -q -p codex-dev -- --json skills validate
cargo run -q -p codex-dev -- --json skills audit
cargo run -q -p codex-dev -- --json task list
cargo run -q -p codex-dev -- --json pr plan --repo BjornMelin/dev-skills --number 25
cargo run -q -p codex-dev -- --json pr agent --help
cargo run -q -p codex-dev -- --json pr agent-action --help
cargo run -q -p codex-dev -- --json pr review --help
cargo run -q -p codex-dev -- --json pr readiness --help
cargo run -q -p codex-dev -- --json pr record --help
cargo run -q -p codex-dev -- --json review --help
cargo run -q -p codex-dev -- --json commit plan --help
cargo run -q -p codex-dev -- --json commit validate --help
```

Use [Validation](../runbooks/validation.md) for the canonical local matrix,
including the `orchestration plan/record/close/verify` fixture smoke and task
capsule smoke. Use [Local Release and Supply Chain](../runbooks/local-release-supply-chain.md)
for release/install supply-chain evidence.
