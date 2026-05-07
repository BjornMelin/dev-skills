---
description: Verification matrix for repo modernization work. Maps repo shapes to validation commands, evidence requirements, and stop conditions.
---

# Verification Matrix

Use this reference near the start of execution to define the expected
verification scope, and again at the end to decide whether the modernization is
actually complete.

## Core Rule

Do not treat “dependencies changed successfully” as completion.

Completion requires:
- dependency graph updated
- code migrated to current supported APIs
- stale/deprecated code removed
- relevant validations green
- residual risks explicitly called out

## Universal Checks

Run these in some repo-native form for every modernization task:

1. install/sync succeeds
2. vulnerability scan rerun after changes
3. outdated scan rerun after changes
4. lint passes
5. typecheck passes
6. tests pass for impacted areas
7. build passes for impacted areas
8. removed dependencies are no longer referenced
9. deprecated APIs are removed or intentionally isolated
10. stale compatibility code is gone

## Stop Conditions

Stop only when one of these is true:

### Green

- relevant validations are green
- post-change audit is clean or materially improved with documented residuals
- no unresolved migration work remains inside the touched scope

### Blocked

- an upstream package bug or ecosystem incompatibility prevents completion
- a real public/persisted compatibility boundary prevents hard-cut cleanup
- a repo-level secret, service, or environment dependency is unavailable

If blocked:
- name the exact blocker
- name the exact file/package/scope affected
- name the exact command that demonstrated the blocker
- separate blocked work from completed work

## Repo Shape Matrix

### Single-Package Library or App

Expected minimum:
- install/sync
- audit/outdated
- lint
- typecheck
- tests
- build

Evidence:
- exact command list
- post-change audit result
- any migration notes for public API changes

### Monorepo

Expected minimum:
- root install/sync
- audit/outdated at the appropriate root/workspace level
- targeted validation for impacted packages first
- broader graph validation after targeted fixes

Evidence:
- workspace graph awareness in the report
- impacted package list
- commands showing targeted and broad validation

### Bun-Managed Repo

Expected minimum:
- Bun-native install/update/audit/outdated verification
- repo-native lint/type/test/build scripts still green

Evidence:
- Bun command outputs summarized
- confirmation that Bun-specific runtime/package-manager behavior still works

### pnpm + Turbo Monorepo

Expected minimum:
- pnpm install/sync
- filtered Turbo validation for touched graph
- broader Turbo/root validation afterward

Evidence:
- filtered commands
- full graph or root follow-up
- confirmation that shared package/version alignment holds

## Framework Matrix

### Expo

Expected minimum:
- Expo dependency alignment complete
- `expo-doctor` run
- repo-native mobile validation run
- native/runtime-sensitive surfaces checked if touched

Stop if:
- SDK alignment is incomplete
- doctor still reports actionable breakage
- deprecated Expo packages remain in active use without documented reason

### Convex

Expected minimum:
- schema/functions/validators remain aligned
- repo-native Convex checks pass
- current official patterns used in touched areas

Stop if:
- validators/returns/schema drift remains
- indexes or runtime boundaries are left in an inconsistent state

### Next.js

Expected minimum:
- codemods applied where appropriate
- version-specific migration requirements satisfied
- build and type generation/typecheck pass
- touched routes/surfaces validated

Stop if:
- deprecated or removed APIs remain in touched scope
- framework version is bumped without corresponding code migration

### Turborepo

Expected minimum:
- targeted package validation
- broader pipeline validation
- task graph still coherent

Stop if:
- one package passes in isolation but the graph is inconsistent
- scripts and Turbo pipelines disagree on expected behavior

### Vercel

Expected minimum:
- relevant Vercel build/deploy/runtime assumptions verified
- stale platform workarounds removed only after current docs/tooling confirm it

Stop if:
- deployment/runtime config changed without validating the repo’s actual Vercel
  path

## Dependency Cleanup Checks

For every dependency removed or replaced, verify:
- no remaining imports
- no remaining scripts invoking it
- no remaining config referencing it
- no remaining docs instructing its use
- no indirect helper/wrapper survives without a purpose

Use repo-wide search to prove removals.

## Hard-Cut Completion Checks

Before calling the work done, verify:
- no fallback branch remains for the replaced shape
- no alias/shim/adapter remains solely for the old path
- no legacy-only tests remain in touched scope
- fixtures/snapshots/builders use only the canonical shape

## Reporting Checklist

The final report should include:
- commands run
- outcomes
- what passed
- what failed and why
- residual risks
- exact blockers if any
- net LOC added vs deleted

If verification is partial, say so explicitly. Do not present partial validation
as complete.
