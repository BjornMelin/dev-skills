---
name: review-remediation
description: Fix local review (Codex, Zen, notes) w/ verify. Not gh-pr-review-fix or passive PR monitoring.
---

# Review Remediation

Use this skill to normalize review inputs, turn them into an actionable checklist, and then fix the valid findings with repo-native verification.

## Modes

- `local-review`: ingest a local review note file
- `codex-review`: ingest local Codex review output
- `zen-review`: ingest local Zen review output

## Workflow

1. Read the repo `AGENTS.md`.
2. Normalize the review source:
   - Local file -> `codex-dev --json review ingest --source <file> --kind codex|zen|manual --out <json>`
3. Render a concise summary:
   - `codex-dev --json review render --worklist <json>`
4. Build the remediation order:
   - correctness and safety first
   - review threads grouped by file or subsystem
   - minimal change that fully resolves the finding
5. Verify with repo-native checks before considering a finding done.
6. If the workflow becomes passive PR monitoring, switch to `codex-dev --json pr readiness` plus GitHub status checks.

## Use When

- The user asks to address local review findings end-to-end.
- Review data exists in files but needs normalization before implementation.

## Do Not Use When

- The user wants unresolved GitHub PR review threads fetched and fixed.
- The user wants a passive or continuous PR watcher.
- The task is only a docs drift pass or only a dependency plan.

## Outputs

- normalized review bundle
- short prioritized remediation summary
- verified fixes with concise reporting

## Resources

- `references/local-review.md`
- `references/verification.md`
