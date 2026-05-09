# codex-dev Operating Layer

Status: active implementation.

Tracking: #20, #21, #22, #23, #24, and #25.

## Purpose

`codex-dev` is the development control-plane family for this repo. The CLI
delivered by issue #22 records agent work as local task capsules. Issue #23
adds a repo-native policy gate that can plan or execute local validation while
recording the result in the capsule. Issue #24 adds offline skill and subagent
eval coverage, and issue #25 adds PR evidence planning plus local normalized
snapshot recording. Later lanes add bootstrap composition and stable JSON
contracts for optional consumers such as a terminal workbench.

`codex-dev` is deliberately separate from `codex-research`. The research CLI
continues to own provider routing, source hydration, research ledgers, cache,
and research evals. The new operating layer coordinates development work around
those tools instead of becoming another research provider.

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
- Do not reimplement hosted review remediation outside `gh-pr-review-fix` and
  `review-remediation`.
- Do not make the optional TUI own business logic.
- Do not support compatibility shims for pre-1.0 draft capsule shapes.

## Ownership Map

| Surface | Canonical owner | `codex-dev` relationship |
| --- | --- | --- |
| Research provider routing, source cache, claim ledgers, research evals | `codex-research` | Calls or records output as external evidence. |
| Task capsules, policy-gate orchestration, PR/eval/bootstrap evidence appenders | `codex-dev` | Primary owner. |
| Skill metadata and package validation | `tools/skill`, skill folders | Runs existing validators and records results. |
| Custom subagent template validation and installs | `subagent-creator` | Reuses validation/install commands. |
| Subagent fanout planning and wait policy | `subspawn` | Records selected plan and subagent outcomes. |
| Hosted PR review remediation | `gh-pr-review-fix`, `review-remediation` | Captures review-pack/CI snapshots and links fixes. |
| Hardened personal subagent pack | `subagents/hardened-codex` | Treats as a bootstrap input and smoke target. |
| Memory and Codex runtime guidance | `codex-sdk` docs/skill | Links proposal docs; does not mutate runtime memory. |
| Optional terminal UI | future `codex-dev-tui` | Reads `codex-dev --json`; never owns policy. |

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
retrospective.md
```

Contract files are `capsule.json`, `evidence.jsonl`, `verification.json`,
`subagents.json`, and `pr.json`. Markdown files are human notes whose headings
are conventional but not machine contracts.

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

Provider response dumps, secrets, private repository content, ignored overlay
manifests, and raw local workstation paths must not be written to any tracked
artifact, including docs and task-capsule evidence files. Capsules may include
local paths only when they remain local and untracked.

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
- `required`
- `network`
- `secrets`

The default `codex_dev` profile references the canonical `codex-dev` validation
section in `docs/runbooks/validation.md`, marks every gate as local, and sets
`network: false` and `secrets: false`. Dry-run policy checks record `planned`
gate status in `verification.json`; executed gates record `passed`, `failed`,
or `skipped`.

Executed gates must run from a discovered or explicit repository root so
repo-relative commands produce stable results whether invoked from the root, a
subdirectory, or an installed binary with `--repo-root`.

### subagents.json

`subagents.json` records delegation evidence. `codex-dev` owns the evidence
shape only. `subspawn` remains the authority for spawn policy, prompt contracts,
wait behavior, and synthesis rules.

```json
{
  "schema": "codex-dev.subagents.v1",
  "batches": [
    {
      "id": "pre-pr-review",
      "status": "completed",
      "agents": [
        {
          "role": "docs_aligner",
          "task": "pre-PR docs alignment review",
          "status": "completed",
          "summary": "one required consistency fix found"
        }
      ]
    }
  ]
}
```

### pr.json

`pr.json` records hosted PR and review evidence. `codex-dev` owns the evidence
shape only. `gh-pr-review-fix`, `review-remediation`, the GitHub app, `gh`, and
review-pack tooling remain the authorities for live review remediation and
thread closure.

```json
{
  "schema": "codex-dev.pr.v1",
  "repository": "BjornMelin/dev-skills",
  "number": 29,
  "url": "https://github.com/BjornMelin/dev-skills/pull/29",
  "state": "open",
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
    "last_checked_at": "2026-05-09T03:45:00Z"
  }
}
```

When `checks` entries are present, each entry must use the typed fields shown
above. Later PR-control work may add more typed PR evidence, but it should not
replace this field with raw provider JSON.

`codex-dev.pr-control-plan.v1` records the live-command plan for PR evidence
capture. It may reference network- and auth-dependent `gh`, `review-pack`, and
`gh-pr-review-fix` commands, but those tools remain the live source of truth for
hosted review and CI state. `codex-dev pr record` accepts local normalized
snapshots and writes only the typed `pr.json` evidence contract plus an
`evidence.jsonl` summary.

### Markdown Notes

`plan.md`, `decisions.md`, and `retrospective.md` are durable human notes inside
the local capsule. They should render summaries that can be copied into issues
or PRs, but automation must read contract JSON files instead of scraping
Markdown.

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
owner for research routing, privacy, budgets, evidence, and report contracts.

## Branch And PR Graph

The release is split into issue-backed lanes:

| Issue | Branch | Depends on | Unblocks | Schema owner | Purpose |
| --- | --- | --- | --- | --- | --- |
| #21 | `docs/codex-dev-operating-layer-contract` | #20 | #22, #23, #24, #25, #26, #27, #28 | `docs/specs/codex-dev-operating-layer.md` | Define architecture, capsule schema, and ownership. |
| #22 | `feat/codex-dev-task-capsules` | #21 | #23, #24, #25, #28 | `crates/codex-dev` | Add CLI core for capsule lifecycle. |
| #23 | `feat/codex-dev-policy-gate` | #21, #22 | #24, #25, #26 | `crates/codex-dev` | Add thin policy-gate orchestration. |
| #24 | `feat/skill-subagent-eval-lab` | #21, #22 | #26 | eval fixtures/scripts | Add offline skill and subagent eval coverage. |
| #25 | `feat/codex-dev-pr-control` | #21, #22, #23 | final release closeout | `crates/codex-dev` | Add PR state and review evidence adapters. |
| #26 | `feat/repo-bootstrap-packs` | #21, #23, #24 | #27 | bootstrap templates/scripts | Add repo bootstrap packs and install smoke matrix. |
| #27 | `docs/memory-guidance-proposals` | #21, #26 | final release closeout | `docs/cookbooks/` surface | Add memory proposal guidance. |
| #28 | `feat/codex-dev-tui-workbench` | #21, #22 | optional release polish | `codex-dev` JSON contracts | Add optional Ratatui workbench after JSON contracts stabilize. |

Each implementation PR must link its lane issue and #20, include validation
evidence, document docs impact, and identify residual risks.

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
