---
name: gh-pr-review-fix
description: "Fix unresolved GitHub PR review threads end-to-end with codex-dev pr review: fetch fresh hosted state, verify still-valid findings, implement minimal fixes, validate, create scoped semantic Conventional Commits, push, and resolve the fixed hosted threads. Do not use for local Codex, Zen, or manual review files; review-remediation owns those."
---

# GitHub PR Review Fix

Use this as the sole active GitHub PR review remediation workflow. The canonical implementation is `codex-dev`; this skill is the operating procedure.

## Default Behavior

When the user invokes `$gh-pr-review-fix`, run the full closeout loop unless blocked:

1. Read repo `AGENTS.md` and inspect `git status --short`.
2. Capture fresh hosted review work:
   - `codex-dev --json pr review start --repo <owner/repo> --number <pr> --fresh`
3. If the worklist reports zero actionable unresolved items, stop with a no-op summary.
4. Verify each item against current code; skip stale findings with a brief reason.
5. Apply minimal fixes, using exact suggestion fences only when `codex-dev pr review apply-suggestions` reports an exact hunk match.
6. Run focused repo-native validation, then broad required gates once before publishing.
7. Create scoped, semantically grouped, reviewable Conventional Commits following `references/closeout-and-commits.md`.
8. Push the PR branch once after all intended semantic commits pass validation.
9. Re-capture fresh PR head/thread state and resolve every matching fixed hosted thread:
   - `codex-dev --json pr review closeout --repo <owner/repo> --number <pr> --worklist <json> --expected-head-sha <pushed-head> --commit <sha> --validation-command <cmd> --apply`
10. Re-run `codex-dev --json pr review start --fresh --repo <owner/repo> --number <pr>` and continue until zero actionable unresolved items remain or a real blocker appears.

## Hosted Closeout Policy

- Default is fix, commit, push, and resolve fixed hosted threads.
- Do not auto-reply by default. Reply only when a finding cannot be fixed/resolved cleanly or the user asks for comments.
- Resolve only current hosted thread IDs that map to verified fixes or already-fixed current head state.
- Never resolve when the PR head changed unexpectedly, validation failed, the finding was skipped, or the thread cannot be matched to closeout evidence.

## Commit Policy

- Read `references/closeout-and-commits.md` before committing.
- Use `codex-dev --json commit plan` to inspect semantic groups in a mixed tree.
- Use `codex-dev --json commit validate --subject "<subject>"` before committing.
- `codex-dev commit validate` owns forbidden process-wording checks; do not maintain a second local phrase list here.

## Route Away

- Local review file, Codex review, Zen review, or manual notes -> `$review-remediation`.
- Passive post-push monitoring -> `codex-dev --json pr readiness` plus GitHub status checks.
- CI-only failure with no review-thread context -> GitHub plugin or `gh` workflow-log remediation.

## Outputs

- `codex-dev.pr-review-worklist.v1` worklist
- verified fix/skipped-item summary
- validation commands and outcomes
- semantic commit SHAs
- `codex-dev.pr-review-closeout.v1` thread-to-commit closeout evidence
- terminal status: `completed`, `blocked`, or `no-op`

## Resources

- `references/closeout-and-commits.md`
