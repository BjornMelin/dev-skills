# Classifier Rules

The cleanup classifier is conservative about development sessions and memory,
but aggressive about old low-value review/fix noise once safety gates pass.

## High Confidence

Select by default when older than the protection window:

- `$gh-pr-review-fix` sessions.
- `review-pack` loops.
- `Review the current code changes...` local review sessions.
- PR review comment remediation sessions.
- CodeRabbit, Sourcery, Copilot, or hosted reviewer fix loops.
- Child reviewer, triager, auditor, or CI-triage roles with review-only titles.

## Medium Confidence

Report by default and include only with `--include-medium`:

- no-user-event threads;
- very short low-token sessions;
- failed or interrupted scratch sessions;
- stale exploratory titles such as temporary probes or throwaway tests.

Medium-confidence memory-linked candidates are audit targets, not automatic
cleanup targets.

## Protected

Exclude unless a future version adds explicit force controls:

- recent sessions within `--min-age-hours`;
- all-session or Codex-home broad candidates without explicit scope;
- sessions without unambiguous ids;
- likely main development sessions;
- high-token implementation-oriented titles;
- active, paused, or budget-limited goals;
- running agent jobs and assigned job items;
- parent/child spawn edges with active or recent related threads;
- candidates whose files or DB rows cannot be backed up;
- any candidate with JSONL parse errors or SQLite integrity failures.

## Memory Triage

Memory findings are not deletion instructions. Recommendations:

- `preserve`: durable workflow/runbook/reusable knowledge.
- `audit`: stale, outdated, conflicting, or drift-prone details.
- `copy`: linked selected-thread memory without durable markers.
- `disposable`: only future versions may use this for confidently useless
  memory, and it must still quarantine first.

Current repo docs/code and live provider or GitHub state outrank historical
memory when they conflict.

## Reason Codes

Reason codes are stable strings in reports and manifests. Examples:

- `gh-pr-review-fix-title`
- `local-code-review-title`
- `hosted-review-fix-title`
- `third-party-review-title`
- `ci-triage-title`
- `reviewer-agent-role`
- `no-user-event`
- `short-low-token-session`
- `stale-exploratory-title`
- `protected-recent`
- `protected-main-dev-likely`
- `protected-active-goal-*`
- `protected-active-agent-job`
- `protected-spawn-edge-active`
- `protected-unknown-cwd-codex-home`
- `durable-memory-marker`
- `drift-or-conflict-marker`
- `stale-command-marker`
