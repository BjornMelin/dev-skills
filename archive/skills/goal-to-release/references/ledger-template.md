# Goal Ledger Template

Use this when a goal spans issues, branches, PRs, reviews, docs, deploys, or
handoffs. Keep entries concise, live-evidence backed, and updated when plans
change.

## Goal Charter

```markdown
Objective:

Non-goals:

User steering and constraints:
- 

Definition of done:
- 

Release/deploy boundary:
- 

Mode:
- plan-only | preflight | implement | monitor | closeout | retrospective
```

## Preflight

```markdown
Repo:
Branch:
Dirty worktree:
Discovered surfaces:
- 

Required gates:
- command -> reason

Docs/deploy evidence needed:
- 

Suggested subagent reviews:
- 

Blockers / warnings:
- 
```

## Issue Plan

```markdown
| Issue | Lane | Branch | PR | Scope | Acceptance checks | Docs/deploy impact | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| #123 | Runtime durability | feat/runtime-ledger | #130 | Durable events and replay | Tests, check, deploy proof | API spec/runbook | In progress |
```

Issue body checklist:
- problem and current evidence
- target behavior or decision
- in-scope and out-of-scope items
- impacted surfaces
- acceptance checks
- expected branch/PR linkage
- docs, release, and deploy requirements

## Branch and PR Graph

```markdown
Base:

Graph:
1. PR #A -> base <branch>: <purpose>
2. PR #B -> base <PR #A branch>: <purpose>
3. PR #C -> base <branch>: independent lane

Stack contract:
- Base branch:
- Dependency:
- Retarget plan:
- Validation scope:
- Merge order:

Merge plan:
- 
```

Use serial PRs for coupled work. Use stacked or parallel PRs only when the
dependency graph is explicit and reviewable.

## Lane Implementation Loop

```markdown
Lane:
Issue:
Branch:
PR:

Evidence gathered:
- 

Decision:
- 

Files changed:
- 

Validation:
- command -> outcome

Hosted checks/review:
- check -> outcome
- review decision -> outcome
- unresolved threads -> count

Deploy/monitoring:
- environment -> outcome
- applicability -> required/not applicable because ...

Residual risk:
- 
```

## Closeout Audit

```markdown
Issues:
- [ ] All implemented issues link to PRs.
- [ ] Closed issues match shipped behavior.
- [ ] Non-shipped items are dropped or moved to follow-up issues.

PRs:
- [ ] All PRs merged into intended targets.
- [ ] Stacked PRs retargeted or closed.
- [ ] Review threads resolved from live hosted state.

Docs:
- [ ] README/current-state docs updated.
- [ ] ADR/SPEC/requirements updated when contracts changed.
- [ ] Runbooks/deploy docs updated when operations changed.

Release/deploy:
- [ ] Required dev deploy completed or explicitly deferred.
- [ ] Required prod deploy completed or explicitly deferred.
- [ ] Monitoring/diagnostics checked.

Durable summary:
- [ ] summary.md exists for achieved durable goals.
- [ ] summary.json exists for achieved durable goals.
```

## Retrospective

```markdown
What shipped:
- 

Major decisions:
- 

User steering that changed the outcome:
- 

Hard cuts / entropy reductions:
- 

Verification and deploy evidence:
- 

Docs and issue hygiene:
- 

What to do differently next time:
- 

Reusable workflow update:
- 
```

## Distilled Summary

When the goal is achieved, create `summary.md` and `summary.json` in the goal
directory with:
- final status and objective
- issues, PRs, branches, and merge outcomes
- major decisions
- validation and deploy evidence
- docs/release impact
- residual risks and follow-ups
- archive pointers for raw evidence
