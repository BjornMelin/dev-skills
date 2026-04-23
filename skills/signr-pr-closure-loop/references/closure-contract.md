# Closure Contract

Treat the PR as ready to hand back for merge only when all of these are true on
the current head SHA:

- required CI checks are green
- the PR is mergeable with no unresolved branch-state blocker
- unresolved hosted review threads and actionable review comments are zero
- no validated human or CodeRabbit blocker remains
- impacted docs are aligned with the final code shape
- final local verification for the shipped shape has been run

## Hosted truth

- Prefer `review-pack remaining` or an explicit hosted unresolved-thread query as
  the source of truth for review closure.
- Treat `reviewDecision` as advisory only. It can lag after fixes or after
  hosted threads have already been resolved.
- Treat explicit CodeRabbit approval as useful evidence, not the sole terminal
  condition. If CodeRabbit status is green and no actionable CodeRabbit comment
  remains, do not wait forever for a separate approval transition.

## Refresh cycle

After every push or hosted resolution:

1. refresh PR metadata
2. refresh checks on the new head SHA
3. refresh unresolved hosted review state
4. re-enter the blocker-routing loop if anything new appeared

## Escalate To The User

Stop and ask for help only when the run is blocked by something the agent cannot
resolve safely, such as:

- unrelated dirty worktree changes
- missing permissions
- repeated infrastructure or flake failures after reasonable retries
- reviewer requests that conflict materially
- destructive or irreversible actions not already authorized
