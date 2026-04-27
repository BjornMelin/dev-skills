---
description: Reporting template for repo modernization runs. Ensures findings, plan, execution, verification, and residual risk are presented in a consistent, decision-ready structure.
---

# Report Template

Use this structure for final delivery after a repo modernization run.

The report must separate:
- findings
- plan
- execution
- verification
- residual risk

Do not blur “code changed” with “migration complete” or “validation complete”.

## 1. Findings

Start with the important findings, not a narrative introduction.

Cover:
- vulnerable dependencies
- outdated dependencies
- deprecated APIs
- abandoned or weakly maintained packages
- custom code that can be replaced by dependency-native functionality
- compatibility layers, adapters, fallbacks, shims, and duplicate codepaths

Recommended structure:

- `critical`
- `high`
- `medium`
- `low`

For each finding, include:
- package or file/scope
- problem
- impact
- recommended action

## 2. Execution Plan

State the plan that was used or finalized.

Cover:
- upgrade waves
- package groups
- framework lanes involved
- touched-file clusters
- expected deletions
- major migration/risk points

Keep it concrete. Avoid vague plan language.

## 3. Changes Applied

Summarize what was actually changed.

Cover:
- dependencies upgraded
- dependencies replaced
- dependencies removed
- framework migrations applied
- codemods run
- deprecated APIs removed
- custom code deleted because dependencies now provide native support
- compatibility code deleted under hard-cut policy

If relevant, split by workspace/package/app.

## 4. Files Touched

List the principal files or directories changed.

Do not dump every generated file unless it matters.
Group by concern where useful:
- dependency manifests / lockfiles
- config
- framework migration files
- runtime code
- tests
- docs

## 5. Verification

This section must be explicit.

Cover:
- exact commands run
- which passed
- which failed
- what was fixed during verification
- post-change audit/outdated results

Minimum expected items:
- install/sync
- audit
- outdated
- lint
- typecheck
- tests
- build
- repo-native validation scripts

If any step was skipped:
- say so explicitly
- say why
- say what risk remains because it was skipped

## 6. Outcome Summary

State the modernization result in concise terms.

Include:
- vulnerabilities fixed count
- outdated dependencies resolved count or summary
- dependencies removed/replaced summary
- deprecations removed summary
- net LOC added vs deleted
- whether the final codebase is smaller/simpler

## 7. Residual Risks and Blockers

Separate incomplete work from completed work.

For each blocker or residual risk, include:
- exact package/file/scope
- reason it remains
- whether it is upstream, environmental, or contract-boundary driven
- what would be required to finish it

Do not hide blockers inside the change summary.

## 8. Recommended Next Steps

Only include if there are real next steps.

Examples:
- finish an upstream-blocked migration once package X releases fix Y
- remove a temporary compatibility boundary after system Z is migrated
- perform follow-up runtime validation in a deployment environment

Keep this list short and concrete.

## Compact Skeleton

Use this skeleton if you need a fast structure:

```text
Findings
- ...

Execution Plan
- ...

Changes Applied
- ...

Files Touched
- ...

Verification
- command: result

Outcome Summary
- ...

Residual Risks and Blockers
- ...

Recommended Next Steps
- ...
```

## Rules

- findings first
- no padded intro
- no “done” language if verification is partial
- no mixing blocked work with completed work
- no lockfile-only summary; always explain code and architecture impact
