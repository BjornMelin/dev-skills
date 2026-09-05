---
name: finalize-branch
description: "Pre-merge branch and PR closeout. Use to identify and implement follow-up work, hard cuts, dependency-native simplification, validation coverage, docs alignment, and merge-readiness fixes."
license: MIT
metadata:
  version: "1.0.0"
  companion-skills:
    - autoreview
    - repo-modernizer
    - repo-docs-align
    - qa-router
---

# Finalize Branch

Answer the practical pre-merge question: what remains before this branch is
safe, coherent, maintainable, and ready to merge? This is a decision and
execution workflow, not a generic code review or a command that automatically
ships the branch.

## Modes

- **Report-only:** inspect and return blockers, lane scores, required hard cuts,
  validation, docs work, and decisions. Do not modify the checkout.
- **Implementation:** resolve confirmed findings, remove superseded paths, add
  the narrow proof, align docs, and rerun applicable gates.
- **PR closeout:** include hosted checks, review threads, branch state, and
  mergeability when the user authorizes those read operations.

State the mode. Do not commit, push, merge, deploy, or resolve hosted threads
unless the user explicitly requests that action and the owning workflow allows
it.

## Source Order

1. Dirty worktree, branch diff, tests, generated consumers, and live PR/CI state.
2. Root and nested agent instructions, README, architecture docs, ADRs, specs,
   package scripts, validation manifests, and release runbooks.
3. Relevant companion skills and their references.
4. Current official docs and dependency source when API behavior affects a
   recommendation.

Preserve unrelated dirty changes. Never infer product status, compatibility
requirements, or validation commands from memory.

## Decision Model

Use hard gates before numeric scoring.

### Hard gates

A failure blocks merge regardless of score:

- security, authorization, tenant or account isolation, privacy, or secret
  handling;
- data integrity, destructive behavior, migration safety, idempotency, or state
  transition correctness;
- public API, schema, protocol, provider, platform, or release safety;
- a branch that does not close a coherent user/operator workflow;
- missing narrow proof for a material changed behavior;
- a required repository gate that fails;
- an unjustified compatibility layer, fallback, alias, dual shape, stale path,
  obsolete test, or dead generated consumer left behind.

### Lane scoring

Select the narrowest applicable profiles in
`references/review-framework.md`. Mixed branches receive separate lane scores
and a blocker summary; never average away a weak security, backend, native,
integration, or UX lane.

Score criteria from 0.0 to 10.0, apply the profile weights, and use these action
tiers:

- **9.0-10.0:** preferred; execute or recommend confidently.
- **8.0-8.9:** acceptable with explicit tradeoffs and verification.
- **7.0-7.9:** narrow, research, ask, or defer before implementation.
- **Below 7.0:** reject, redesign, or hard-cut more aggressively.

Scores support decisions; hard gates remain authoritative.

## Core Posture

- Keep one canonical schema, API, path, function, export, fixture, document,
  and test lane per concept.
- Delete compatibility wrappers, fallback paths, aliases, dual shapes, stale
  exports, orphaned tests, obsolete docs, unused migrations, and dead generated
  references unless a proven external boundary requires them.
- Name that external boundary, its owner, and removal condition. "Safer" is not
  proof that a compatibility path must remain.
- Prefer maintained dependency-native capabilities over local wrappers when
  they reduce ownership without weakening contracts.
- Preserve server-side authorization and isolation. Finalization must not move
  trust into clients or weaken denied-path tests.
- Keep tests deterministic and proportional to changed risk.
- Update docs only when behavior, architecture, ownership, setup, operations,
  or public contracts changed.
- Separate report-only advice from implementation. Never imply fixes were made
  or gates passed when they were only recommended.

## Companion Routing

Load only relevant skills that exist in this repository:

- `autoreview`, `codex-review`, or `multi-model-review` for independent review.
- `review-remediation` for verified local review-note fixes.
- `repo-modernizer` for dependency upgrades and dependency-native cleanup.
- `repo-docs-align` or `docs-align` for changed documentation contracts.
- `vitest-dev` or `pytest-dev` for runner-specific test work.
- `qa-router` for repository gate selection and validation routing.
- `grill-me` for an unresolved user-owned product or architecture tradeoff.
- `pre-mortem` before an irreversible, high-risk decision.
- `commit`, `ship-branch`, or `gh-pr-review-fix` only when the user explicitly
  requests their mutation/hosting workflow.

Companion skills do not override repository instructions or this skill's hard
gates.

## Workflow

1. **Establish branch scope.** Read instructions and authority docs. Inspect
   status, merge-base diff, commits, generated consumers, and, when authorized,
   live PR checks/reviews. Separate task changes from unrelated dirt.
2. **Build the system map.** Trace each changed producer to consumers, public
   boundaries, data shapes, authorization, UI/native surfaces, tests, docs,
   operations, and generated artifacts.
3. **Run architect lanes.** Cover hard-cut shape, product completeness,
   performance/cost, QA, and docs. Add backend, auth/security, web, native,
   UI/UX, or integration profiles only when touched.
4. **Apply hard gates.** Report blockers before scores. Stop risky mutation when
   a product, security, data-loss, public-contract, or provider decision needs
   user approval.
5. **Score and resolve.** Show each active profile, criterion scores, weighted
   result, tradeoffs, and required action tier. Answer evidence-resolvable
   questions directly.
6. **Ask only when necessary.** If two viable options depend on user intent,
   use `grill-me`: one to three independent questions, recommended option first,
   mutually exclusive choices, and profile scores.
7. **Execute or report.** In implementation mode, make the smallest complete
   end-state change, delete the old path, update tests/docs, and avoid unrelated
   cleanup. In report-only mode, return an ordered, self-contained plan.
8. **Verify.** Rerun the narrow changed lane, then every wider repository gate
   selected by touched files. A gate not run is `UNVERIFIED`.
9. **Close out.** Reinspect diff and status, confirm no unresolved blocker or
   stale path remains, and report merge readiness without performing unasked
   Git or provider mutations.

## Severity

- **HIGH:** hard-gate failure or defect that can compromise security, data,
  contracts, release safety, core workflow completion, or merge confidence.
- **MEDIUM:** maintainability, performance, test, docs, or product-completeness
  gap that should be resolved before merge but is not itself a hard gate.
- **LOW:** bounded cleanup or clarity improvement with low operational risk.

## Review Output Format

For report-only mode:

```markdown
## Finalization Review
- Scope and evidence:
- Hard-gate blockers:
- Lane scores and profiles:
- Required hard cuts:
- Product completeness:
- Performance and cost:
- QA and validation:
- Docs and operations:
- Decisions needed:
- Merge readiness: Ready | Needs changes | Blocked | Inconclusive
- UNVERIFIED:
```

For implementation mode:

```markdown
## Finalization Implemented
- What changed and why:
- Canonical path retained:
- Compatibility/dead paths deleted:
- Files touched:
- Tests/docs aligned:
- Verification commands and results:
- Residual risks and UNVERIFIED gaps:
- Merge readiness: Ready | Needs changes | Blocked | Inconclusive
```

Read `references/review-framework.md` for lane checklists, weighted profiles,
and scoring discipline on broad or mixed branches.
