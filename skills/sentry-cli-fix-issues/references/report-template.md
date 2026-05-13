# Report Template

Use this shape for final closeout after fixing a Sentry issue.

## Summary

- Sentry issue: SHORT_ID or permalink
- Impact: affected users/events/window
- Root cause: one sentence
- Fix: one sentence

## Evidence

| Command | Result |
| --- | --- |
| `sentry issue view ...` | status, project, release, top frame |
| `sentry issue events ...` | representative event evidence |
| `sentry trace view ...` | trace/span evidence, or `not available` |
| `sentry issue explain/plan ...` | advisory finding, or `not used` |

Mark any incomplete claim as `UNVERIFIED`.

## Files Changed

- `path/to/file`: why it changed
- `path/to/test`: coverage added or updated

## Verification

- `command`: outcome
- `command`: outcome

## Sentry Follow-Up

- Fresh CLI check: `sentry issue view ISSUE --fresh ...`
- State change: resolved, archived, left open, or not requested
- Residual risk: remaining uncertainty, deployment dependency, or monitoring
  window needed
