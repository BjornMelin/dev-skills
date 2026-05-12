# codex-dev Operating Layer

Status: active implementation.

Tracking: #20 through #28, parent epic #37, child issues #38 through #57, and
#80.

## Purpose

`codex-dev` is the development control-plane family for this repo. The CLI
delivered by issue #22 records agent work as local task capsules. Issue #23
adds a repo-native policy gate that can plan or execute local validation while
recording the result in the capsule. Issue #24 adds offline skill and subagent
eval coverage, issue #25 adds PR evidence planning plus local normalized
snapshot recording, issue #26 adds bootstrap packs, issue #27 adds memory
proposal guidance, and issue #28 adds an optional terminal workbench.

`codex-dev` is deliberately separate from `codex-research`. The research CLI
continues to own provider routing, source hydration, research ledgers, cache,
and research evals. The new operating layer coordinates development work around
those tools instead of becoming another research provider.

The next release sequence is tracked by the
[dev-skills v0.3/v1 Roadmap](dev-skills-v0.3-roadmap.md). That roadmap is the
canonical issue ledger for parent epic #37 and child issues #38 through #57:
strict contracts, local CLI/TUI-first work, apply-gated PR-agent behavior,
audited local release, and future-surface design.
The [codex-dev PR-Agent Safety Model](codex-dev-pr-agent-safety-model.md)
defines the token, hosted-write, trust-boundary, idempotency, and
verify-before-fix rules that all future PR-agent implementation lanes must
preserve.

## Goals

- Create one canonical task capsule schema for goal, branch, PR, verification,
  subagents, decision, and evidence state.
- Keep policy gates as thin wrappers over repo-native validation commands.
- Reuse existing owners for skills, subagents, review remediation, research,
  bootstrap packs, and memory guidance.
- Make every command scriptable with stable JSON output before adding a TUI.
- Keep private local manifests, ignored overlays, secrets, and run artifacts out
  of tracked artifacts and commits.

## Non-goals

- Do not add general development commands to `codex-research`.
- Do not reimplement subagent validation outside `subagent-creator`.
- Do not reimplement delegation policy outside `subspawn`.
- Do not duplicate today's hosted review remediation flow outside
  `gh-pr-review-fix` and `review-remediation`; future PR-agent write lanes that
  orchestrate those tools must still preserve the dedicated PR-agent safety
  model.
- Do not make the optional TUI own business logic.
- Do not support compatibility shims for pre-1.0 draft capsule shapes.

## Ownership Map

| Surface | Canonical owner | `codex-dev` relationship |
| --- | --- | --- |
| Research provider routing, source cache, claim ledgers, research evals | `codex-research` | May reference sanitized source IDs or summaries as local evidence; provider calls and raw provider output remain outside `codex-dev`. |
| Task capsule contracts and local read models | `codex-dev-core` | Shared schema and file-helper owner for CLI, TUI, and future PR-agent surfaces. |
| Policy-gate orchestration and PR/eval/bootstrap evidence appenders | `codex-dev` | CLI/process boundary over `codex-dev-core` contracts. |
| Local workstation readiness | `codex-dev local doctor/status` | Read-only CLI-owned schema for PATH/tool, GitHub auth class, ignored capsule root, cache root, and policy-profile summaries. |
| Machine-readable skill inventory | `codex-dev skills inventory` | Read-only CLI-owned schema for skill metadata, bounded resource counts, README/docs exposure heuristics, package-artifact presence, validation status, and underbuilt signals. |
| Skill metadata and package validation | `tools/skill`, skill folders | Owns validation and package writes; inventory reports shallow status but does not replace validators or packagers. |
| Custom subagent template validation and installs | `subagent-creator` | Reuses validation/install commands. |
| Subagent fanout planning and wait policy | `subspawn` | Records selected plan and subagent outcomes. |
| Hosted PR review remediation | `gh-pr-review-fix`, `review-remediation` | Captures review-pack/CI snapshots and links fixes. |
| PR-agent hosted write safety | `docs/specs/codex-dev-pr-agent-safety-model.md` | Defines explicit target, dry-run, `--apply`, stale-thread, idempotency, token, and prompt-injection rules before hosted mutations exist. |
| Hardened personal subagent pack | `subagents/hardened-codex` | Treats as a bootstrap input and smoke target. |
| Memory and Codex runtime guidance | `codex-sdk` docs/skill | Links proposal docs; does not mutate runtime memory. |
| Optional terminal UI | `codex-dev-tui` | Reads `codex-dev-core` JSON contracts; never owns policy. |

## Local Doctor Contract

`codex-dev.local-doctor.v1` is the CLI-owned local readiness contract for
workstation and checkout preflight. It is deliberately outside the task capsule:
it describes the current machine, not a branch artifact that should be copied
into a PR. The command family is read-only and has no repair, network, hosted
write, or policy-execution mode.

`local doctor` and `local status` share one schema. `mode` distinguishes the
operator intent while allowing automation to use one parser. Required local
tool failures and a non-ignored capsule root make `ok` false. Globally
installed `codex-dev`, `codex-dev-tui`, and `codex-research` binaries are
warnings by default so source validation with `cargo run` remains checkout
hermetic; `--strict-global-binaries` upgrades those missing binaries to errors
when validating the global install posture.

The GitHub auth report is categorical only. It may emit source names such as
`GH_TOKEN`, `GITHUB_TOKEN`, or `gh_config`, but it must not print credential
values or run `gh auth status` as a network/auth probe. Subprocess probes run
through resolved executable paths with a sanitized environment and bounded
stdout capture.

GitHub config discovery follows `GH_CONFIG_DIR`, then `XDG_CONFIG_HOME/gh`, then
`HOME/.config/gh`. The global `codex-research` cache path follows
`XDG_CACHE_HOME/codex-research`, then `HOME/.cache/codex-research`.

## Skill Inventory Contract

`skill_inventory.v1` is the CLI-owned read-only catalog contract for
`skills/*/SKILL.md`. It describes the current checkout and is intended for
automation that needs one stable JSON surface before planning packaging,
validation, docs, or future UI/TUI work.

The command walks immediate non-symlinked skill directories, skips build
artifacts such as `skills/dist`, and emits one sorted entry per regular
`SKILL.md`. Each entry includes bounded frontmatter basics (`name`,
`description`, `license`, simple `allowed-tools`, and whether `metadata` is
present), repo-relative source paths, non-symlinked resource file counts for
`references/`, `scripts/`, `assets/`, `templates/`, and `agents/`, README and
`docs/index.md` mention/link exposure heuristics from regular non-symlinked
files, local `skills/dist/<skill>.skill` presence using the frontmatter name
only when it is valid and directory-matching and otherwise falling back to the
directory name, shallow frontmatter validation, and non-blocking underbuilt
signals.

Resource counts carry `capped: true` when the inventory hit its defensive
resource depth or file-count limit before completing a directory walk.

Validation is deliberately a shallow Rust subset of the durable public rules in
`tools/skill/quick_validate.py`: frontmatter must exist, use allowed keys, carry
string `name` and `description` values, use hyphen-case names matching the
directory, and keep descriptions non-empty and free of angle brackets. The
inventory does not shell out to Python, package skills, update docs, install
skills, or perform network checks. `tools/skill/quick_validate.py` remains the
validation authority, and `tools/skill/package_skill.py` remains the
package-writing authority.

`ok` is false only for error-severity diagnostics such as invalid skill
frontmatter or a missing skills root. `invalid_frontmatter` is also mirrored in
`underbuilt_signals` so planning consumers can sort invalid entries without
reading validation details first. Missing README exposure, missing docs
exposure, missing `references/`, missing `scripts/`, missing dist bundles, and
the absence of all three optional buildout directories (`assets/`, `templates/`,
and `agents/`) are non-blocking `underbuilt_signals` for planning, not
validation failures.

## Task Capsule Contract

Task capsules are local development artifacts. The default root is:

```text
.codex/tasks/<timestamp>-<slug>/
```

The root is intentionally local. A PR should summarize capsule evidence in its
description, not commit the capsule directory unless a future issue explicitly
requests a tracked fixture.

Minimum capsule layout:

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

Contract files are `capsule.json`, `evidence.jsonl`, `verification.json`,
`subagents.json`, `pr.json`, and `policy.json`. Markdown files are human notes
whose headings are conventional but not machine contracts; `output.md` is the
operator-facing closeout slot for rendered summaries.

Validation is strict. Every required file must exist and every JSON contract
must keep its exact schema identifier. Capsule initialization is the command
that creates the layout; follow-on commands such as `pr record` update their
owned files but do not silently repair missing contracts in an already-created
capsule. Follow-on write commands must not move `capsule.json.updated_at`
backwards when recording backfilled evidence.

### capsule.json

`capsule.json` is the canonical state file. Other files provide human-readable
or append-only evidence views.

Required fields for `codex-dev.task-capsule.v1`:

- `schema`
- `id`
- `title`
- `status`
- `objective`
- `branch`
- `base_branch`
- `issues`
- `pull_requests`
- `created_at`
- `updated_at`

Keep the example in this section synchronized with the required-field list so
implementation lanes cannot drift from the documented validation contract.

```json
{
  "schema": "codex-dev.task-capsule.v1",
  "id": "20260509-032500-codex-dev-operating-layer",
  "title": "Define codex-dev operating layer",
  "status": "active",
  "objective": "Define the architecture and task capsule contract.",
  "branch": "docs/codex-dev-operating-layer-contract",
  "base_branch": "main",
  "issues": [21],
  "pull_requests": [],
  "created_at": "2026-05-09T03:25:00Z",
  "updated_at": "2026-05-09T03:25:00Z"
}
```

Allowed `status` values:

- `active`
- `blocked`
- `ready_for_pr`
- `in_review`
- `merged`
- `closed`

### evidence.jsonl

`evidence.jsonl` is append-only. Each line is a JSON object:

```json
{
  "schema": "codex-dev.evidence.v1",
  "kind": "command",
  "at": "2026-05-09T03:30:00Z",
  "summary": "docs link check passed",
  "command": "python3 tools/docs/check_links.py docs README.md AGENTS.md",
  "exit_code": 0,
  "source_ids": ["validation:docs-links"],
  "actor": "codex",
  "tool": "codex-dev",
  "confidence": 100,
  "residual_risk": "none identified",
  "artifacts": []
}
```

Allowed `kind` values:

- `command`
- `subagent`
- `review`
- `ci`
- `decision`
- `research`
- `manual`
- `output`

Optional evidence metadata:

- `command` and `exit_code` record command evidence; an `exit_code` requires a
  command.
- `source_ids` records local source IDs, issue IDs, fixture IDs, or sanitized
  IDs from an external evidence ledger. It does not authorize `codex-dev` to
  fetch, ingest, or persist raw provider output.
- `actor` and `tool` record who or what produced the evidence.
- `confidence` is an integer from `0` to `100`.
- `residual_risk` records known risks.
- `artifacts` records local paths or stable artifact identifiers.

Record validity rules:

- `schema` must be `codex-dev.evidence.v1`.
- Text fields and repeated values must be non-empty and must not contain control
  characters.
- `exit_code` requires `command`.
- `confidence` must be between `0` and `100`.

Provider response dumps, secrets, private repository content, ignored overlay
manifests, and raw local workstation paths must not be written to any tracked
artifact, including docs and task-capsule evidence files. Capsules may include
local paths only when they remain local and untracked.

`codex-dev evidence append` is the CLI owner for adding validated evidence
records without hand-editing this JSONL file. Follow-on write commands reject
symlinked JSON/JSONL contract files before validation or writing. The appender
updates `capsule.json.updated_at` monotonically: backfilled evidence never
moves the capsule timestamp backwards. `capsule status` and `capsule render`
summarize total evidence count, count by kind, and latest evidence by kind.

### verification.json

`verification.json` records the current gate snapshot:

```json
{
  "schema": "codex-dev.verification.v1",
  "required": [
    {
      "name": "docs-links",
      "command": "python3 tools/docs/check_links.py docs README.md AGENTS.md",
      "status": "passed"
    }
  ],
  "optional": [],
  "last_checked_at": "2026-05-09T03:30:00Z"
}
```

The policy gate must reference repo-native commands. It may plan, execute, and
record gates, but it is not a second source of truth for what the repo requires.

`codex-dev.policy-gates.v1` is the machine-readable manifest for policy-gate
planning. Each gate includes:

- `id`
- `name`
- `command`
- `source`
- `working_directory`
- `required_tools`
- `required`
- `network`
- `secrets`
- `failure_interpretation`

The default `codex_dev` profile owns the executable operating-layer gate
manifest: formatting, `codex-dev-core` and `codex-dev` Rust gates, CLI help,
manifest and PR-plan smokes, docs links, and whitespace checks. Additional
profiles are `codex_dev_tui`, `codex_research`, `skills`, `bootstrap_install`,
`docs`, `release`, and `full_local`. Profiles are branch-selection helpers for
agents; they do not replace the validation runbook as the human source of truth.
Dry-run policy checks record `planned` gate status in `verification.json`;
executed gates record `passed`, `failed`, or `skipped`.

Executed gates must run from a discovered or explicit repository root so
repo-relative commands produce stable results whether invoked from the root, a
subdirectory, or an installed binary with `--repo-root`.
Execution rejects ambiguous capsule/current-directory repo mismatches unless
`--repo-root` is explicit. Gate `working_directory` values are repo-relative and
must not escape the selected root. Gates marked `network` or `secrets` require
explicit execution opt-in flags before they run.

### subagents.json

`subagents.json` records delegation evidence. `codex-dev` owns the evidence
shape only. `subspawn` remains the authority for spawn policy, prompt contracts,
wait behavior, and synthesis rules. `codex-dev` derives mechanical batch status
from recorded rows for scanability, but it does not spawn, wait on, retry, or
semantically interpret agent output.

```json
{
  "schema": "codex-dev.subagents.v1",
  "batches": [
    {
      "id": "pre-pr-review",
      "status": "completed",
      "task": "pre-PR branch review",
      "mode": "read-only",
      "scope": "current branch diff",
      "wait_policy": "strict",
      "rendezvous_required": true,
      "duplicate_roles_ignored": {
        "test_runner": [
          "skills/subagent-creator/templates/agents/test_runner.toml",
          "skills/subspawn/templates/agents/test_runner.toml"
        ]
      },
      "prompts": [
        {
          "role": "docs_aligner",
          "prompt_id": "pre-pr-review:docs_aligner",
          "prompt_hash": "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        }
      ],
      "agents": [
        {
          "role": "docs_aligner",
          "task": "pre-PR docs alignment review",
          "status": "completed",
          "summary": "one required consistency fix found",
          "disposition": "accepted",
          "human_verified": true,
          "source_ids": ["docs_aligner:1"],
          "artifacts": ["review-notes.md"]
        }
      ],
      "synthesis": {
        "status": "completed",
        "summary": "accepted one docs fix and reran links",
        "human_verified": true,
        "source_ids": ["synthesis:pre-pr-review"],
        "artifacts": ["review-summary.md"],
        "updated_at": "2026-05-09T06:20:00Z"
      }
    }
  ]
}
```

`codex-dev subagents record-plan` records `subspawn_plan.py --json` output
without storing raw prompt text. It preserves duplicate-role warnings and
registry issues, then records stable prompt IDs and deterministic prompt
SHA-256 hashes. `record-outcome` and `record-synthesis` require human
verification, source IDs, and artifact references, append `subagent` evidence to
`evidence.jsonl`, and keep `capsule.json` updated monotonically. Completed
synthesis is accepted only after every planned role has a terminal
human-verified outcome and a final disposition. These commands do not execute or
wait on agents.

### pr.json

`pr.json` records hosted PR and review evidence. `codex-dev` owns the evidence
shape only. Today, `gh-pr-review-fix`, `review-remediation`, the GitHub app,
`gh`, and review-pack tooling remain the live remediation and thread-closure
authorities. Future PR-agent write lanes may orchestrate hosted actions only
when they preserve the explicit-target, dry-run, `--apply`, idempotency, and
verify-before-fix rules in the
[codex-dev PR-Agent Safety Model](codex-dev-pr-agent-safety-model.md).

```json
{
  "schema": "codex-dev.pr.v1",
  "repository": "BjornMelin/dev-skills",
  "number": 29,
  "url": "https://github.com/BjornMelin/dev-skills/pull/29",
  "state": "open",
  "mergeable": "mergeable",
  "review_decision": "approved",
  "head_sha": "abc123",
  "head_ref_name": "feat/codex-dev-pr-agent",
  "base_ref_name": "main",
  "base_ref_oid": "base123",
  "checks": [
    {
      "name": "GitGuardian Security Checks",
      "status": "completed",
      "conclusion": "success",
      "url": "https://github.com/BjornMelin/dev-skills/pull/29/checks",
      "checked_at": "2026-05-09T03:45:00Z"
    }
  ],
  "review_threads": {
    "unresolved": 0,
    "total": 3,
    "resolved": 3,
    "outdated": 0,
    "authoritative": true,
    "last_checked_at": "2026-05-09T03:45:00Z"
  },
  "sources": [
    {
      "kind": "gh-pr-view",
      "parser_version": "codex-dev.pr-source-parser.v1",
      "retrieved_at": "2026-05-09T03:45:00Z",
      "command": "gh pr view 29 --json number,url,state,isDraft,mergeable,mergeStateStatus,reviewDecision,statusCheckRollup,headRefOid,headRefName,baseRefName,baseRefOid,updatedAt,labels",
      "path": "/tmp/gh-pr-view.json"
    }
  ]
}
```

Each present `checks` entry must use the typed fields shown above.
`review_threads.unresolved` means current unresolved threads only when
`review_threads.authoritative` is true. Sources that do not carry hosted thread
state, such as `gh pr checks`, must leave that flag false when no earlier
authoritative thread source exists. Outdated threads remain visible through
`review_threads.outdated` instead of being folded into the current unresolved
count. `sources[]` records deterministic parser provenance for saved
normalized, `gh`, GitHub API, and review-pack artifacts. Later PR-control work
may add more typed PR evidence, but it should not replace these fields with raw
provider JSON.

### policy.json

`policy.json` records the capsule's local policy-gate manifest snapshot. The
schema identifier is `codex-dev.policy-gates.v1`; the default profile is
`codex_dev`. Policy execution may append verification and evidence records, but
the manifest contract remains a typed JSON file instead of a rendered Markdown
section.

`codex-dev.pr-control-plan.v1` records the live-command plan for PR evidence
capture. `codex-dev` constructs the executable plan at the CLI boundary; the
typed plan data model remains a `codex-dev-core` contract. Plans may reference
network- and auth-dependent `gh`, `review-pack`, and `gh-pr-review-fix`
commands, but those tools remain the live source of truth for hosted review and
CI state. Commands that need a caller-supplied artifact expose that requirement
with `manual_input` and are not marked directly required. `codex-dev pr record`
accepts local normalized snapshots plus saved `gh pr view`, `gh pr checks`,
REST review, REST review-comment, GraphQL review-thread, and `review-pack
remaining` outputs. Every non-`normalized` source must preserve explicit PR
identity through a GitHub PR URL or caller-provided `--repo` and `--number`.
Provider-derived partial sources merge into the existing `pr.json` snapshot
instead of replacing unrelated fields, and they must not silently turn unknown
thread state into a clean authoritative thread count. GraphQL review-thread
captures are authoritative only when `pageInfo.hasNextPage` is false, while REST
review comments contribute thread-root counts but never current unresolved state.
It writes only the typed `pr.json` evidence contract plus an `evidence.jsonl`
summary. It updates
`capsule.json.updated_at` monotonically, matching the evidence appender
freshness rule.

`codex-dev.pr-agent-state.v1` records the dry-run state engine output. The CLI
executes read-only `gh` sources, writes captured provider JSON under
`pr-agent-sources/<timestamp>/`, records PR-state-bearing sources through the
existing `pr record` normalizers, and writes `pr-agent-state.json` with explicit
repository, PR number, timestamp, source records, diagnostics, and recommended
next actions. Diagnostics must surface command failures, malformed JSON, missing
permissions, non-final pagination, and GitHub rate-limit state instead of
silently treating stale or partial evidence as clean. Diagnostic-only sources
such as `gh-rate-limit`, failed commands, malformed JSON, and incomplete
pagination are stored raw or as failure artifacts under
`pr-agent-sources/<timestamp>/` and referenced from diagnostics rather than
converted into `pr record` source kinds. This command has no hosted-write mode;
future hosted-action and TUI lanes should consume the typed report instead of
rediscovering PR state ad hoc.

`codex-dev.pr-agent-hosted-action.v1` records the first apply-gated hosted PR
action contract. The command captures fresh PR state before every plan, writes
`pr-agent-actions/<plan-id>/before-state.json`, writes
`pr-agent-actions/<plan-id>/plan.json`, and requires explicit repository, PR
number, plan ID, action kind, and `--apply` before any hosted mutation runs.
Replay sources are allowed only for dry-run planning; `--source-dir` is rejected
with `--apply` so write execution is always based on live state capture. Apply
mode fails closed when required live source capture or normalization emits error
diagnostics. Comment and review-reply actions include a stable hidden
idempotency marker derived from the plan hash and run a duplicate check before
posting. Thread and label actions check current PR state and skip if the target
is already in the desired state. Failed-job reruns fetch the workflow run,
require same-repository head identity, allowed workflow events, matching PR head
branch/SHA, and a bound workflow-run URL, then skip non-failed or still-running
runs. Applied or duplicate skipped actions append evidence and capture
after-state when possible.

The hosted action layer may plan or apply issue comments, review-comment
replies, review-thread resolve/unresolve mutations, label edits, and rerun
failed workflow-job requests. Permission diagnostics are advisory: the report
records token-environment posture and expected GitHub permission classes, while
GitHub remains authoritative for actual write authorization. Failed hosted
commands must be recorded with command, exit code, redacted stderr excerpt, and
residual risk rather than being treated as successful review cleanup.

`codex-dev.pr-agent-readiness.v1` records the bounded PR closeout loop. The CLI
reuses the `pr agent` state engine for every poll attempt, writes
`pr-readiness.json` and `pr-readiness.md`, and classifies the final state as
`ready`, `waiting`, `blocked`, `merged`, or `stopped`. Readiness evaluates
checks, allowlisted check conclusions, GitHub Actions run IDs from
same-repository check URLs, authoritative review-thread state, stale review
comments, draft state, mergeability, `mergeStateStatus`, head SHA, and branch
refs. It deliberately keeps hosted review-thread resolution separate
from local code fixes and from stale `reviewDecision` values.

Non-ready final states are gate failures: with `--json`, `pr readiness` still
writes the readiness artifacts and output envelope, but exits nonzero unless
the final status is `ready` or `merged` and all requested hosted actions avoided
failure.

`pr readiness` has bounded polling only; it is not a daemon. Dry-run mode may
use replay sources. Apply mode rejects replay sources and only executes hosted
mutations when the caller also requests the specific intent: `--rerun-failed`
for failed-job reruns or `--merge` for merging. Failed-job reruns delegate to
the apply-gated hosted-action contract so workflow-run repository, event, PR
binding, head branch, and head SHA are rechecked. Merge execution requires a
ready final state, explicit `--apply --merge`, an immediate fresh readiness
refresh, and `gh pr merge --match-head-commit <fresh-head-sha>`.

### Markdown Notes

`plan.md`, `decisions.md`, `output.md`, and `retrospective.md` are durable
human notes inside the local capsule. They should render summaries that can be
copied into issues or PRs, but automation must read contract JSON files instead
of scraping Markdown.

## Skill And Subagent Eval Lab

`tools/eval/skill_subagent_eval.py` owns offline development eval orchestration
for skill metadata and subagent contracts. It records a normalized
`dev-skills.skill-subagent-eval.v1` report with bounded per-check timeouts and
isolated Python bytecode caches while delegating the actual checks to existing
owners:

- `tools/skill/quick_validate.py`
- `skills/subagent-creator/scripts/subagent_creator.py validate`
- `skills/subspawn/scripts/subspawn_plan.py validate-roles`
- `skills/subspawn/scripts/subspawn_plan.py plan`
- `python3 -m compileall`

The eval lab is not a research evaluator. `codex-research eval` remains the
owner for research routing, privacy, budgets, evidence, report, and closeout
bundle contracts.

## Branch And PR Graph

The first operating-layer release was split into issue-backed lanes:

| Issue | Branch | Depends on | Unblocks | Schema owner | Purpose |
| --- | --- | --- | --- | --- | --- |
| #21 | `docs/codex-dev-operating-layer-contract` | #20 | #22, #23, #24, #25, #26, #27, #28 | `docs/specs/codex-dev-operating-layer.md` | Define architecture, capsule schema, and ownership. |
| #22 | `feat/codex-dev-task-capsules` | #21 | #23, #24, #25, #28 | `crates/codex-dev` | Add CLI core for capsule lifecycle. |
| #23 | `feat/codex-dev-policy-gate` | #21, #22 | #24, #25, #26 | `crates/codex-dev` | Add thin policy-gate orchestration. |
| #24 | `feat/skill-subagent-eval-lab` | #21, #22 | #26 | eval fixtures/scripts | Add offline skill and subagent eval coverage. |
| #25 | `feat/codex-dev-pr-control` | #21, #22, #23 | final release closeout | `crates/codex-dev` | Add PR state and review evidence adapters. |
| #26 | `feat/repo-bootstrap-packs` | #21, #23, #24 | #27 | bootstrap templates/scripts | Add repo bootstrap packs and install smoke matrix. |
| #27 | `docs/memory-guidance-proposals` | #21, #26 | final release closeout | `docs/cookbooks/` surface | Add memory proposal guidance. |
| #28 | `feat/codex-dev-tui-workbench` | #21, #22 | optional release polish | `codex-dev-core` JSON contracts | Add optional Ratatui workbench after JSON contracts stabilize. |

## Optional TUI Consumer

`codex-dev-tui` is a separate crate that renders a local operator view from the
existing `codex-dev-core` JSON contracts:

- capsule summary from `capsule.json`;
- validation snapshot from `verification.json`;
- hosted PR snapshot from `pr.json`;
- validation errors from `codex_dev_core::validate_capsule`.

The TUI owns only UI state, event handling, rendering, and terminal cleanup. It
does not execute policy gates, mutate PR state, call hosted review tools, or
scrape Markdown notes. Deterministic `--render-once` output uses Ratatui's
`TestBackend` for review and CI smoke evidence.

Each implementation PR must link its lane issue and #20, include validation
evidence, document docs impact, and identify residual risks.

After this roadmap PR merges, implementation PRs for the remaining child issues
from #39 through #57 must follow the dependency order and branch ledger in the
[dev-skills v0.3/v1 Roadmap](dev-skills-v0.3-roadmap.md). Implement one issue
per branch and PR, merge into `main`, sync local `main`, and then start the next
issue.

## Hardened Subagent Pack Boundary

`subagents/hardened-codex` is a tracked source pack. Public artifacts include
global roles, public overlays, renderer, sync helper, examples, and generated
catalog. Local manifests and private overlays are ignored by design.

Rules:

- Run dry-run/list/validation commands before install writes.
- Treat prior smoke results as historical context; refresh live validation after
  install work.
- Do not publish ignored `roles.local.json`, `overlays.local.json`, private
  overlay TOMLs, or raw local target paths.
- When summarizing workstation state, distinguish tracked public roles from
  ignored local overlays.

Baseline checks:

```bash
PYTHONDONTWRITEBYTECODE=1 python3 skills/subagent-creator/scripts/subagent_creator.py validate subagents/hardened-codex/agents
PYTHONDONTWRITEBYTECODE=1 python3 subagents/hardened-codex/scripts/sync_agents.py --global --all-overlays --dry-run
PYTHONDONTWRITEBYTECODE=1 python3 skills/subspawn/scripts/subspawn_plan.py validate-roles
```

## Validation Expectations

Docs-only operating-layer changes must run:

```bash
python3 tools/docs/check_links.py docs README.md AGENTS.md
git diff --check
```

Rust implementation lanes must use the canonical command matrix in
[Validation](../runbooks/validation.md). Later lanes should extend that runbook
only when they add a new canonical surface.
