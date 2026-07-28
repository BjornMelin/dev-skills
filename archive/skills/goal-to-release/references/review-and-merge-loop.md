# Review and Merge Loop

Use this after opening a PR, after each push, when hosted reviews are stale, or
before merging.

## Publication Mechanics

Before opening:
- push the branch to the remote target
- choose the base from the PR graph, not habit
- link the issue in the PR body
- include validation, docs/deploy impact, hosted-review expectations, and
  residual risks

For a stacked PR, use the immediate dependency branch as the base. For a serial
PR, use the release target branch.

## Fresh State

After each push, refresh:
- PR head SHA
- required checks
- mergeability
- latest review decision
- unresolved review threads
- bot comments generated after the current head

Treat unresolved review-thread state as closure truth. Review decisions can lag
after threads are resolved.

For stacked PRs:
1. Verify each PR's base branch matches the recorded stack contract.
2. Merge the root/base PR first.
3. Retarget or rebase the next PR onto the real merge target after the base
   lands.
4. Push the updated head, then rerun the fresh-state loop before merge.
5. Repeat one stack layer at a time.

## Comment Policy

For every review comment:
1. Verify against current code and current head.
2. If valid, fix with focused tests.
3. If stale, invalid, or worse than the implemented design, reply with evidence.
4. Resolve only after the finding is addressed or proven stale.

For CodeRabbit replies, start with `@coderabbitai` when disagreeing or
declining a suggestion.

## Waiting Policy

When CI or review is pending:
- use time for read-only work: issue audit, docs review, release notes,
  preflight for the next lane, or follow-up triage
- do not start dependent implementation unless the PR graph explicitly allows a
  stack or parallel branch
- update the ledger with current hosted state

If checks or review are stuck with no new signal after bounded refreshes, record
the last poll time, URLs, and blocker owner in the ledger. Continue only on
safe, independent read-only work; otherwise pause the lane as externally
blocked instead of inventing changes.

## Merge Readiness

Merge only when:
- required CI is green
- hosted review state is approving or clean
- unresolved review threads are zero or explicitly accepted by the repo process
- PR branch is current enough for the repo policy
- issue and PR bodies match shipped behavior
- validation and deploy evidence are recorded
- deploy evidence is either recorded or explicitly marked not applicable with a
  reason

After merge:
- verify the PR landed in the intended target
- close or update linked issues
- update the ledger before starting the next lane
