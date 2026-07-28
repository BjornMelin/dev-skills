# Goal to Release Templates

Load this when drafting issues, PR bodies, release summaries, follow-ups,
handoffs, or closeout summaries.

## Issue Body

```markdown
## Problem

<What is broken, missing, risky, or strategically required. Include live
evidence links or paths when available.>

## Decision

<Chosen direction, or what research must settle.>

## Scope

In:
- 

Out:
- 

## Implementation Plan

1. 
2. 
3. 

## Acceptance Checks

- [ ] Code or docs implement the agreed behavior.
- [ ] Relevant local and hosted checks pass.
- [ ] PR links this issue.
- [ ] Docs/runbooks/specs are updated when behavior or operations change.
- [ ] Deploy or browser proof is recorded when relevant.

## Expected Branch / PR

- Branch:
- PR target:
- Stack position:

## Follow-ups

- None expected.
```

## PR Body

```markdown
## Summary

- 
- 

## Linked Issues

Closes #

## Scope Notes

- Base/target:
- Stack position:
- Intentional hard cuts:
- Out of scope:

## Validation

- [ ] `<command>` -> `<outcome>`

## Hosted Review / CI

- CI:
- Review decision:
- Review threads:
- Mergeability:

## Deploy / Operations

- Dev:
- Prod:
- Monitoring:
- Not required because:

## Docs / Release Notes

- Docs updated:
- Release note impact:

## Follow-ups

- None.
```

## Stacked PR Contract

```markdown
Stack:
- Base branch:
- Depends on:
- Merge order:
- Retarget after merge:
- After base merge:
  - retarget/rebase onto:
  - rerun checks:
  - refresh review state:

Validation scope:
- This PR proves:
- Later PR proves:

Conflict risk:
- 
```

## Review Reply

Use for stale or intentionally rejected review comments.

```markdown
@coderabbitai Verified against current head `<sha>`.

Evidence:
- `<path>` now ...
- `<command>` -> passed

Decision:
- No code change is needed because ...
```

Omit `@coderabbitai` for non-CodeRabbit reviewers unless appropriate.

## Release Summary

```markdown
## Release Summary

This release includes:
- 

## Included PRs

- # - 

## User-Facing Changes

- 

## Operational Changes

- 

## Validation and Deploy Evidence

- 

## Known Follow-ups

- None.
```

## Follow-up Issue

```markdown
## Context

<What shipped, and why this remaining work is separate.>

## Follow-up Scope

- 

## Not in Scope

- Reopening shipped behavior without new evidence.
- Compatibility paths for abandoned drafts.

## Acceptance Checks

- [ ] 
```

## Final Closeout

```markdown
What shipped:
- 

Issues and PRs:
- 

Validation:
- 

Deploy:
- 

Docs/release:
- 

Distilled summary:
- summary.md:
- summary.json:

Follow-up:
- None.
```

## Session Handoff

```markdown
Current state:
- 

Merged / open PRs:
- 

Issues:
- 

Validation and deploy evidence:
- 

Next exact action:
- 

Do not redo:
- 

Blockers / risks:
- 
```
